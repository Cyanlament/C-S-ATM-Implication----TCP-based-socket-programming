$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$tsRoot = Join-Path $repoRoot "ts-atm"
$packagesRoot = Join-Path $repoRoot "packages"
$stageDir = Join-Path $packagesRoot "ts-atm-portable"
$zipPath = Join-Path $packagesRoot "ts-atm-portable-win64.zip"

Write-Host "[TS pack] Build project first..."
Push-Location $tsRoot
npm run build
Pop-Location

$electronRuntime = Join-Path $tsRoot "node_modules\electron\dist"
if (-not (Test-Path $electronRuntime)) {
  throw "Electron runtime not found. Run npm install in ts-atm first."
}

$nodeExe = (Get-Command node -ErrorAction Stop).Source
$nodeDir = Split-Path -Parent $nodeExe

if (Test-Path $stageDir) {
  Remove-Item -Recurse -Force $stageDir
}
if (-not (Test-Path $packagesRoot)) {
  New-Item -ItemType Directory -Path $packagesRoot | Out-Null
}

New-Item -ItemType Directory -Path $stageDir | Out-Null
New-Item -ItemType Directory -Path (Join-Path $stageDir "runtime") | Out-Null
New-Item -ItemType Directory -Path (Join-Path $stageDir "runtime\node") | Out-Null
New-Item -ItemType Directory -Path (Join-Path $stageDir "runtime\electron") | Out-Null
New-Item -ItemType Directory -Path (Join-Path $stageDir "src") | Out-Null
New-Item -ItemType Directory -Path (Join-Path $stageDir "src\\client") | Out-Null
New-Item -ItemType Directory -Path (Join-Path $stageDir "logs") | Out-Null

Copy-Item (Join-Path $tsRoot "dist") (Join-Path $stageDir "dist") -Recurse -Force
Copy-Item (Join-Path $tsRoot "src\client\renderer") (Join-Path $stageDir "src\client\renderer") -Recurse -Force
Copy-Item (Join-Path $tsRoot "data") (Join-Path $stageDir "data") -Recurse -Force
Copy-Item (Join-Path $electronRuntime "*") (Join-Path $stageDir "runtime\electron") -Recurse -Force
Copy-Item (Join-Path $nodeDir "*") (Join-Path $stageDir "runtime\node") -Recurse -Force

$startServer = @"
@echo off
chcp 65001>nul
cd /d %~dp0
echo [TS ATM] Starting server...
runtime\node\node.exe dist\src\server.js
pause
"@
Set-Content -Path (Join-Path $stageDir "start-server.bat") -Value $startServer -Encoding UTF8

$startClient = @"
@echo off
chcp 65001>nul
cd /d %~dp0
echo [TS ATM] Starting GUI client...
runtime\electron\electron.exe dist\src\client\main.js
pause
"@
Set-Content -Path (Join-Path $stageDir "start-client.bat") -Value $startClient -Encoding UTF8

$runTest = @"
@echo off
chcp 65001>nul
cd /d %~dp0
echo [TS ATM] Running automated test cases...
runtime\node\node.exe dist\scripts\test_case.js %*
pause
"@
Set-Content -Path (Join-Path $stageDir "run-test-case.bat") -Value $runTest -Encoding UTF8

$readme = @"
# TypeScript ATM Portable Package

Unzip and double-click these files:

1. start-server.bat
- Starts the server on port 2525.

2. start-client.bat
- Starts the GUI client.

3. run-test-case.bat
- Runs automated test cases.

Notes:
- Account data file: data/accounts.json
- Logs folder: logs
- Portable package includes Node and Electron runtime.
"@
Set-Content -Path (Join-Path $stageDir "README.txt") -Value $readme -Encoding UTF8

if (Test-Path $zipPath) {
  Remove-Item -Force $zipPath
}
Compress-Archive -Path (Join-Path $stageDir "*") -DestinationPath $zipPath

Write-Host "[TS pack] Done: $zipPath"
