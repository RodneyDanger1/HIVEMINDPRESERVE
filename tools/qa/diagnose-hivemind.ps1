# Read-only HiveMind runtime/ghost diagnostic.
# Reports one launcher, one server, the port owner, the app window count, stale
# runtime records, and the tail of the runtime logs. Exit 1 means an anomaly.
param(
    [string]$DataRoot = (Join-Path $env:LOCALAPPDATA 'HiveMind'),
    [int]$Port = 8099,
    [int]$LogLines = 12,
    [switch]$Json
)

$ErrorActionPreference = 'Stop'
$problems = New-Object System.Collections.Generic.List[string]
$launchers = @(Get-CimInstance Win32_Process -Filter "Name='HiveMind.exe'" -ErrorAction SilentlyContinue)
$servers = @(Get-CimInstance Win32_Process -Filter "Name='hiverouter.exe'" -ErrorAction SilentlyContinue)
$windows = @(Get-Process msedge, chrome -ErrorAction SilentlyContinue |
    Where-Object { $_.MainWindowTitle -like 'HiveMind*Spatial*' })
$listeners = @(Get-NetTCPConnection -State Listen -ErrorAction SilentlyContinue |
    Where-Object { $_.LocalPort -ge 8099 -and $_.LocalPort -le 8199 })

if ($launchers.Count -gt 1) { $problems.Add("multiple launchers: $($launchers.Count)") }
if ($servers.Count -gt 1) { $problems.Add("multiple servers: $($servers.Count)") }
if ($windows.Count -gt 1) { $problems.Add("multiple app windows: $($windows.Count)") }

$runtimePath = Join-Path $DataRoot 'runtime.json'
$runtime = $null
if (Test-Path -LiteralPath $runtimePath) {
    try { $runtime = Get-Content -LiteralPath $runtimePath -Raw | ConvertFrom-Json }
    catch { $problems.Add('runtime.json is unreadable') }
} else {
    $problems.Add('runtime.json is missing')
}

if ($runtime) {
    $recorded = @($servers | Where-Object { $_.ProcessId -eq [int]$runtime.pid })
    if ($recorded.Count -eq 0) {
        $problems.Add("runtime.json pid $($runtime.pid) is not a live server (stale record)")
    }
    $owner = @($listeners | Where-Object { $_.LocalPort -eq [int]$runtime.port })
    if ($owner.Count -ne 1 -or ($owner[0].OwningProcess -ne [int]$runtime.pid)) {
        $problems.Add("runtime.json port $($runtime.port) is not owned by pid $($runtime.pid)")
    }
}

foreach ($server in $servers) {
    if ($launchers.Count -gt 0 -and -not ($launchers | Where-Object { $_.ProcessId -eq $server.ParentProcessId })) {
        $problems.Add("server $($server.ProcessId) parent $($server.ParentProcessId) is not a live launcher")
    }
}

$health = $null
try { $health = Invoke-RestMethod -Uri "http://127.0.0.1:$Port/health" -TimeoutSec 5 }
catch { $problems.Add("health endpoint unreachable on $Port") }

function Tail([string]$path) {
    if (-not (Test-Path -LiteralPath $path)) { return @() }
    return @(Get-Content -LiteralPath $path -Tail $LogLines -ErrorAction SilentlyContinue)
}

$report = [ordered]@{
    data_root = $DataRoot
    health = if ($health) { [ordered]@{ status = $health.status; schema = $health.database.schema_version; pid = $health.pid } } else { $null }
    launchers = @($launchers | ForEach-Object { [ordered]@{ pid = $_.ProcessId; parent = $_.ParentProcessId } })
    servers = @($servers | ForEach-Object { [ordered]@{ pid = $_.ProcessId; parent = $_.ParentProcessId } })
    listeners = @($listeners | ForEach-Object { [ordered]@{ port = $_.LocalPort; pid = $_.OwningProcess } })
    app_windows = @($windows | ForEach-Object { [ordered]@{ pid = $_.Id; title = $_.MainWindowTitle } })
    runtime = $runtime
    problems = @($problems)
    logs = [ordered]@{
        launcher = Tail (Join-Path $DataRoot 'launcher.log')
        stdout = Tail (Join-Path $DataRoot 'server.stdout.log')
        stderr = Tail (Join-Path $DataRoot 'server.stderr.log')
    }
}

if ($Json) {
    $report | ConvertTo-Json -Depth 8
} else {
    "HiveMind diagnostic - $($DataRoot)"
    "health:       $(if ($health) { $health.status + ' / schema ' + $health.database.schema_version + ' / pid ' + $health.pid } else { 'UNREACHABLE' })"
    "launchers:    $($launchers.Count)  servers: $($servers.Count)  app windows: $($windows.Count)"
    "listeners:    $(($listeners | ForEach-Object { "$($_.LocalPort)->$($_.OwningProcess)" }) -join ', ')"
    "runtime:      $(if ($runtime) { "port $($runtime.port) pid $($runtime.pid)" } else { 'missing' })"
    if ($problems.Count -eq 0) { "ghosts:       none detected" }
    else { "ghosts:       $($problems.Count) problem(s)"; $problems | ForEach-Object { "  - $_" } }
    foreach ($name in @('launcher', 'stdout', 'stderr')) {
        "--- $name.log (last $LogLines) ---"
        $lines = @($report.logs[$name])
        if ($lines.Count -eq 0) { "  (empty or missing)" } else { $lines | ForEach-Object { "  $_" } }
    }
}

if ($problems.Count -gt 0) { exit 1 }
exit 0
