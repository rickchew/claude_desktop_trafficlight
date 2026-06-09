use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    Zh,
    En,
}

impl Default for Lang {
    fn default() -> Self {
        Lang::En
    }
}

impl Lang {
    pub fn from_id(id: &str) -> Option<Lang> {
        match id {
            "lang-zh" => Some(Lang::Zh),
            "lang-en" => Some(Lang::En),
            _ => None,
        }
    }
}

/// State labels shown in the overlay status text
pub fn state_label(lang: Lang, key: &str) -> &'static str {
    match (lang, key) {
        (Lang::Zh, "starting") => "启动中",
        (Lang::Zh, "working") => "工作中",
        (Lang::Zh, "thinking") => "思考中",
        (Lang::Zh, "attention") => "需要交互",
        (Lang::Zh, "error") => "错误",
        (Lang::Zh, "idle") => "空闲",
        (Lang::Zh, "done") => "完成",
        (Lang::Zh, "stopped") => "已停止",
        (Lang::En, "starting") => "Starting",
        (Lang::En, "working") => "Working",
        (Lang::En, "thinking") => "Thinking",
        (Lang::En, "attention") => "Needs input",
        (Lang::En, "error") => "Error",
        (Lang::En, "idle") => "Idle",
        (Lang::En, "done") => "Done",
        (Lang::En, "stopped") => "Stopped",
        _ => "",
    }
}

/// Menu strings
pub struct MenuStrings {
    pub start_process: &'static str,
    pub start_files:   &'static str,
    pub stop_monitor:  &'static str,
    pub setup_hooks:   &'static str,
    pub switch_skin:   &'static str,
    pub debug:         &'static str,
    pub language:      &'static str,
    pub show_label:    &'static str,
    pub hide_label:    &'static str,
    pub quit:          &'static str,
    pub toggle:        &'static str,
    pub sim_starting:  &'static str,
    pub sim_working:   &'static str,
    pub sim_thinking:  &'static str,
    pub sim_attention: &'static str,
    pub sim_error:     &'static str,
    pub sim_idle:      &'static str,
    pub sim_done:      &'static str,
    pub tray_tooltip:  &'static str,
}

pub fn menu_strings(lang: Lang) -> MenuStrings {
    match lang {
        Lang::Zh => MenuStrings {
            start_process: "启动子进程监控",
            start_files:   "启动文件监听",
            stop_monitor:  "停止监控",
            setup_hooks:   "安装 Claude Code Hooks",
            switch_skin:   "切换皮肤",
            debug:         "调试",
            language:      "语言",
            show_label:    "显示状态文字",
            hide_label:    "隐藏状态文字",
            quit:          "退出",
            toggle:        "显示/隐藏窗口",
            sim_starting:  "启动中",
            sim_working:   "工作中",
            sim_thinking:  "思考中",
            sim_attention: "需要交互",
            sim_error:     "错误",
            sim_idle:      "空闲",
            sim_done:      "完成",
            tray_tooltip:  "Claude Code 红绿灯",
        },
        Lang::En => MenuStrings {
            start_process: "Start subprocess monitor",
            start_files:   "Start file watcher",
            stop_monitor:  "Stop monitoring",
            setup_hooks:   "Install Claude Code hooks",
            switch_skin:   "Switch skin",
            debug:         "Debug",
            language:      "Language",
            show_label:    "Show status text",
            hide_label:    "Hide status text",
            quit:          "Quit",
            toggle:        "Show / hide window",
            sim_starting:  "Starting",
            sim_working:   "Working",
            sim_thinking:  "Thinking",
            sim_attention: "Needs input",
            sim_error:     "Error",
            sim_idle:      "Idle",
            sim_done:      "Done",
            tray_tooltip:  "Claude Code Traffic Light",
        },
    }
}
