@echo off
REM Claude Code Overlay - 开发模式启动脚本
REM 双击运行即可启动 Tauri 开发模式（支持热更新）

call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
if errorlevel 1 (
  echo 找不到 Visual Studio 2022 Build Tools
  echo 请安装 https://visualstudio.microsoft.com/visual-cpp-build-tools/
  pause
  exit /b 1
)

set PATH=%PATH%;%USERPROFILE%\.cargo\bin
cd /d "%~dp0"

echo ^>^> 启动 Tauri 开发模式...
echo ^>^> 红绿灯悬浮窗将自动弹出
npx tauri dev

pause
