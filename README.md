# Claude Code 红绿灯悬浮窗

半透明桌面红绿灯挂件，监控 Claude Code CLI 的工作状态，通过红黄绿三色灯提示是否需要交互。

> **致谢 / Credits**
>
> 本项目原作者：[@kabumos](https://github.com/kabumos) — 上游仓库 [kabumos/claude-code-overlay](https://github.com/kabumos/claude-code-overlay)。所有核心功能、设计、Rust/Svelte 实现均由原作者完成。
>
> 本 fork 仅做：
> - **macOS 兼容性修复** — 移除硬编码的 Windows MSVC 构建目标、启用 `macOSPrivateApi` 让窗口真正透明
> - **UI 微调** — 调整窗口尺寸、间距，把灯的光晕从 `filter:blur`（小尺寸下方块伪影）改成 `box-shadow`（真圆形）、面板加 inset 高光做 3D 感
> - **新增 `glass` 主题**
> - **macOS .app + .dmg 打包**（Developer ID 签名 + Apple 公证）
>
> Releases 里提供 macOS 已签名 + 公证的 `.dmg`，双击直接安装运行。

## 功能

- **🟢 绿灯** — 空闲 / 任务完成
- **🟡 黄灯闪烁** — 工作中 / 思考中
- **🔴 红灯闪烁** — 需要交互（权限请求等）
- **🔴 红灯常亮** — 发生错误
- **⚫ 灰色** — 已停止

## 两种监控模式

### 1. Hooks 文件监听（推荐）

通过 Claude Code 的 `hooks` 机制，将运行状态写入文件，Overlay 监听文件变化。

### 2. 子进程监控

Overlay 直接启动 `claude` 子进程，实时解析 stdout/stderr 检测状态。

---

## 配置 Hooks（文件监听模式）

### 步骤 1：放置 Hook 脚本

**scripts/** 目录下已提供 hook 脚本：

| 脚本 | 用途 |
|------|------|
| `scripts/overlay-hook.bat` | Windows |
| `scripts/overlay-hook.sh` | macOS / Linux |

复制对应脚本到 Claude Code 项目目录或全局配置中。

### 步骤 2：配置 Claude Code Hooks

在 Claude Code 的项目配置文件（项目根目录的 `.claude/settings.json`）或全局配置（`~/.claude/settings.json`）中添加：

```json
{
  "hooks": {
    "onStartup": "<脚本路径>/overlay-hook.bat",
    "onStartupComplete": "<脚本路径>/overlay-hook.bat",
    "onShutdown": "<脚本路径>/overlay-hook.bat",
    "onTaskStart": "<脚本路径>/overlay-hook.bat",
    "onTaskStop": "<脚本路径>/overlay-hook.bat",
    "onSpawn": "<脚本路径>/overlay-hook.bat",
    "onRespawn": "<脚本路径>/overlay-hook.bat",
    "onWait": "<脚本路径>/overlay-hook.bat",
    "onPermissionRequest": "<脚本路径>/overlay-hook.bat",
    "onError": "<脚本路径>/overlay-hook.bat"
  }
}
```

Hook 脚本根据环境变量 `CLAUDE_HOOK` 自动判断事件类型，并写入状态文件到 `%TEMP%/claude-overlay/state.json`（Windows）或 `/tmp/claude-overlay/state.json`（macOS/Linux）。

### 安装示例（Windows）

```batch
REM 将 hooks 目录添加到 Claude Code 全局设置
notepad %USERPROFILE%\.claude\settings.json
```

然后在 `.claude/settings.json` 中使用**绝对路径**指向 `overlay-hook.bat`：

```json
{
  "hooks": {
    "onTaskStart": "D:\\workspaces\\claude\\claude-code-overlay\\scripts\\overlay-hook.bat",
    "onTaskStop": "D:\\workspaces\\claude\\claude-code-overlay\\scripts\\overlay-hook.bat",
    "onPermissionRequest": "D:\\workspaces\\claude\\claude-code-overlay\\scripts\\overlay-hook.bat",
    "onError": "D:\\workspaces\\claude\\claude-code-overlay\\scripts\\overlay-hook.bat"
  }
}
```

> 提示：不在 `.claude/settings.json` 中注册的事件不会被调用 — 按需添加即可。

---

## 右键菜单

| 操作 | 效果 |
|------|------|
| **右键红绿灯窗口** | 弹出原生菜单（监控控制 / 切换皮肤 / 模拟调试 / 退出） |
| **右键系统托盘图标** | 同样菜单 + 显示/隐藏窗口 |
| **左键拖拽窗口** | 移动红绿灯位置 |

### 菜单项说明

- **启动子进程监控** — 直接启动 `claude` 进程并检测输出
- **启动文件监听** — 监听 Hooks 状态文件（启动时默认启动）
- **停止监控** — 停止当前监控
- **切换皮肤** — default / neon / minimal
- **调试** — 模拟 7 种状态用于测试

---

## 开发

```bash
# 安装依赖
npm install

# 开发模式（热重载）
npm run tauri dev

# 生产构建
npm run build
cd src-tauri
cargo build --release
```

### 构建说明（Windows）

需要 MSVC 构建工具链：

```bash
# 设置 MSVC 环境
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"

# 构建
cargo build --release
```

或使用 `run.cmd` 一键构建 + 启动。

---

## 技术栈

- **框架**: Tauri v2 + SvelteKit (Svelte 5)
- **后端**: Rust（状态检测引擎、子进程管理、文件监听）
- **前端**: Svelte + CSS（红绿灯 UI、皮肤系统）
- **皮肤**: JSON 驱动的主题，3 套初始皮肤（default / neon / minimal）
