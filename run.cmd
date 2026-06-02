@echo off
REM Claude Code Overlay - 生产模式启动脚本
REM 先构建完整 bundle，然后启动

call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
if errorlevel 1 (
  echo 找不到 Visual Studio 2022 Build Tools
  pause
  exit /b 1
)

set PATH=%PATH%;%USERPROFILE%\.cargo\bin
cd /d "%~dp0"

echo ^>^> 构建前端...
call npm run build
if errorlevel 1 (
  echo 前端构建失败
  pause
  exit /b 1
)

echo ^>^> 构建 Tauri 应用...
cd src-tauri
cargo build --release
if errorlevel 1 (
  echo Rust 构建失败
  pause
  exit /b 1
)

echo ^>^> 启动 Claude Code Overlay...
start "" "target/x86_64-pc-windows-msvc/release/claude-code-overlay.exe"
