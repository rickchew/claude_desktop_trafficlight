use crate::detector::Detector;
use crate::state::LightState;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::thread;
use tauri::{AppHandle, Emitter};

/// 子进程管理器 — 启动和管理 claude 进程
pub struct Monitor {
    process: Option<Child>,
    detector: Detector,
    running: bool,
}

impl Monitor {
    pub fn new() -> Self {
        Self {
            process: None,
            detector: Detector::new(),
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
        self.detector.set_starting();
        self.emit_state(&app_handle, LightState::Starting);

        let stdout = process.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = process.stderr.take().ok_or("Failed to capture stderr")?;

        // 启动 stdout 读取线程
        let app = app_handle.clone();
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    // 发送到主线程处理
                    let _ = app.emit("overlay:raw-output", &line);
                }
            }
        });

        // 启动 stderr 读取线程
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let _ = app_handle.emit("overlay:raw-output", &line);
                }
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
        self.detector.set_stopped();
        self.emit_state(app_handle, LightState::Stopped);
    }

    /// 处理一行输出（可由 stdout 线程或文件 watcher 调用）
    pub fn process_line(&mut self, line: &str, app_handle: &AppHandle) -> LightState {
        let new_state = self.detector.process_line(line);
        self.emit_state(app_handle, new_state);
        new_state
    }

    /// 发送状态到前端
    fn emit_state(&self, app_handle: &AppHandle, state: LightState) {
        let payload = serde_json::json!({
            "state": state,
            "colorGroup": state.color_group(),
            "animation": state.animation(),
            "blinkInterval": state.blink_interval_ms(),
            "label": state.label(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        let _ = app_handle.emit("overlay:state-change", &payload);
    }

    /// 检查进程是否在运行
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// 获取当前状态
    pub fn current_state(&self) -> LightState {
        self.detector.current_state()
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
