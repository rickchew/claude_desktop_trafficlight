# Claude Code 红绿灯悬浮窗

## 项目定位
半透明桌面红绿灯挂件，监控 Claude Code CLI 的工作状态，通过红黄绿三色灯提示是否需要交互。

## 技术栈
- **框架**：Tauri v2 + Svelte
- **后端**：Rust（子进程管理、状态检测引擎、文件监听）
- **前端**：Svelte + CSS（红绿灯 UI、皮肤系统）
- **目标**：跨平台 Win + Mac

## 架构要点
| 模式 | 说明 |
|------|------|
| 主模式 | 包裹启动 `claude` 子进程，实时解析 stdout 检测状态 |
| 备选模式 | 通过 Claude Code hooks 写状态文件，Tauri 监听文件变化 |
| 皮肤系统 | JSON 驱动的主题，可切换显示风格 |

## 红绿灯状态映射
| 检测状态 | 灯色 | 行为 | 触发条件 |
|---------|------|------|---------|
| `starting` | 🟡 黄 | 长亮 | 子进程启动 |
| `working` | 🟡 黄 | 慢闪 1s | 持续 stdout 输出 |
| `thinking` | 🟡 黄 | 慢闪 1.5s | 输出短暂停顿 |
| `attention` | 🔴 红 | 快闪 300ms | 检测到 `? (y/N)` `Allow?` 等交互提示 |
| `error` | 🔴 红 | 长亮 | 检测到 error/panic |
| `idle` | 🟢 绿 | 长亮 | 输出停止 >5s，未在等待 |
| `done` | 🟢 绿 | 呼吸 2s | 任务完成标记 |
| `stopped` | ⚫ 灰 | 熄灭 | 子进程退出 |

## 项目文件结构
```
claude-code-overlay/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs           # 入口、系统托盘
│   │   ├── monitor.rs        # 子进程管理、stdout 读取
│   │   ├── detector.rs       # 状态检测引擎
│   │   ├── file_watcher.rs   # Hooks 文件监听
│   │   ├── state.rs          # 状态机
│   │   └── skins.rs          # 皮肤管理
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── main.ts
│   ├── App.svelte            # 主组件（拖拽、右键菜单）
│   ├── TrafficLight.svelte   # 红绿灯组件
│   ├── StatusText.svelte     # 状态文字
│   ├── SkinManager.ts
│   └── types.ts
├── skins/
│   ├── default/theme.json
│   ├── neon/theme.json
│   └── minimal/theme.json
├── scripts/
│   ├── overlay-hook.sh
│   └── overlay-hook.bat
└── package.json
```

## 实施步骤
1. 项目初始化：Tauri v2 + Svelte 脚手架，配置透明窗口
2. Rust 后端：状态检测引擎（detector.rs）、状态机（state.rs）、子进程管理（monitor.rs）
3. Rust 后端：Hooks 文件监听（file_watcher.rs）
4. 前端 UI：TrafficLight + StatusText + 窗口拖拽
5. 皮肤系统：theme.json 解析 + 3 套初始皮肤
6. 集成调试：IPC 对接 + 实际场景测试
7. Hook 脚本 + 配置文档

## 验证方式
- 启动 overlay → 黄灯长亮/慢闪
- Claude 工作 → 黄灯闪烁
- Claude 请求权限 → 红灯快闪
- 任务完成 → 绿灯亮
- 右键切换皮肤 → UI 变化
- 内存占用 < 20MB
