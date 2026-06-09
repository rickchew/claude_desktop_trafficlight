# Claude Code Traffic Light

[English](README.md) · [中文](README-zh.md)

A small always-on-top desktop overlay that watches Claude Code CLI's state and shows it as a traffic light, so you know at a glance whether to look back at your terminal.

- 🟢 **Green** — idle / task complete
- 🟡 **Yellow** (blinking) — working / thinking
- 🔴 **Red** (blinking) — needs your input (permission request, etc.)
- 🔴 **Red** (solid) — error
- ⚫ **Gray** — stopped

> **Credits**
>
> Original project by [@kabumos](https://github.com/kabumos) — upstream: [kabumos/claude-code-overlay](https://github.com/kabumos/claude-code-overlay). All core functionality, design, and Rust/Svelte implementation are by the original author.
>
> This fork adds:
> - **macOS build fixes** — remove the hardcoded Windows MSVC target, enable Tauri `macos-private-api` so the window is truly transparent
> - **UI polish** — tighter padding, true-circular glow via `box-shadow` (the original `filter: blur` produced square artifacts at small sizes), inset top-highlight / bottom-shadow for a 3D look
> - **`glass` skin**
> - **macOS .app + .dmg packaging** signed with Developer ID and notarized by Apple
> - **One-click hooks install** from the right-click menu (no manual `settings.json` editing)
> - **Built-in English / 中文 toggle**
> - **Show / hide status text** option

## Install

### macOS — one click

1. Download the `.dmg` from [Releases](https://github.com/rickchew/claude_desktop_trafficlight/releases)
2. Drag **Claude Code Overlay** to `/Applications`
3. Open it (no Gatekeeper warning — signed + notarized)
4. Right-click the floating window → **Install Claude Code hooks** — done

A green checkmark confirms hooks were installed into `~/.claude/settings.json` (the file is automatically backed up to `settings.json.trafficlight-bak`). Now any `claude` CLI session will drive the lights.

### macOS — build from source

```bash
git clone https://github.com/rickchew/claude_desktop_trafficlight.git
cd claude_desktop_trafficlight
npm install
npm run tauri dev      # hot-reload development
npm run tauri build    # production .app + .dmg
```

Requires Node.js and Rust (`brew install rust`).

### Windows / Linux

See [the upstream project](https://github.com/kabumos/claude-code-overlay) for original platform support.

## Right-click menu

| Item | Effect |
|------|--------|
| **Start subprocess monitor** | Launch `claude` as a child process and parse its stdout for state |
| **Start file watcher** | Watch the hooks state file (default mode, started automatically) |
| **Stop monitoring** | Stop the current monitor |
| **Install Claude Code hooks** | Write `overlay-hook.sh` to `~/.claude-trafficlight/` and merge hook entries into `~/.claude/settings.json` (idempotent, backs up the original) |
| **Hide / Show status text** | Toggle the "Idle" / "Working" label under the lights |
| **Switch skin** | `default` / `glass` / `neon` / `minimal` |
| **Language** | English / 中文 (persisted to `~/.claude-trafficlight/config.json`) |
| **Debug** | Simulate any state for testing |
| **Quit** | Exit |

## Two monitoring modes

### Hooks (file watcher) — default

The overlay reads `/tmp/claude-overlay/state.json`. The hook script writes to it as Claude Code's lifecycle events fire. Install via the menu, or manually:

```bash
mkdir -p ~/.claude-trafficlight
curl -fsSL https://raw.githubusercontent.com/rickchew/claude_desktop_trafficlight/main/scripts/overlay-hook.sh \
  -o ~/.claude-trafficlight/overlay-hook.sh
chmod +x ~/.claude-trafficlight/overlay-hook.sh
```

Then add to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "SessionStart":     [{"matcher":"*","hooks":[{"type":"command","command":"bash \"~/.claude-trafficlight/overlay-hook.sh\" idle"}]}],
    "Stop":             [{"matcher":"*","hooks":[{"type":"command","command":"bash \"~/.claude-trafficlight/overlay-hook.sh\" idle"}]}],
    "SubagentStop":     [{"matcher":"*","hooks":[{"type":"command","command":"bash \"~/.claude-trafficlight/overlay-hook.sh\" idle"}]}],
    "UserPromptSubmit": [{"matcher":"*","hooks":[{"type":"command","command":"bash \"~/.claude-trafficlight/overlay-hook.sh\" working"}]}],
    "PreToolUse":       [{"matcher":"*","hooks":[{"type":"command","command":"bash \"~/.claude-trafficlight/overlay-hook.sh\" working"}]}],
    "PostToolUse":      [{"matcher":"*","hooks":[{"type":"command","command":"bash \"~/.claude-trafficlight/overlay-hook.sh\" working"}]}],
    "Notification":     [{"matcher":"*","hooks":[{"type":"command","command":"bash \"~/.claude-trafficlight/overlay-hook.sh\" attention"}]}]
  }
}
```

### Subprocess monitor

Right-click → **Start subprocess monitor**. The overlay starts `claude` itself and parses its stdout/stderr to detect state. Useful when you can't or don't want to touch `~/.claude/settings.json`.

## Skins

Themes live in `skins/<name>/theme.json`. Edit any field — colors, opacity, blur, border radius — restart the app to see the change. Add a new skin by creating a new folder; it appears automatically in **Switch skin**.

## Tech stack

- **Framework**: Tauri 2 + SvelteKit (Svelte 5)
- **Backend**: Rust (state detection, subprocess management, file watching, hooks install)
- **Frontend**: Svelte + CSS (traffic light UI, skin system)
- **Skins**: JSON-driven themes (`default` / `glass` / `neon` / `minimal`)

## License

MIT — same as upstream.
