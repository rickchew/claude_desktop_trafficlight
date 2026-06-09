#!/bin/bash
# Claude Code Overlay Hook Script
# 配置：在 .claude/settings.json 中设置 hooks 即可
# 该脚本在 Claude Code 的各种生命周期事件中被调用，
# 将当前状态写入 /tmp/claude-overlay/state.json 供 Overlay 读取

set -euo pipefail

STATE_DIR="${TMPDIR:-/tmp}/claude-overlay"
STATE_FILE="$STATE_DIR/state.json"

# 确保目录存在
mkdir -p "$STATE_DIR"

# 写入状态
write_state() {
  local state="$1"
  local message="${2:-}"
  local timestamp
  timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

  cat > "$STATE_FILE" << EOF
{
  "state": "$state",
  "message": $(echo "$message" | jq -Rs '.' 2>/dev/null || echo "\"$message\""),
  "timestamp": "$timestamp"
}
EOF
}

# 根据 Hook 事件类型处理
case "${CLAUDE_HOOK:-}" in
  "onStartup")
    write_state "starting" "Claude Code 启动中"
    ;;
  "onStartupComplete")
    write_state "idle" "Claude Code 就绪"
    ;;
  "onShutdown")
    write_state "stopped" "Claude Code 已停止"
    ;;
  "onTaskStart")
    write_state "working" "任务开始"
    ;;
  "onTaskStop")
    write_state "done" "任务完成"
    ;;
  "onSpawn")
    write_state "working" "子任务启动"
    ;;
  "onRespawn")
    write_state "working" "子任务恢复"
    ;;
  "onWait")
    write_state "thinking" "等待输入"
    ;;
  "onPermissionRequest")
    write_state "attention" "请求权限"
    ;;
  "onError")
    write_state "error" "发生错误"
    ;;
  *)
    # 如果传递了参数，直接使用
    if [ $# -ge 1 ]; then
      write_state "$1" "${2:-}"
    fi
    ;;
esac
