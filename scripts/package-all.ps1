$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$packagesRoot = Join-Path $repoRoot "packages"

Write-Host "=== Clean old portable packages ==="
if (Test-Path $packagesRoot) {
  Remove-Item -Recurse -Force $packagesRoot
}

Write-Host "=== Build portable packages ==="
& (Join-Path $PSScriptRoot "package-rust.ps1")
& (Join-Path $PSScriptRoot "package-ts.ps1")
Write-Host "=== Done. Packages are in the packages directory. ==="
