@echo off
rem HiveMind agent-native CLI (headless). stdout is JSON when piped.
node "%~dp0tools\hivemind-cli.mjs" %*
exit /b %ERRORLEVEL%
