use crate::detector::Detector;
use crate::state::LightState;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// 子进程管理器 — 启动和管理 claude 进程
pub struct Monitor {
    process: Option<Child>,
    detector: Arc<Mutex<Detector>>,
    running: bool,
}

impl Monitor {
    pub fn new() -> Self {
        Self {
            process: None,
            detector: Arc::new(Mutex::new(Detector::new())),
            running: false,
        }
    }

    /// 启动 claude 子进程
    pub fn start(&mut self, app_handle: AppHandle) -> Result<(), String> {
        if self.running {
            return Err("Monitor is already running".to_string());
        }

        let mut process = Command::new("claude")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start claude process: {}", e))?;

        self.running = true;
        {
            let mut det = self.detector.lock().unwrap();
            det.set_starting();
        }
        Self::emit_state(&app_handle, LightState::Starting);

        let stdout = process.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = process.stderr.take().ok_or("Failed to capture stderr")?;

        let detector = self.detector.clone();

        // Stdout 处理线程
        let det = detector.clone();
        let app = app_handle.clone();
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let state = {
                        let mut d = det.lock().unwrap();
                        d.process_line(&line)
                    };
                    Self::emit_state(&app, state);
                }
            }
        });

        // Stderr 处理线程
        let det = detector.clone();
        let app_stderr = app_handle.clone();
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let state = {
                        let mut d = det.lock().unwrap();
                        d.process_line(&line)
                    };
                    Self::emit_state(&app_stderr, state);
                }
            }
        });

        // Ticker 线程：每 500ms 检查超时（Working→Thinking, any→Idle）
        let det = detector.clone();
        let app_tick = app_handle.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(500));
            let changed = {
                let mut d = det.lock().unwrap();
                d.tick()
            };
            if let Some(state) = changed {
                Self::emit_state(&app_tick, state);
            }
        });

        self.process = Some(process);
        Ok(())
    }

    /// 停止子进程
    pub fn stop(&mut self, app_handle: &AppHandle) {
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
        self.running = false;
        {
            let mut det = self.detector.lock().unwrap();
            det.set_stopped();
        }
        Self::emit_state(app_handle, LightState::Stopped);
    }

    /// 处理一行输出（供外部调用，例如 file_watcher 转接）
    pub fn process_line(&mut self, line: &str, app_handle: &AppHandle) -> LightState {
        let state = self.detector.lock().unwrap().process_line(line);
        Self::emit_state(app_handle, state);
        state
    }

    /// 检查进程是否在运行
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// 获取当前状态
    pub fn current_state(&self) -> LightState {
        self.detector.lock().unwrap().current_state()
    }

    fn emit_state(app_handle: &AppHandle, state: LightState) {
        let payload = serde_json::json!({
            "state": format!("{:?}", state).to_lowercase(),
            "colorGroup": state.color_group(),
            "animation": state.animation(),
            "blinkInterval": state.blink_interval_ms(),
            "label": state.label(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        let _ = app_handle.emit("overlay:state-change", &payload);
    }
}

impl Drop for Monitor {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
    }
}
