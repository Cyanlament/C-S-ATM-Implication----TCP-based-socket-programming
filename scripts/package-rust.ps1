$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$rustRoot = Join-Path $repoRoot "rust-atm"
$packagesRoot = Join-Path $repoRoot "packages"
$stageDir = Join-Path $packagesRoot "rust-atm-portable"
$zipPath = Join-Path $packagesRoot "rust-atm-portable-win64.zip"
$lanHost = "172.19.153.48"
$defaultPort = "2525"

$cargoCommand = Get-Command cargo.exe -ErrorAction SilentlyContinue
$cargoPath = if ($cargoCommand) { $cargoCommand.Source } else { $null }
if (-not $cargoPath) {
  $userCargo = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
  if (Test-Path $userCargo) {
    $cargoPath = $userCargo
  }
}
if (-not $cargoPath) {
  throw "cargo.exe not found. Install Rust from https://www.rust-lang.org/tools/install or add cargo.exe to PATH."
}

Write-Host "[Rust pack] Build release binaries..."
Push-Location $rustRoot
& $cargoPath build --release --bin server --bin client --bin test_case
Pop-Location

if (Test-Path $stageDir) {
  Remove-Item -Recurse -Force $stageDir
}
if (-not (Test-Path $packagesRoot)) {
  New-Item -ItemType Directory -Path $packagesRoot | Out-Null
}

New-Item -ItemType Directory -Path $stageDir | Out-Null
New-Item -ItemType Directory -Path (Join-Path $stageDir "logs") | Out-Null

Copy-Item (Join-Path $rustRoot "target\release\server.exe") (Join-Path $stageDir "server.exe") -Force
Copy-Item (Join-Path $rustRoot "target\release\client.exe") (Join-Path $stageDir "client.exe") -Force
Copy-Item (Join-Path $rustRoot "target\release\test_case.exe") (Join-Path $stageDir "test_case.exe") -Force
Copy-Item (Join-Path $rustRoot "users.txt") (Join-Path $stageDir "users.txt") -Force
Copy-Item (Join-Path $rustRoot "balances.txt") (Join-Path $stageDir "balances.txt") -Force

$startServer = @"
@echo off
chcp 65001>nul
cd /d %~dp0
echo [Rust ATM] Starting server...
if "%~1"=="" (
  set PORT_HINT=$defaultPort
) else (
  set PORT_HINT=%~1
)
echo Listening on 0.0.0.0:%PORT_HINT%
echo LAN client host: $lanHost
echo If Windows Firewall asks, allow private network access.
server.exe %*
pause
"@
Set-Content -Path (Join-Path $stageDir "start-server.bat") -Value $startServer -Encoding UTF8

$startClient = @"
@echo off
chcp 65001>nul
cd /d %~dp0
echo [Rust ATM] Starting GUI client...
if "%~1"=="" (
  client.exe $lanHost $defaultPort
) else (
  client.exe %*
)
pause
"@
Set-Content -Path (Join-Path $stageDir "start-client.bat") -Value $startClient -Encoding UTF8

$runTest = @"
@echo off
chcp 65001>nul
cd /d %~dp0
echo [Rust ATM] Running automated test cases...
if "%~1"=="" (
  test_case.exe $lanHost $defaultPort 100001 1234 100
) else (
  test_case.exe %*
)
pause
"@
Set-Content -Path (Join-Path $stageDir "run-test-case.bat") -Value $runTest -Encoding UTF8

$readme = @"
# Rust ATM Portable Package

## Runtime Files

- start-server.bat: start the bank server on all local network interfaces.
- start-client.bat: start the ATM GUI client. With no arguments, it connects to ${lanHost}:$defaultPort.
- run-test-case.bat: run automated protocol tests.
- users.txt: card number and PIN data.
- balances.txt: account balance data.
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

Write-Host "[Rust pack] Done: $zipPath"
