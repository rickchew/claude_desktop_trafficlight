use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Instant;

/// 红绿灯状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightState {
    /// 🟡 黄 常亮 — 子进程启动
    Starting,
    /// 🟡 黄 慢闪 1s — 持续 stdout 输出
    Working,
    /// 🟡 黄 慢闪 1.5s — 输出短暂停顿
    Thinking,
    /// 🔴 红 快闪 300ms — 检测到交互提示
    Attention,
    /// 🔴 红 常亮 — 检测到 error/panic
    Error,
    /// 🟢 绿 常亮 — 输出停止 >5s，未在等待
    Idle,
    /// 🟢 绿 呼吸 2s — 任务完成标记
    Done,
    /// ⚫ 灰 熄灭 — 子进程退出
    Stopped,
}

impl LightState {
    /// 获取灯色类别
    pub fn color_group(&self) -> &str {
        match self {
            LightState::Starting | LightState::Working | LightState::Thinking => "yellow",
            LightState::Attention | LightState::Error => "red",
            LightState::Idle | LightState::Done => "green",
            LightState::Stopped => "gray",
        }
    }

    /// 获取动画类型
    pub fn animation(&self) -> &str {
        match self {
            LightState::Starting => "solid",
            LightState::Working => "slow-blink",
            LightState::Thinking => "slow-blink",
            LightState::Attention => "fast-blink",
            LightState::Error => "solid",
            LightState::Idle => "solid",
            LightState::Done => "breathing",
            LightState::Stopped => "off",
        }
    }

    /// 获取动画间隔 (ms)
    pub fn blink_interval_ms(&self) -> u64 {
        match self {
            LightState::Starting => 0,
            LightState::Working => 1000,
            LightState::Thinking => 1500,
            LightState::Attention => 300,
            LightState::Error => 0,
            LightState::Idle => 0,
            LightState::Done => 2000,
            LightState::Stopped => 0,
        }
    }

    /// 获取中文状态名称
    pub fn label(&self) -> &str {
        match self {
            LightState::Starting => "启动中",
            LightState::Working => "工作中",
            LightState::Thinking => "思考中",
            LightState::Attention => "需要交互",
            LightState::Error => "错误",
            LightState::Idle => "空闲",
            LightState::Done => "完成",
            LightState::Stopped => "已停止",
        }
    }
}

impl fmt::Display for LightState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// 状态机 — 管理状态转换和超时
#[derive(Debug, Clone)]
pub struct StateMachine {
    current: LightState,
    last_output: Instant,
    last_state_change: Instant,
    idle_timeout_ms: u64,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            current: LightState::Stopped,
            last_output: Instant::now(),
            last_state_change: Instant::now(),
            idle_timeout_ms: 5000, // 5s 无输出进入 Idle
        }
    }

    pub fn current(&self) -> LightState {
        self.current
    }

    /// 记录输出活动，更新状态
    pub fn on_output(&mut self, line: &str) -> LightState {
        self.last_output = Instant::now();

        let new_state = self.detect_state(line);

        if new_state != self.current {
            self.current = new_state;
            self.last_state_change = Instant::now();
        }

        self.current
    }

    /// 检测空闲超时（每次 tick 调用）
    pub fn tick(&mut self) -> Option<LightState> {
        if self.current == LightState::Stopped {
            return None;
        }

        let elapsed = self.last_output.elapsed().as_millis() as u64;

        // 如果超过空闲超时且当前不是交互/错误状态，切换到 Idle
        if elapsed > self.idle_timeout_ms
            && self.current != LightState::Attention
            && self.current != LightState::Error
            && self.current != LightState::Stopped
            && self.current != LightState::Starting
        {
            if self.current != LightState::Idle {
                self.current = LightState::Idle;
                self.last_state_change = Instant::now();
                return Some(self.current);
            }
        }

        // Working -> Thinking 过渡：短暂停顿（>2s 无输出但未到 idle）
        if self.current == LightState::Working && elapsed > 2000 && elapsed <= self.idle_timeout_ms {
            if self.current != LightState::Thinking {
                self.current = LightState::Thinking;
                self.last_state_change = Instant::now();
                return Some(self.current);
            }
        }

        None
    }

    /// 重置状态
    pub fn reset(&mut self) {
        self.current = LightState::Stopped;
        self.last_output = Instant::now();
        self.last_state_change = Instant::now();
    }

    pub fn set_starting(&mut self) {
        self.current = LightState::Starting;
        self.last_output = Instant::now();
        self.last_state_change = Instant::now();
    }

    /// 基于一行输出检测状态
    fn detect_state(&self, line: &str) -> LightState {
        let lower = line.to_lowercase();

        // 优先级 1: 交互提示 (最快响应)
        if line.contains("? (y/n)")
            || line.contains("? (Y/n)")
            || line.contains("? (y/N)")
            || line.contains("allow?")
            || line.contains("Allow?")
            || line.contains("permission?")
            || lower.contains("allow")
            || line.contains("[y/N]")
            || line.contains("[Y/n]")
        {
            return LightState::Attention;
        }

        // 优先级 2: 错误
        if lower.starts_with("error:")
            || lower.starts_with("error ")
            || lower.contains("panic!")
            || lower.contains("fatal:")
            || (lower.starts_with("thread '") && lower.contains("panicked at"))
        {
            return LightState::Error;
        }

        // 优先级 3: 任务完成标记
        if lower.contains("task completed")
            || lower.contains("done!")
            || lower.contains("finished")
            || lower.contains("✓")
            || lower.contains("✅")
        {
            return LightState::Done;
        }

        // 优先级 4: 启动中
        if self.current == LightState::Stopped && !line.is_empty() {
            return LightState::Starting;
        }

        // 默认: 工作中
        if !line.is_empty() {
            return LightState::Working;
        }

        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attention_detection() {
        let mut sm = StateMachine::new();
        sm.set_starting();

        let state = sm.on_output("? (y/N)");
        assert_eq!(state, LightState::Attention);
    }

    #[test]
    fn test_error_detection() {
        let mut sm = StateMachine::new();
        sm.set_starting();

        let state = sm.on_output("error: could not compile");
        assert_eq!(state, LightState::Error);
    }

    #[test]
    fn test_working_state() {
        let mut sm = StateMachine::new();
        sm.set_starting();

        let state = sm.on_output("some normal output");
        assert_eq!(state, LightState::Working);
    }

    #[test]
    fn test_done_detection() {
        let mut sm = StateMachine::new();
        sm.set_starting();

        let state = sm.on_output("Task completed successfully ✓");
        assert_eq!(state, LightState::Done);
    }
}
