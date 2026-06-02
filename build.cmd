@echo off
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
if errorlevel 1 exit /b 1
set PATH=%PATH%;%USERPROFILE%\.cargo\bin
cd /d "%~dp0"
call npm run build
if errorlevel 1 exit /b 1
cd src-tauri
cargo build --release
if errorlevel 1 exit /b 1
echo "=== Build complete ==="
