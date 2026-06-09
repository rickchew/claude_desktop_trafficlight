use crate::state::LightState;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// Hooks 写入的状态文件格式
#[derive(Debug, Deserialize)]
struct HookStateFile {
    state: Option<String>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    timestamp: Option<String>,
}

/// 文件监听器 — 监听 Claude Code hooks 写入的状态文件
pub struct FileWatcher {
    state_path: PathBuf,
    running: bool,
    last_state: LightState,
}

impl FileWatcher {
    /// 创建新的文件监听器
    ///
    /// `state_path`: 状态文件路径，默认为临时目录下的 claude-overlay/state.json
    pub fn new(state_path: Option<PathBuf>) -> Self {
        let path = state_path.unwrap_or_else(|| {
            let temp = std::env::temp_dir();
            temp.join("claude-overlay").join("state.json")
        });

        Self {
            state_path: path,
            running: false,
            last_state: LightState::Stopped,
        }
    }

    /// 读取并解析状态文件
    fn read_state_file(&self) -> Result<LightState, String> {
        let content = fs::read_to_string(&self.state_path)
            .map_err(|e| format!("Failed to read state file: {}", e))?;

        let hook_state: HookStateFile = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse state file: {}", e))?;

        let state_str = hook_state.state.as_deref().unwrap_or("stopped");
        Ok(match state_str {
            "starting" => LightState::Starting,
            "working" => LightState::Working,
            "thinking" => LightState::Thinking,
            "attention" => LightState::Attention,
            "error" => LightState::Error,
            "idle" => LightState::Idle,
            "done" => LightState::Done,
            _ => LightState::Stopped,
        })
    }

    /// 启动文件监听
    pub fn start(&mut self, app_handle: AppHandle) -> Result<(), String> {
        if self.running {
            return Err("FileWatcher is already running".to_string());
        }
        self.running = true;

        // 确保目录存在
        if let Some(parent) = self.state_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
        }

        // 初始读取
        if let Ok(state) = self.read_state_file() {
            self.last_state = state;
            Self::emit_state(&app_handle, state);
        }

        // 使用 notify 监听文件变化
        let (tx, rx) = mpsc::channel::<Result<Event, notify::Error>>();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())
            .map_err(|e| format!("Failed to create file watcher: {}", e))?;

        let watch_path = self.state_path.clone();
        watcher
            .watch(
                watch_path.parent().unwrap_or(Path::new(".")),
                RecursiveMode::NonRecursive,
            )
            .map_err(|e| format!("Failed to start watching: {}", e))?;

        let app = app_handle.clone();
        let state_path = self.state_path.clone();

        thread::spawn(move || {
            // 也定期轮询作为后备（notify 在某些平台可能不可靠）
            let (poll_tx, poll_rx) = mpsc::channel();
            let poll_path = state_path.clone();
            let poll_app = app.clone();

            thread::spawn(move || {
                let mut last_modified = std::time::SystemTime::now();
                loop {
                    thread::sleep(Duration::from_millis(1000));
                    if let Ok(metadata) = fs::metadata(&poll_path) {
                        if let Ok(modified) = metadata.modified() {
                            if modified != last_modified {
                                last_modified = modified;
                                let _ = poll_tx.send(());
                            }
                        }
                    }
                }
            });

            // 处理通知和轮询事件
            loop {
                // 检查是否收到通知
                let _ = rx.recv_timeout(Duration::from_millis(500));
                let _ = poll_rx.recv_timeout(Duration::from_millis(100));

                // 读取状态文件
                if let Ok(content) = fs::read_to_string(&state_path) {
                    if let Ok(hook_state) =
                        serde_json::from_str::<HookStateFile>(&content)
                    {
                        let new_state = match hook_state.state.as_deref() {
                            Some("starting") => LightState::Starting,
                            Some("working") => LightState::Working,
                            Some("thinking") => LightState::Thinking,
                            Some("attention") => LightState::Attention,
                            Some("error") => LightState::Error,
                            Some("idle") => LightState::Idle,
                            Some("done") => LightState::Done,
                            _ => LightState::Stopped,
                        };
                        Self::emit_state(&poll_app, new_state);
                    }
                }
            }
        });

        // 将 watcher 移到后台（防止被 drop）
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(3600));
            }
        });

        Ok(())
    }

    /// 发送状态到前端
    fn emit_state(app_handle: &AppHandle, state: LightState) {
        let payload = serde_json::json!({
            "state": state,
            "colorGroup": state.color_group(),
            "animation": state.animation(),
            "blinkInterval": state.blink_interval_ms(),
            "label": crate::current_label(app_handle, state),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        let _ = app_handle.emit("overlay:state-change", &payload);
    }

    /// 检查是否在运行
    pub fn is_running(&self) -> bool {
        self.running
    }
}
