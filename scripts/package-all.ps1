$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot

Write-Host "=== 开始双版本打包 ==="
& (Join-Path $PSScriptRoot "package-rust.ps1")
& (Join-Path $PSScriptRoot "package-ts.ps1")
Write-Host "=== 全部打包完成，去 packages 目录拿压缩包吧 ==="
