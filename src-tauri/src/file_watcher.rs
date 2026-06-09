use crate::state::LightState;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

/// Per-session timer: if a session stays in Working for this long without any
/// new hook event, assume Claude Code is blocked on a permission prompt and
/// upgrade THIS session to Attention. The aggregate then surfaces red.
const WORKING_TO_ATTENTION_SECS: u64 = 2;

/// Drop sessions we haven't heard from in this long.
const SESSION_EXPIRE_SECS: u64 = 30 * 60;

#[derive(Debug, Deserialize)]
struct HookStateFile {
    state: Option<String>,
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    timestamp: Option<String>,
}

#[derive(Debug, Clone)]
struct SessionState {
    light: LightState,
    hook_timestamp: Option<String>,
    /// When we last saw a hook event for this session.
    last_event: Instant,
    /// True if the watchdog already promoted this session to Attention.
    promoted: bool,
}

pub struct FileWatcher {
    sessions_dir: PathBuf,
    running: bool,
}

impl FileWatcher {
    pub fn new() -> Self {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let sessions_dir = home.join(".claude-trafficlight").join("sessions");
        Self {
            sessions_dir,
            running: false,
        }
    }

    pub fn start(&mut self, app_handle: AppHandle) -> Result<(), String> {
        if self.running {
            return Err("FileWatcher is already running".to_string());
        }
        self.running = true;

        fs::create_dir_all(&self.sessions_dir)
            .map_err(|e| format!("Failed to create sessions dir: {}", e))?;

        let (tx, rx) = mpsc::channel::<Result<Event, notify::Error>>();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())
            .map_err(|e| format!("Failed to create file watcher: {}", e))?;
        watcher
            .watch(&self.sessions_dir, RecursiveMode::NonRecursive)
            .map_err(|e| format!("Failed to start watching: {}", e))?;

        let sessions_dir = self.sessions_dir.clone();
        let app = app_handle.clone();

        thread::spawn(move || {
            let mut sessions: HashMap<String, SessionState> = HashMap::new();

            // Initial scan
            Self::scan_dir(&sessions_dir, &mut sessions);
            let initial = Self::aggregate(&sessions);
            Self::emit_state(&app, initial);
            let mut last_emitted: Option<LightState> = Some(initial);

            loop {
                // Short timeout so the watchdog can tick deterministically.
                let _ = rx.recv_timeout(Duration::from_millis(250));

                // Re-scan the directory on every tick. Cheap (≤ a few files).
                Self::scan_dir(&sessions_dir, &mut sessions);

                // Per-session watchdog: a session that has been Working for
                // WORKING_TO_ATTENTION_SECS without a new event gets promoted.
                let now = Instant::now();
                for sess in sessions.values_mut() {
                    if sess.light == LightState::Working
                        && !sess.promoted
                        && now.duration_since(sess.last_event)
                            >= Duration::from_secs(WORKING_TO_ATTENTION_SECS)
                    {
                        sess.light = LightState::Attention;
                        sess.promoted = true;
                    }
                }

                // Expire old sessions
                sessions.retain(|_, s| {
                    now.duration_since(s.last_event)
                        < Duration::from_secs(SESSION_EXPIRE_SECS)
                });

                let agg = Self::aggregate(&sessions);
                if last_emitted != Some(agg) {
                    Self::emit_state(&app, agg);
                    last_emitted = Some(agg);
                }
            }
        });

        // Keep the notify watcher alive for the process lifetime.
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(3600));
            let _ = &watcher;
        });

        Ok(())
    }

    fn scan_dir(dir: &PathBuf, sessions: &mut HashMap<String, SessionState>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            // Skip stale .tmp.* atomic-write intermediates
            let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if name.contains(".tmp") {
                continue;
            }

            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(file) = serde_json::from_str::<HookStateFile>(&content) else {
                continue;
            };

            let session_id = file
                .session_id
                .clone()
                .unwrap_or_else(|| name.to_string());

            let light = match file.state.as_deref() {
                Some("starting") => LightState::Starting,
                Some("working") => LightState::Working,
                Some("thinking") => LightState::Thinking,
                Some("attention") => LightState::Attention,
                Some("error") => LightState::Error,
                Some("idle") => LightState::Idle,
                Some("done") => LightState::Done,
                Some("stopped") => LightState::Stopped,
                _ => continue,
            };

            let new_ts = file.timestamp.clone();
            let existing = sessions.get(&session_id);
            let timestamp_changed =
                existing.map(|s| s.hook_timestamp != new_ts).unwrap_or(true);

            if timestamp_changed {
                // Fresh event: reset watchdog and apply new state.
                sessions.insert(
                    session_id,
                    SessionState {
                        light,
                        hook_timestamp: new_ts,
                        last_event: Instant::now(),
                        promoted: false,
                    },
                );
            }
            // If the timestamp hasn't changed, we leave the session alone —
            // in particular, watchdog promotions (Attention) stay sticky.
        }
    }

    /// Aggregate priority: anything that demands user attention wins.
    fn aggregate(sessions: &HashMap<String, SessionState>) -> LightState {
        if sessions.is_empty() {
            return LightState::Stopped;
        }
        const PRIORITY: &[LightState] = &[
            LightState::Attention,
            LightState::Error,
            LightState::Working,
            LightState::Thinking,
            LightState::Starting,
            LightState::Done,
            LightState::Idle,
            LightState::Stopped,
        ];
        for p in PRIORITY {
            if sessions.values().any(|s| s.light == *p) {
                return *p;
            }
        }
        LightState::Stopped
    }

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

    pub fn is_running(&self) -> bool {
        self.running
    }
}
