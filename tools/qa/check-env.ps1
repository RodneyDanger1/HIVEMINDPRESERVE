# QA environment evidence (read-only). Prints facts; writes nothing.
# Run: powershell -NoProfile -ExecutionPolicy Bypass -File tools/qa/check-env.ps1
$ErrorActionPreference = 'SilentlyContinue'
$root = Resolve-Path (Join-Path $PSScriptRoot '..\..')
Write-Output "root=$root"
Write-Output "timestamp=$((Get-Date).ToString('o'))"

# Node / npm / Playwright
Write-Output "node=$(cmd /c 'node --version')"
Write-Output "npm=$(cmd /c 'npm --version')"
Write-Output "npx_playwright=$(cmd /c 'npx --yes playwright --version')"
Write-Output "npm_global_playwright=$(cmd /c 'npm ls -g playwright --depth=0 2>nul')"

# Installed browser channels (used instead of any download)
$chrome = 'C:\Program Files\Google\Chrome\Application\chrome.exe'
$edge = 'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
foreach ($pair in @(@('chrome', $chrome), @('edge', $edge))) {
  $name = $pair[0]; $path = $pair[1]
  if (Test-Path -LiteralPath $path) {
    $v = (Get-Item -LiteralPath $path).VersionInfo.ProductVersion
    Write-Output "$name=$path version=$v"
  } else {
    Write-Output "$name=ABSENT path=$path"
  }
}

# Project state
Write-Output "web_src_files=$(@(Get-ChildItem -LiteralPath (Join-Path $root 'web\src') -Recurse -File -ErrorAction SilentlyContinue).Count)"
Write-Output "web_package_json=$(Test-Path -LiteralPath (Join-Path $root 'web\package.json'))"
Write-Output "root_package_json=$(Test-Path -LiteralPath (Join-Path $root 'package.json'))"
Write-Output "root_node_modules=$(Test-Path -LiteralPath (Join-Path $root 'node_modules'))"
Write-Output "e2e_node_modules=$(Test-Path -LiteralPath (Join-Path $root 'e2e\node_modules'))"
Write-Output "backend_crates=$((Get-ChildItem -LiteralPath (Join-Path $root 'backend\crates') -Directory).Name -join ',')"
Write-Output "git_head=$(& git -C $root rev-parse --short HEAD)"
Write-Output "disk_free_bytes=$((Get-PSDrive C).Free)"

# Blueprint / contract pins (must match EXECUTION-PROMPT.md §0)
$pins = @{
  'blueprint\HIVEMIND-BLUEPRINT-v1.md' = '4921ABE45746D5932FE5EF681D86C0D8F6217E9607F89D6AEBA9F8E9BBBD716B'
  'blueprint\UI-WIRING-GUIDE.md' = '3911C3E5DED6ED4728E00811126E1639C10BDEA281587997837A302A5F1E8104'
  'blueprint\EXTERNAL-DEPENDENCIES.md' = '0C933FBFDD84B68AF89CCD34C2FC27DFC3BC3878A23CD50551870235FBF10308'
}
$blueprintRoot = Join-Path (Split-Path -Parent $root) 'hivemind-blueprint'
foreach ($rel in $pins.Keys) {
  $path = Join-Path $blueprintRoot $rel
  if (Test-Path -LiteralPath $path) {
    $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash
    Write-Output "pin=$rel match=$($hash -eq $pins[$rel]) sha256=$hash"
  } else {
    Write-Output "pin=$rel MISSING"
  }
}

# FigCraft assets + expected manifest / design file
$assets = Join-Path $root 'docs\ui\figcraft\ASSETS'
if (Test-Path -LiteralPath $assets) {
  Get-ChildItem -LiteralPath $assets -File | Sort-Object Name | ForEach-Object {
    Write-Output "figcraft_asset=$($_.Name) bytes=$($_.Length) sha256=$((Get-FileHash -Algorithm SHA256 -LiteralPath $_.FullName).Hash)"
  }
}
Write-Output "figcraft_assets_index_present=$(Test-Path -LiteralPath (Join-Path $root 'docs\ui\figcraft\ASSETS-INDEX.json'))"
Write-Output "figcraft_a_design_present=$(Test-Path -LiteralPath (Join-Path $root 'docs\ui\figcraft\A-design.figcraft'))"
$cli = 'C:\Program Files\FigCraft\figcraft-cli.exe'
Write-Output "figcraft_cli_present=$(Test-Path -LiteralPath $cli) path=$cli"

# Trace/audit state
Write-Output "docs_trace_files=$((Get-ChildItem -LiteralPath (Join-Path $root 'docs\trace') -File -ErrorAction SilentlyContinue).Name -join ',')"
Write-Output "docs_audit_files=$((Get-ChildItem -LiteralPath (Join-Path $root 'docs\audit') -File -ErrorAction SilentlyContinue).Name -join ',')"
