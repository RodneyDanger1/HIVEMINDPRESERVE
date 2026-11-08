@echo off
rem Agent-native HiveMind CLI wrapper. No provider key is handled here.
node "%~dp0hivemind-cli.mjs" %*
exit /b %ERRORLEVEL%
