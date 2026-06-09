#!/bin/bash
# Double-click to launch the dev build (hot-reload). Ctrl+C in this window to stop.
cd "$(dirname "$0")" || exit 1
echo "▶ Starting Claude Traffic Light (dev mode)..."
echo "  Frontend changes hot-reload. Rust changes auto-rebuild."
echo "  Close this terminal window or hit Ctrl+C to stop."
echo ""
exec npm run tauri dev
