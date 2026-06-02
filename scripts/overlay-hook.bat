@echo off
REM Claude Code Overlay Hook Script (Windows)
REM 该脚本在 Claude Code 的各种生命周期事件中被调用，
REM 将当前状态写入 %TEMP%\claude-overlay\state.json 供 Overlay 读取

setlocal enabledelayedexpansion

set "STATE_DIR=%TEMP%\claude-overlay"
set "STATE_FILE=%STATE_DIR%\state.json"

REM 确保目录存在
if not exist "%STATE_DIR%" mkdir "%STATE_DIR%"

REM 获取当前时间 (UTC)
for /f "skip=1 tokens=2-4 delims= " %%a in ('wmic os get localdatetime') do (
  set "DT=%%a"
  goto :got_time
)
:got_time
set "TIMESTAMP=%DT:~0,4%-%DT:~4,2%-%DT:~6,2%T%DT:~8,2%:%DT:~10,2%:%DT:~12,2%Z"

REM 写入状态文件
set "STATE=%1"
set "MESSAGE=%2"

if "%STATE%"=="" (
  if "%CLAUDE_HOOK%"=="onStartup" set "STATE=starting" & set "MESSAGE=Claude Code 启动中"
  if "%CLAUDE_HOOK%"=="onStartupComplete" set "STATE=idle" & set "MESSAGE=Claude Code 就绪"
  if "%CLAUDE_HOOK%"=="onShutdown" set "STATE=stopped" & set "MESSAGE=Claude Code 已停止"
  if "%CLAUDE_HOOK%"=="onTaskStart" set "STATE=working" & set "MESSAGE=任务开始"
  if "%CLAUDE_HOOK%"=="onTaskStop" set "STATE=done" & set "MESSAGE=任务完成"
  if "%CLAUDE_HOOK%"=="onSpawn" set "STATE=working" & set "MESSAGE=子任务启动"
  if "%CLAUDE_HOOK%"=="onRespawn" set "STATE=working" & set "MESSAGE=子任务恢复"
  if "%CLAUDE_HOOK%"=="onWait" set "STATE=thinking" & set "MESSAGE=等待输入"
  if "%CLAUDE_HOOK%"=="onPermissionRequest" set "STATE=attention" & set "MESSAGE=请求权限"
  if "%CLAUDE_HOOK%"=="onError" set "STATE=error" & set "MESSAGE=发生错误"
)

if "%STATE%"=="" set "STATE=stopped"
if "%MESSAGE%"=="" set "MESSAGE="

> "%STATE_FILE%" echo {"state": "%STATE%", "message": "%MESSAGE%", "timestamp": "%TIMESTAMP%"}
