use crate::state::{LightState, StateMachine};

/// 检测引擎 — 处理来自子进程的输出行
pub struct Detector {
    state_machine: StateMachine,
}

impl Detector {
    pub fn new() -> Self {
        Self {
            state_machine: StateMachine::new(),
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

    /// 设置为启动状态
    pub fn set_starting(&mut self) {
        self.state_machine.set_starting();
    }

    /// 设置为已停止
    pub fn set_stopped(&mut self) {
        self.state_machine.reset();
    }
}
