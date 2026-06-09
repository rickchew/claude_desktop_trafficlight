mod detector;
mod file_watcher;
mod i18n;
mod monitor;
mod setup;
mod skins;
mod state;

use i18n::{menu_strings, state_label, Lang};
use monitor::Monitor;
use serde::Serialize;
use serde_json::json;
use skins::SkinManager;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{
    menu::{ContextMenu, MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, State,
};

/// 应用全局状态
pub struct AppState {
    pub monitor: Mutex<Monitor>,
    pub file_watcher: Mutex<Option<file_watcher::FileWatcher>>,
    pub skin_manager: Mutex<SkinManager>,
    pub lang: Mutex<Lang>,
    pub show_label: Mutex<bool>,
}

fn config_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".claude-trafficlight").join("config.json"))
}

fn read_config() -> serde_json::Value {
    let Some(p) = config_path() else { return json!({}) };
    let Ok(raw) = std::fs::read_to_string(&p) else { return json!({}) };
    serde_json::from_str(&raw).unwrap_or_else(|_| json!({}))
}

fn write_config(v: &serde_json::Value) {
    let Some(p) = config_path() else { return };
    let _ = std::fs::create_dir_all(p.parent().unwrap());
    if let Ok(s) = serde_json::to_string_pretty(v) {
        let _ = std::fs::write(&p, s);
    }
}

fn load_lang() -> Lang {
    match read_config().get("lang").and_then(|s| s.as_str()) {
        Some("zh") => Lang::Zh,
        Some("en") => Lang::En,
        _ => Lang::default(),
    }
}

fn save_lang(lang: Lang) {
    let mut v = read_config();
    let key = match lang { Lang::Zh => "zh", Lang::En => "en" };
    if let Some(o) = v.as_object_mut() {
        o.insert("lang".to_string(), json!(key));
    } else {
        v = json!({"lang": key});
    }
    write_config(&v);
}

fn load_show_label() -> bool {
    read_config().get("show_label").and_then(|b| b.as_bool()).unwrap_or(true)
}

fn save_show_label(show: bool) {
    let mut v = read_config();
    if let Some(o) = v.as_object_mut() {
        o.insert("show_label".to_string(), json!(show));
    } else {
        v = json!({"show_label": show});
    }
    write_config(&v);
}

/// Get current language from AppState (safe; falls back to Zh)
pub fn current_lang(app: &AppHandle) -> Lang {
    app.try_state::<AppState>()
        .and_then(|s| s.lang.lock().ok().map(|g| *g))
        .unwrap_or(Lang::Zh)
}

/// Localized status label for a LightState — call from emit sites
pub fn current_label(app: &AppHandle, st: state::LightState) -> String {
    let key = format!("{:?}", st).to_lowercase();
    state_label(current_lang(app), &key).to_string()
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

fn make_payload(app: &AppHandle, ls: state::LightState) -> StatePayload {
    StatePayload {
        state: format!("{:?}", ls).to_lowercase(),
        color_group: ls.color_group().to_string(),
        animation: ls.animation().to_string(),
        blink_interval: ls.blink_interval_ms(),
        label: current_label(app, ls),
        timestamp: chrono::Utc::now().to_rfc3339(),
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
    let result = monitor.start(app.clone());
    if result.is_ok() {
        emit_source(&app, "process");
    }
    result
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
        *fw = Some(file_watcher::FileWatcher::new());
    }
    if let Some(ref mut watcher) = *fw {
        let result = watcher.start(app.clone());
        if result.is_ok() {
            emit_source(&app, "files");
        }
        result
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
    let payload = make_payload(&app, ls);
    emit_source(&app, "simulation");
    app.emit("overlay:state-change", &payload)
        .map_err(|e| e.to_string())
}

/// Install Claude Code hooks into ~/.claude/settings.json
#[tauri::command]
fn install_hooks_cmd() -> Result<String, String> {
    setup::install_hooks()
}

/// Switch UI language and persist
#[tauri::command]
fn set_language(app: AppHandle, state: State<AppState>, lang: String) -> Result<(), String> {
    let new_lang = match lang.as_str() {
        "en" => Lang::En,
        _ => Lang::Zh,
    };
    *state.lang.lock().map_err(|e| e.to_string())? = new_lang;
    save_lang(new_lang);
    // Re-emit current state with new label so the UI updates immediately
    let current = state.monitor.lock().ok()
        .map(|m| m.current_state())
        .unwrap_or(state::LightState::Stopped);
    let payload = make_payload(&app, current);
    let _ = app.emit("overlay:state-change", &payload);
    let _ = app.emit("overlay:lang-change", &json!({"lang": lang}));
    Ok(())
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

/// 发射监控模式事件（前端显示当前模式）
fn emit_source(app_handle: &AppHandle, source: &str) {
    let payload = serde_json::json!({"source": source});
    let _ = app_handle.emit("overlay:source-change", &payload);
}

/// 菜单事件处理器（托盘和右键弹出菜单共享）
fn handle_menu_event(app_handle: &AppHandle, id: &str) {
    match id {
        "toggle" => {
            if let Some(window) = app_handle.get_webview_window("main") {
                if window.is_visible().unwrap_or(true) {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            };
        }
        "quit" => {
            let state = app_handle.state::<AppState>();
            if let Ok(mut monitor) = state.monitor.lock() {
                monitor.stop(app_handle);
            };
            app_handle.exit(0);
        }
        "start-monitor" => {
            let state = app_handle.state::<AppState>();
            if let Ok(mut monitor) = state.monitor.lock() {
                if monitor.start(app_handle.clone()).is_ok() {
                    emit_source(app_handle, "process");
                }
            };
        }
        "start-filewatcher" => {
            let state = app_handle.state::<AppState>();
            if let Ok(mut fw) = state.file_watcher.lock() {
                if fw.is_none() {
                    *fw = Some(file_watcher::FileWatcher::new());
                }
                if let Some(ref mut watcher) = *fw {
                    if watcher.start(app_handle.clone()).is_ok() {
                        emit_source(app_handle, "files");
                    }
                }
            };
        }
        "stop-monitor" => {
            let state = app_handle.state::<AppState>();
            if let Ok(mut monitor) = state.monitor.lock() {
                monitor.stop(app_handle);
            };
            emit_source(app_handle, "none");
        }
        id if id.starts_with("skin-") => {
            let name = &id[5..];
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
            };
        }
        id if id.starts_with("simulate-") => {
            let state_name = &id[9..];
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
            let payload = make_payload(app_handle, ls);
            let _ = app_handle.emit("overlay:state-change", &payload);
        }
        "toggle-label" => {
            let state = app_handle.state::<AppState>();
            let new_val = if let Ok(mut g) = state.show_label.lock() {
                *g = !*g;
                save_show_label(*g);
                *g
            } else { true };
            let _ = app_handle.emit("overlay:show-label-change", &json!({"show": new_val}));
        }
        "install-hooks" => {
            match setup::install_hooks() {
                Ok(msg) => {
                    let _ = app_handle.emit("overlay:notice", &json!({"kind":"ok","message": msg}));
                }
                Err(err) => {
                    let _ = app_handle.emit("overlay:notice", &json!({"kind":"err","message": err}));
                }
            }
        }
        id if id == "lang-zh" || id == "lang-en" => {
            if let Some(new_lang) = Lang::from_id(id) {
                let state = app_handle.state::<AppState>();
                if let Ok(mut g) = state.lang.lock() {
                    *g = new_lang;
                    save_lang(new_lang);
                }
                let current = state.monitor.lock().ok()
                    .map(|m| m.current_state())
                    .unwrap_or(state::LightState::Stopped);
                let payload = make_payload(app_handle, current);
                let _ = app_handle.emit("overlay:state-change", &payload);
                let _ = app_handle.emit("overlay:lang-change", &json!({"lang": match new_lang { Lang::Zh => "zh", Lang::En => "en" }}));
            }
        }
        _ => {}
    }
}

/// Build the menu used by both the right-click context menu and the tray.
/// `include_window_toggle` adds a "Show / hide window" entry at the top
/// (tray menu only).
fn build_menu(
    app: &AppHandle,
    include_window_toggle: bool,
) -> Result<tauri::menu::Menu<tauri::Wry>, tauri::Error> {
    let app_state = app.state::<AppState>();
    let lang = *app_state.lang.lock().expect("lang mutex poisoned");
    let show_label_now = *app_state.show_label.lock().expect("show_label mutex poisoned");
    let m = menu_strings(lang);

    let skin_names: Vec<String> = app_state
        .skin_manager
        .lock()
        .expect("skin_manager mutex poisoned")
        .list()
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    let mut skin_sub = SubmenuBuilder::new(app, m.switch_skin);
    for name in &skin_names {
        let item = MenuItemBuilder::with_id(format!("skin-{}", name), name.as_str()).build(app)?;
        skin_sub = skin_sub.item(&item);
    }
    let skin_submenu = skin_sub.build()?;

    let mut debug_sub = SubmenuBuilder::new(app, m.debug);
    for (id, label) in &[
        ("simulate-starting", m.sim_starting),
        ("simulate-working", m.sim_working),
        ("simulate-thinking", m.sim_thinking),
        ("simulate-attention", m.sim_attention),
        ("simulate-error", m.sim_error),
        ("simulate-idle", m.sim_idle),
        ("simulate-done", m.sim_done),
    ] {
        let item = MenuItemBuilder::with_id(*id, *label).build(app)?;
        debug_sub = debug_sub.item(&item);
    }
    let debug_submenu = debug_sub.build()?;

    let lang_sub = SubmenuBuilder::new(app, m.language)
        .item(&MenuItemBuilder::with_id("lang-zh", "中文").build(app)?)
        .item(&MenuItemBuilder::with_id("lang-en", "English").build(app)?)
        .build()?;

    let start_mon = MenuItemBuilder::with_id("start-monitor", m.start_process).build(app)?;
    let start_fw = MenuItemBuilder::with_id("start-filewatcher", m.start_files).build(app)?;
    let stop_mon = MenuItemBuilder::with_id("stop-monitor", m.stop_monitor).build(app)?;
    let install = MenuItemBuilder::with_id("install-hooks", m.setup_hooks).build(app)?;
    let toggle_label = MenuItemBuilder::with_id(
        "toggle-label",
        if show_label_now { m.hide_label } else { m.show_label },
    )
    .build(app)?;
    let quit = MenuItemBuilder::with_id("quit", m.quit).build(app)?;

    let mut mb = MenuBuilder::new(app);
    if include_window_toggle {
        let toggle = MenuItemBuilder::with_id("toggle", m.toggle).build(app)?;
        mb = mb.item(&toggle).separator();
    }
    mb.item(&start_mon)
        .item(&start_fw)
        .item(&stop_mon)
        .separator()
        .item(&install)
        .item(&toggle_label)
        .separator()
        .item(&skin_submenu)
        .item(&lang_sub)
        .separator()
        .item(&debug_submenu)
        .separator()
        .item(&quit)
        .build()
}

/// 显示右键上下文菜单（弹出原生系统菜单）
#[tauri::command]
async fn show_context_menu(app: AppHandle, _x: f64, _y: f64) -> Result<(), String> {
    let menu = build_menu(&app, false).map_err(|e| e.to_string())?;
    let window = app.get_webview_window("main").ok_or("No main window".to_string())?;
    app.run_on_main_thread(move || {
        let _ = menu.popup(window.as_ref().window().clone());
    })
    .map_err(|e| e.to_string())
}

/// 设置系统托盘
fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let lang = *app.state::<AppState>().lang.lock().unwrap();
    let m = menu_strings(lang);
    let menu = build_menu(app, true)?;

    let mut tray = TrayIconBuilder::new().tooltip(m.tray_tooltip).menu(&menu);
    if let Some(icon) = app.default_window_icon().cloned() {
        tray = tray.icon(icon);
    }
    tray = tray.on_menu_event(move |tray, event| {
        let app = tray.app_handle();
        handle_menu_event(&app, event.id().as_ref());
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
            lang: Mutex::new(load_lang()),
            show_label: Mutex::new(load_show_label()),
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
            show_context_menu,
            install_hooks_cmd,
            set_language,
        ])
        .setup(|app| {
            // 全局菜单事件处理器（右键弹出菜单等非托盘菜单）
            app.on_menu_event(|app_handle, event| {
                handle_menu_event(app_handle, event.id().as_ref());
            });

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

            // 发送初始 lang + show_label 给前端
            let lang = *handle.state::<AppState>().lang.lock().unwrap();
            let show = *handle.state::<AppState>().show_label.lock().unwrap();
            let _ = handle.emit("overlay:lang-change", &json!({"lang": match lang { Lang::Zh => "zh", Lang::En => "en" }}));
            let _ = handle.emit("overlay:show-label-change", &json!({"show": show}));

            // 设置系统托盘
            if let Err(e) = setup_tray(app.handle()) {
                eprintln!("Failed to setup tray: {}", e);
            }

            // 自动启动文件监听（Hooks 模式）
            {
                let handle = app.handle().clone();
                let state = handle.state::<AppState>();
                if let Ok(mut fw) = state.file_watcher.lock() {
                    if fw.is_none() {
                        *fw = Some(file_watcher::FileWatcher::new());
                    }
                    if let Some(ref mut watcher) = *fw {
                        if watcher.start(handle.clone()).is_ok() {
                            emit_source(&handle, "files");
                        }
                    }
                };
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
