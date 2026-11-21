# Genera el código del proyecto de ejemplo con el mismo código que usa la app.
# La primera vez le pide a Gradle el classpath del proyecto para armar el índice de imports.
# Uso:
#   powershell -ExecutionPolicy Bypass -File .\scripts\generar.ps1
#   powershell -ExecutionPolicy Bypass -File .\scripts\generar.ps1 -SoloMostrar
param(
    [string]$Proyecto = 'examples\rubymod',
    [switch]$SoloMostrar
)
$ErrorActionPreference = 'Stop'
# Recarga el PATH por si Java o Rust se instalaron después de abrir esta terminal.
$env:Path = [Environment]::GetEnvironmentVariable('Path', 'Machine') + ';' + [Environment]::GetEnvironmentVariable('Path', 'User')

$raiz = Resolve-Path (Join-Path $PSScriptRoot '..')
Push-Location $raiz
try {
    $argumentos = @('run', '-q', '-p', 'mcgod-generator', '--example', 'generate', '--', $Proyecto)
    if (-not $SoloMostrar) { $argumentos += '--write' }
    cargo @argumentos
} finally {
    Pop-Location
}
