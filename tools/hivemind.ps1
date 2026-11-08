#!/usr/bin/env pwsh
# Agent-native HiveMind CLI wrapper. No provider key is handled here.
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$CliArgs
)
& node (Join-Path $PSScriptRoot 'hivemind-cli.mjs') @CliArgs
exit $LASTEXITCODE
