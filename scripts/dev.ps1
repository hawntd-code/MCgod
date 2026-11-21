# Abre MCgod en modo desarrollo (interfaz con recarga en vivo + motor en Rust).
# Uso: powershell -ExecutionPolicy Bypass -File .\scripts\dev.ps1

$ErrorActionPreference = 'Stop'
# Recarga el PATH por si Rust, Node o Java se instalaron después de abrir esta terminal.
$env:Path = [Environment]::GetEnvironmentVariable('Path', 'Machine') + ';' + [Environment]::GetEnvironmentVariable('Path', 'User')

$app = Join-Path $PSScriptRoot '..\app'
Push-Location $app
try {
    if (-not (Test-Path 'node_modules')) { npm install }
    cargo tauri dev
} finally {
    Pop-Location
}
