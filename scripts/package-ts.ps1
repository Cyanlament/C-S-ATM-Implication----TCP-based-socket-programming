$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$tsRoot = Join-Path $repoRoot "ts-atm"
$packagesRoot = Join-Path $repoRoot "packages"
$stageDir = Join-Path $packagesRoot "ts-atm-portable"
$zipPath = Join-Path $packagesRoot "ts-atm-portable-win64.zip"
$lanHost = "172.19.153.48"
$defaultPort = "2525"

$npmCommand = Get-Command npm.cmd -ErrorAction SilentlyContinue
if (-not $npmCommand) {
  throw "npm.cmd not found. Install Node.js or add npm.cmd to PATH."
}
$npmPath = $npmCommand.Source

Write-Host "[TS pack] Build project first..."
Push-Location $tsRoot
& $npmPath run build
Pop-Location

$electronRuntime = Join-Path $tsRoot "node_modules\electron\dist"
if (-not (Test-Path $electronRuntime)) {
  throw "Electron runtime not found. Run npm install in ts-atm first."
}

$nodeExe = (Get-Command node.exe -ErrorAction Stop).Source
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
Copy-Item (Join-Path $tsRoot "users.txt") (Join-Path $stageDir "users.txt") -Force
Copy-Item (Join-Path $tsRoot "balances.txt") (Join-Path $stageDir "balances.txt") -Force
Copy-Item (Join-Path $electronRuntime "*") (Join-Path $stageDir "runtime\electron") -Recurse -Force
Copy-Item (Join-Path $nodeDir "*") (Join-Path $stageDir "runtime\node") -Recurse -Force

$startServer = @"
@echo off
chcp 65001>nul
cd /d %~dp0
echo [TS ATM] Starting server...
if "%~1"=="" (
  set PORT_HINT=$defaultPort
) else (
  set PORT_HINT=%~1
)
echo Listening on 0.0.0.0:%PORT_HINT%
echo LAN client host: $lanHost
echo If Windows Firewall asks, allow private network access.
runtime\node\node.exe dist\src\server.js %*
pause
"@
Set-Content -Path (Join-Path $stageDir "start-server.bat") -Value $startServer -Encoding UTF8

$startClient = @"
@echo off
chcp 65001>nul
cd /d %~dp0
echo [TS ATM] Starting GUI client...
if "%~1"=="" (
  runtime\electron\electron.exe dist\src\client\main.js $lanHost $defaultPort
) else (
  runtime\electron\electron.exe dist\src\client\main.js %*
)
pause
"@
Set-Content -Path (Join-Path $stageDir "start-client.bat") -Value $startClient -Encoding UTF8

$runTest = @"
@echo off
chcp 65001>nul
cd /d %~dp0
echo [TS ATM] Running automated test cases...
if "%~1"=="" (
  runtime\node\node.exe dist\scripts\test_case.js $lanHost $defaultPort 100001 1234 100
) else (
  runtime\node\node.exe dist\scripts\test_case.js %*
)
pause
"@
Set-Content -Path (Join-Path $stageDir "run-test-case.bat") -Value $runTest -Encoding UTF8

$readme = @"
# TypeScript ATM Portable Package

## Runtime Files

- start-server.bat: start the bank server on all local network interfaces.
- start-client.bat: start the ATM GUI client. With no arguments, it connects to ${lanHost}:$defaultPort.
- run-test-case.bat: run automated protocol tests.
- users.txt: card number and PIN data.
- balances.txt: account balance data.
- runtime: bundled Node.js and Electron runtime.
- logs: runtime logs.

## LAN Test

The current server WLAN IPv4 address is $lanHost.
Use this address on another computer in the same Wi-Fi/LAN.

1. Run start-server.bat on the server computer.
2. Allow Windows Defender Firewall private-network access if prompted.
3. Run start-client.bat on the ATM/client computer.
4. Use user 100001, PIN 1234, amount 100 for the demo flow.

Command-line overrides:

start-server.bat [port]
start-client.bat [host] [port]
run-test-case.bat [host] [port] [user] [pin] [amount]
"@
Set-Content -Path (Join-Path $stageDir "README.txt") -Value $readme -Encoding UTF8

if (Test-Path $zipPath) {
  Remove-Item -Force $zipPath
}
Compress-Archive -Path (Join-Path $stageDir "*") -DestinationPath $zipPath

Write-Host "[TS pack] Done: $zipPath"
