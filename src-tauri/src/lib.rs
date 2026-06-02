// 开发阶段允许未使用的代码
#![allow(dead_code)]

mod detector;
mod file_watcher;
mod monitor;
mod skins;
mod state;

use monitor::Monitor;
use serde::Serialize;
use skins::SkinManager;
use std::sync::Mutex;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, State,
};

/// 应用全局状态
pub struct AppState {
    pub monitor: Mutex<Monitor>,
    pub file_watcher: Mutex<Option<file_watcher::FileWatcher>>,
    pub skin_manager: Mutex<SkinManager>,
}

/// 状态响应载荷
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatePayload {
    pub state: String,
    pub color_group: String,
    pub animation: String,
    pub blink_interval: u64,
    pub label: String,
    pub timestamp: String,
}

impl From<state::LightState> for StatePayload {
    fn from(ls: state::LightState) -> Self {
        Self {
            state: format!("{:?}", ls).to_lowercase(),
            color_group: ls.color_group().to_string(),
            animation: ls.animation().to_string(),
            blink_interval: ls.blink_interval_ms(),
            label: ls.label().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// 皮肤信息载荷
#[derive(Debug, Clone, Serialize)]
pub struct SkinPayload {
    pub name: String,
    pub description: String,
    pub lights: skins::LightColors,
    pub background: skins::BackgroundConfig,
    pub border: skins::BorderConfig,
    pub label: skins::TextStyle,
}

/// 启动子进程监控模式
#[tauri::command]
fn start_monitor(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    let mut monitor = state.monitor.lock().map_err(|e| e.to_string())?;
    monitor.start(app)
}

/// 停止子进程监控
#[tauri::command]
fn stop_monitor(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    let mut monitor = state.monitor.lock().map_err(|e| e.to_string())?;
    monitor.stop(&app);
    Ok(())
}

/// 启动文件监听模式
#[tauri::command]
fn start_file_watcher(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    let mut fw = state.file_watcher.lock().map_err(|e| e.to_string())?;
    if fw.is_none() {
        *fw = Some(file_watcher::FileWatcher::new(None));
    }
    if let Some(ref mut watcher) = *fw {
        watcher.start(app)
    } else {
        Err("Failed to create file watcher".to_string())
    }
}

/// 模拟状态（用于测试）
#[tauri::command]
fn simulate_state(app: AppHandle, state_name: String) -> Result<(), String> {
    let ls = match state_name.to_lowercase().as_str() {
        "starting" => state::LightState::Starting,
        "working" => state::LightState::Working,
        "thinking" => state::LightState::Thinking,
        "attention" => state::LightState::Attention,
        "error" => state::LightState::Error,
        "idle" => state::LightState::Idle,
        "done" => state::LightState::Done,
        "stopped" => state::LightState::Stopped,
        _ => return Err(format!("Unknown state: {}", state_name)),
    };
    let payload: StatePayload = ls.into();
    app.emit("overlay:state-change", &payload)
        .map_err(|e| e.to_string())
}

/// 切换皮肤
#[tauri::command]
fn switch_skin(state: State<AppState>, name: String) -> Result<SkinPayload, String> {
    let mut sm = state.skin_manager.lock().map_err(|e| e.to_string())?;
    match sm.switch(&name) {
        Some(skin) => Ok(SkinPayload {
            name: skin.name.clone(),
            description: skin.description.clone(),
            lights: skin.lights.clone(),
            background: skin.background.clone(),
            border: skin.border.clone(),
            label: skin.label.clone(),
        }),
        None => Err(format!("Skin '{}' not found", name)),
    }
}

/// 获取当前皮肤
#[tauri::command]
fn get_current_skin(state: State<AppState>) -> Result<SkinPayload, String> {
    let sm = state.skin_manager.lock().map_err(|e| e.to_string())?;
    match sm.current() {
        Some(skin) => Ok(SkinPayload {
            name: skin.name.clone(),
            description: skin.description.clone(),
            lights: skin.lights.clone(),
            background: skin.background.clone(),
            border: skin.border.clone(),
            label: skin.label.clone(),
        }),
        None => Err("No skin loaded".to_string()),
    }
}

/// 获取皮肤列表
#[tauri::command]
fn list_skins(state: State<AppState>) -> Result<Vec<String>, String> {
    let sm = state.skin_manager.lock().map_err(|e| e.to_string())?;
    Ok(sm.list().into_iter().map(|s| s.to_string()).collect())
}

/// 退出应用
#[tauri::command]
fn exit_app(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    if let Ok(mut monitor) = state.monitor.lock() {
        monitor.stop(&app);
    }
    app.exit(0);
    Ok(())
}

/// 设置系统托盘
fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // 获取皮肤列表用于菜单
    let skin_names = {
        let state = app.state::<AppState>();
        let sm = state.skin_manager.lock().unwrap();
        sm.list().into_iter().map(|s| s.to_string()).collect::<Vec<_>>()
    };

    // 构建皮肤子菜单
    let mut skin_sub = SubmenuBuilder::new(app, "切换皮肤");
    for name in &skin_names {
        let item = MenuItemBuilder::with_id(format!("skin-{}", name), name.as_str()).build(app)?;
        skin_sub = skin_sub.item(&item);
    }
    let skin_submenu = skin_sub.build()?;

    // 构建调试子菜单
    let mut debug_sub = SubmenuBuilder::new(app, "调试");
    for (id, label) in &[
        ("simulate-starting", "启动中"),
        ("simulate-working", "工作中"),
        ("simulate-thinking", "思考中"),
        ("simulate-attention", "需要交互"),
        ("simulate-error", "错误"),
        ("simulate-idle", "空闲"),
        ("simulate-done", "完成"),
    ] {
        let item = MenuItemBuilder::with_id(*id, *label).build(app)?;
        debug_sub = debug_sub.item(&item);
    }
    let debug_submenu = debug_sub.build()?;

    // 主菜单项
    let toggle = MenuItemBuilder::with_id("toggle", "显示/隐藏窗口").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&toggle)
        .separator()
        .item(&skin_submenu)
        .item(&debug_submenu)
        .separator()
        .item(&quit)
        .build()?;

    // 创建托盘图标
    let mut tray = TrayIconBuilder::new()
        .tooltip("Claude Code 红绿灯")
        .menu(&menu);

    if let Some(icon) = app.default_window_icon().cloned() {
        tray = tray.icon(icon);
    }

    // 菜单事件处理
    let app_handle = app.clone();
    tray = tray.on_menu_event(move |_app, event| {
        let id = event.id().as_ref().to_string();

        // 显示/隐藏
        if id == "toggle" {
            if let Some(window) = app_handle.get_webview_window("main") {
                if window.is_visible().unwrap_or(true) {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            return;
        }

        // 退出
        if id == "quit" {
            app_handle.exit(0);
            return;
        }

        // 切换皮肤
        if let Some(ref name) = id.strip_prefix("skin-") {
            let state = app_handle.state::<AppState>();
            if let Ok(mut sm) = state.skin_manager.lock() {
                if let Some(skin) = sm.switch(name) {
                    let payload = SkinPayload {
                        name: skin.name.clone(),
                        description: skin.description.clone(),
                        lights: skin.lights.clone(),
                        background: skin.background.clone(),
                        border: skin.border.clone(),
                        label: skin.label.clone(),
                    };
                    let _ = app_handle.emit("overlay:skin-change", &payload);
                }
            }
            return;
        }

        // 模拟状态
        if let Some(state_name) = id.strip_prefix("simulate-") {
            let ls = match state_name {
                "starting" => state::LightState::Starting,
                "working" => state::LightState::Working,
                "thinking" => state::LightState::Thinking,
                "attention" => state::LightState::Attention,
                "error" => state::LightState::Error,
                "idle" => state::LightState::Idle,
                "done" => state::LightState::Done,
                _ => return,
            };
            let payload: StatePayload = ls.into();
            let _ = app_handle.emit("overlay:state-change", &payload);
        }
    });

    tray.build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let skins_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(&std::path::PathBuf::from("."))
        .join("skins");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            monitor: Mutex::new(Monitor::new()),
            file_watcher: Mutex::new(None),
            skin_manager: Mutex::new(SkinManager::new(skins_dir)),
        })
        .invoke_handler(tauri::generate_handler![
            start_monitor,
            stop_monitor,
            start_file_watcher,
            simulate_state,
            switch_skin,
            get_current_skin,
            list_skins,
            exit_app,
        ])
        .setup(|app| {
            // 发送初始皮肤
            let handle = app.handle().clone();
            let sm = handle.state::<AppState>();
            if let Ok(skin_mgr) = sm.skin_manager.lock() {
                if let Some(skin) = skin_mgr.current() {
                    let payload = SkinPayload {
                        name: skin.name.clone(),
                        description: skin.description.clone(),
                        lights: skin.lights.clone(),
                        background: skin.background.clone(),
                        border: skin.border.clone(),
                        label: skin.label.clone(),
                    };
                    let _ = handle.emit("overlay:skin-change", &payload);
                }
            }

            // 设置系统托盘
            if let Err(e) = setup_tray(app.handle()) {
                eprintln!("Failed to setup tray: {}", e);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
