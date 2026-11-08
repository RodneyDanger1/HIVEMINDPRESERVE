#!/usr/bin/env pwsh
# HiveMind agent-native CLI (headless). stdout is JSON when piped.
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$CliArgs
)
& node (Join-Path $PSScriptRoot 'tools\hivemind-cli.mjs') @CliArgs
exit $LASTEXITCODE
