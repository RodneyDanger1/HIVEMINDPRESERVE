# Live capability walkthrough for the portable HiveMind package.
#
# Runs the shipped CLI/server on a TEMP profile/port, exercises the real HTTP,
# CLI, MCP and eval surfaces, then stops the server. No provider key is read or
# used; no paid calls. Writes a JSON receipt and exits non-zero on any failure.
[CmdletBinding()]
param(
    [int]$Port = 8167,
    [string]$Package = ''
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
if (-not $Package) { $Package = Join-Path $root 'dist\HiveMind' }
if (-not (Test-Path (Join-Path $Package 'hivemind.cmd'))) { throw "package missing: $Package" }
$tmp = Join-Path $env:TEMP ('hm-live-walk-' + [guid]::NewGuid().ToString('N'))
$data = Join-Path $tmp 'data'
$workspace = Join-Path $tmp 'workspace'
New-Item -ItemType Directory -Force -Path $workspace, $data | Out-Null
$results = [System.Collections.Generic.List[object]]::new()
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
function Record([string]$Name, [bool]$Ok, [string]$Detail) {
    $results.Add([pscustomobject]@{ name = $Name; ok = $Ok; detail = $Detail })
    Write-Host ("{0}  {1}: {2}" -f $(if ($Ok) { 'PASS' } else { 'FAIL' }), $Name, $Detail)
}
function Api([string]$Method, [string]$Path, $Body = $null, $Headers = @{}) {
    $uri = "http://127.0.0.1:$Port$Path"
    $args = @{ Uri = $uri; Method = $Method; Headers = $Headers; ContentType = 'application/json' }
    if ($null -ne $Body) { $args.Body = ($Body | ConvertTo-Json -Depth 8 -Compress) }
    Invoke-RestMethod @args
}
Push-Location $Package
$started = $false
try {
    $start = & cmd.exe /c "hivemind.cmd start --port $Port --data-dir `"$data`" --workspace-root `"$workspace`" --json" 2>&1 | Out-String
    $started = $LASTEXITCODE -eq 0
    Record 'package start' $started ($start.Trim().Split("`n")[-1])
    if (-not $started) { throw 'server did not start' }

    $health = Api 'GET' '/health'
    Record 'health schema' ($health.database.schema_version -ge 31) ("schema=" + $health.database.schema_version)

    $session = Api 'GET' '/api/session'
    $headers = @{ Authorization = "Bearer $($session.token)" }
    Record 'session token' ([bool]$session.token) 'token present (value redacted)'

    $named = Api 'POST' '/api/sessions' @{ name = 'walkthrough'; project_id = 'walk'; workspace_root = $workspace; permission_policy = 'classic' } $headers
    Record 'session create' ($named.name -eq 'walkthrough') ("id=" + $named.id)

    $todo = Api 'POST' ("/api/sessions/" + $named.id + '/todos') @{ text = 'verify every new capability' } $headers
    Record 'todo create' ($todo.state -eq 'open') ("state=" + $todo.state)

    $evalBody = @{ suite_id = 'walk'; case_id = 'cargo-version'; program = 'cargo.exe'; args = @('--version'); expected_exit_code = 0 }
    $eval = Api 'POST' '/api/eval/run' $evalBody $headers
    Record 'executed eval' ([bool]$eval.passed -and $eval.actual_exit -eq 0) ("state=" + $eval.state)

    $suiteBody = @{ name = 'walk-suite'; cases = @(@{ case_id = 'cargo'; program = 'cargo.exe'; args = @('--version'); expected_exit_code = 0 }) }
    $suite = Api 'POST' '/api/eval/suites' $suiteBody $headers
    $run = Api 'POST' ("/api/eval/suites/" + $suite.id + '/run') $null $headers
    Record 'eval suite metrics' ($run.metrics.total -eq 1 -and $run.metrics.passed -eq 1) ("passed=" + $run.metrics.passed + "/" + $run.metrics.total)

    $jobs = Api 'GET' '/api/jobs' $null $headers
    Record 'jobs list' ($null -ne $jobs.jobs) ("count=" + @($jobs.jobs).Count)

    $skills = Api 'GET' '/api/skills' $null $headers
    Record 'skills list' ($null -ne $skills.skills) ("count=" + @($skills.skills).Count)

    $index = Invoke-WebRequest -Uri "http://127.0.0.1:$Port/" -UseBasicParsing
    Record 'embedded UI' ($index.StatusCode -eq 200 -and $index.Content -match 'HiveMind') 'index served'

    $mcpRequest = '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
    $mcpOut = $mcpRequest | node hivemind-mcp.mjs 2>&1 | Out-String
    Record 'MCP tools/list' ($mcpOut -match 'hivemind_status') 'initialize/list responded'
} catch {
    Record 'walkthrough exception' $false $_.Exception.Message
} finally {
    if ($started) { & cmd.exe /c "hivemind.cmd stop --data-dir `"$data`" --json --yes" 2>&1 | Out-Null }
    Pop-Location
    $passed = @($results | Where-Object { $_.ok }).Count
    $total = $results.Count
    $receipt = [pscustomobject]@{
        schema = 'hivemind.live-walkthrough.v1'
        generated_at = (Get-Date).ToString('o')
        package = $Package
        port = $Port
        passed = $passed
        total = $total
        results = $results
    }
    $out = Join-Path $env:TEMP 'hivemind-live-walkthrough.json'
    $receipt | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $out -Encoding UTF8
    Write-Host ("RESULT: {0}/{1} PASS -> {2}" -f $passed, $total, $out)
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
    if ($passed -ne $total -or $total -eq 0) { exit 1 }
}
