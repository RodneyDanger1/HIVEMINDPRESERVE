@echo off
setlocal
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0tools\start-hivemind.ps1" %*
if errorlevel 1 pause
endlocal
