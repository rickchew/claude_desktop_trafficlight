use crate::state::{LightState, StateMachine};

/// 检测引擎 — 处理来自子进程或文件监听的输出行
pub struct Detector {
    state_machine: StateMachine,
    running: bool,
    tick_interval_ms: u64,
}

impl Detector {
    pub fn new() -> Self {
        Self {
            state_machine: StateMachine::new(),
            running: false,
            tick_interval_ms: 500, // 每 500ms 检查一次超时
        }
    }

    /// 处理一行输出并返回新的状态
    pub fn process_line(&mut self, line: &str) -> LightState {
        self.state_machine.on_output(line)
    }

    /// 获取当前状态
    pub fn current_state(&self) -> LightState {
        self.state_machine.current()
    }

    /// 执行一次 tick 检测（由外部定时调用）
    pub fn tick(&mut self) -> Option<LightState> {
        self.state_machine.tick()
    }

    /// 重置检测器
    pub fn reset(&mut self) {
        self.state_machine.reset();
    }

    /// 设置为启动状态
    pub fn set_starting(&mut self) {
        self.state_machine.set_starting();
    }

    /// 设置为已停止
    pub fn set_stopped(&mut self) {
        self.state_machine.reset();
    }
}

/// 输出行模式匹配 — 用于快速预检
pub mod patterns {
    /// 交互提示模式
    pub const ATTENTION_PATTERNS: &[&str] = &[
        "? (y/N)",
        "? (Y/n)",
        "? (y/N)",
        "Allow?",
        "[y/N]",
        "[Y/n]",
        "? Allow",
        "? Proceed",
    ];

    /// 错误模式
    pub const ERROR_PATTERNS: &[&str] = &[
        "error:",
        "Error:",
        "panic!",
        "fatal:",
        "panicked at",
    ];

    /// 完成模式
    pub const DONE_PATTERNS: &[&str] = &[
        "✓",
        "✅",
        "task completed",
        "Done!",
        "finished",
    ];

    /// 检查行是否匹配任何模式
    pub fn matches_any(line: &str, patterns: &[&str]) -> bool {
        patterns.iter().any(|p| line.contains(p))
    }
}
