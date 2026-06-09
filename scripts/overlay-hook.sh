#!/bin/bash
# Claude Code Overlay Hook Script
#
# Called by Claude Code lifecycle hooks. Reads stdin JSON (which includes
# session_id), then writes a per-session JSON file under:
#   ~/.claude-trafficlight/sessions/<session_id>.json
#
# The overlay's Rust file_watcher reads the directory, maintains a session
# map, and computes the aggregate state with attention > error > working > ...
# priority. Multi-session safe; no locks required (one writer per file).

STATE="${1:-stopped}"
MESSAGE="${2:-}"

STDIN_JSON=""
if [ ! -t 0 ]; then
  STDIN_JSON=$(cat)
fi

# Single python3 invocation does parse + timestamp + atomic write.
STATE="$STATE" MESSAGE="$MESSAGE" STDIN_JSON="$STDIN_JSON" /usr/bin/python3 - <<'PY'
import datetime
import json
import os
import re
import sys
import tempfile

state = os.environ.get("STATE", "stopped")
message = os.environ.get("MESSAGE", "")
raw = os.environ.get("STDIN_JSON", "")

sessions_dir = os.path.expanduser("~/.claude-trafficlight/sessions")
os.makedirs(sessions_dir, exist_ok=True)

session_id = "default"
if raw.strip():
    try:
        data = json.loads(raw)
        candidate = data.get("session_id") or data.get("sessionId") or ""
        if candidate:
            # Sanitize: only allow filename-safe characters
            candidate = re.sub(r"[^A-Za-z0-9._-]", "_", str(candidate))
            if candidate:
                session_id = candidate
    except Exception:
        pass

payload = {
    "session_id": session_id,
    "state": state,
    "message": message,
    "timestamp": datetime.datetime.utcnow().isoformat() + "Z",
}

session_file = os.path.join(sessions_dir, f"{session_id}.json")
fd, tmp = tempfile.mkstemp(dir=sessions_dir, prefix=f".{session_id}.", suffix=".tmp")
try:
    with os.fdopen(fd, "w") as f:
        json.dump(payload, f)
    os.replace(tmp, session_file)
except Exception:
    try:
        os.unlink(tmp)
    except OSError:
        pass
PY

exit 0
