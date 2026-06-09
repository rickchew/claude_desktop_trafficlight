use serde_json::{json, Value};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

const HOOK_SCRIPT: &str = include_str!("../../scripts/overlay-hook.sh");

const EVENTS: &[(&str, &str)] = &[
    ("SessionStart", "idle"),
    ("Stop", "idle"),
    ("SubagentStop", "idle"),
    ("UserPromptSubmit", "working"),
    ("PreToolUse", "working"),
    ("PostToolUse", "working"),
    ("Notification", "attention"),
];

fn home_dir() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME not set".to_string())
}

/// Write overlay-hook.sh to ~/.claude-trafficlight/ and merge hook entries
/// into ~/.claude/settings.json. Idempotent; backs up settings to .bak.
pub fn install_hooks() -> Result<String, String> {
    let home = home_dir()?;
    let hook_dir = home.join(".claude-trafficlight");
    let hook_path = hook_dir.join("overlay-hook.sh");
    let settings_dir = home.join(".claude");
    let settings_path = settings_dir.join("settings.json");

    // 1. Write hook script
    fs::create_dir_all(&hook_dir).map_err(|e| format!("mkdir {}: {}", hook_dir.display(), e))?;
    fs::write(&hook_path, HOOK_SCRIPT).map_err(|e| format!("write hook script: {}", e))?;
    let mut perms = fs::metadata(&hook_path)
        .map_err(|e| e.to_string())?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&hook_path, perms).map_err(|e| e.to_string())?;

    // 2. Read existing settings
    fs::create_dir_all(&settings_dir).map_err(|e| e.to_string())?;
    let mut settings: Value = if settings_path.exists() {
        let raw = fs::read_to_string(&settings_path).map_err(|e| e.to_string())?;
        // Back up before touching
        let bak = settings_path.with_extension("json.trafficlight-bak");
        let _ = fs::write(&bak, &raw);
        if raw.trim().is_empty() {
            json!({})
        } else {
            serde_json::from_str(&raw).map_err(|e| format!("settings.json parse: {}", e))?
        }
    } else {
        json!({})
    };

    // 3. Merge hook entries
    let hook_cmd_for = |state: &str| format!("bash \"{}\" {}", hook_path.display(), state);

    let hooks_obj = settings
        .as_object_mut()
        .ok_or_else(|| "settings.json root is not an object".to_string())?
        .entry("hooks")
        .or_insert(json!({}));
    let hooks_obj = hooks_obj
        .as_object_mut()
        .ok_or_else(|| "settings.hooks is not an object".to_string())?;

    let mut added = 0usize;
    for (event, state) in EVENTS {
        let entries = hooks_obj
            .entry(event.to_string())
            .or_insert(json!([]))
            .as_array_mut()
            .ok_or_else(|| format!("settings.hooks.{} is not an array", event))?;

        let cmd = hook_cmd_for(state);
        let already = entries.iter().any(|e| {
            e.get("hooks")
                .and_then(|h| h.as_array())
                .map(|arr| {
                    arr.iter().any(|h| {
                        h.get("command")
                            .and_then(|c| c.as_str())
                            .map(|s| s == cmd)
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
        });
        if already {
            continue;
        }
        entries.push(json!({
            "matcher": "*",
            "hooks": [{"type": "command", "command": cmd}]
        }));
        added += 1;
    }

    // 4. Write back
    let serialized =
        serde_json::to_string_pretty(&settings).map_err(|e| format!("serialize: {}", e))?;
    fs::write(&settings_path, serialized).map_err(|e| format!("write settings: {}", e))?;

    if added == 0 {
        Ok("Already installed".to_string())
    } else {
        Ok(format!("Installed {} hooks", added))
    }
}
