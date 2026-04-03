$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$rustRoot = Join-Path $repoRoot "rust-atm"
$packagesRoot = Join-Path $repoRoot "packages"
$stageDir = Join-Path $packagesRoot "rust-atm-portable"
$zipPath = Join-Path $packagesRoot "rust-atm-portable-win64.zip"

Write-Host "[Rust pack] Build release binaries..."
Push-Location $rustRoot
cargo build --release --bin server --bin client --bin test_case
Pop-Location

if (Test-Path $stageDir) {
  Remove-Item -Recurse -Force $stageDir
}
if (-not (Test-Path $packagesRoot)) {
  New-Item -ItemType Directory -Path $packagesRoot | Out-Null
}

New-Item -ItemType Directory -Path $stageDir | Out-Null
New-Item -ItemType Directory -Path (Join-Path $stageDir "data") | Out-Null
New-Item -ItemType Directory -Path (Join-Path $stageDir "logs") | Out-Null

Copy-Item (Join-Path $rustRoot "target\release\server.exe") (Join-Path $stageDir "server.exe") -Force
Copy-Item (Join-Path $rustRoot "target\release\client.exe") (Join-Path $stageDir "client.exe") -Force
Copy-Item (Join-Path $rustRoot "target\release\test_case.exe") (Join-Path $stageDir "test_case.exe") -Force
Copy-Item (Join-Path $rustRoot "data\accounts.json") (Join-Path $stageDir "data\accounts.json") -Force

$startServer = @"
@echo off
chcp 65001>nul
cd /d %~dp0
echo [Rust ATM] Starting server...
server.exe
pause
"@
Set-Content -Path (Join-Path $stageDir "start-server.bat") -Value $startServer -Encoding UTF8

$startClient = @"
@echo off
chcp 65001>nul
cd /d %~dp0
echo [Rust ATM] Starting GUI client...
client.exe
pause
"@
Set-Content -Path (Join-Path $stageDir "start-client.bat") -Value $startClient -Encoding UTF8

$runTest = @"
@echo off
chcp 65001>nul
cd /d %~dp0
echo [Rust ATM] Running automated test cases...
test_case.exe %*
pause
"@
Set-Content -Path (Join-Path $stageDir "run-test-case.bat") -Value $runTest -Encoding UTF8

$readme = @"
# Rust ATM Portable Package

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
- If port 2525 is busy, stop the other process first.
"@
Set-Content -Path (Join-Path $stageDir "README.txt") -Value $readme -Encoding UTF8

if (Test-Path $zipPath) {
  Remove-Item -Force $zipPath
}
Compress-Archive -Path (Join-Path $stageDir "*") -DestinationPath $zipPath

Write-Host "[Rust pack] Done: $zipPath"
