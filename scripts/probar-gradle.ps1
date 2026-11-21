# Ejecuta Gradle sobre el proyecto de ejemplo con el mismo código que usa la app.
# Uso:
#   powershell -ExecutionPolicy Bypass -File .\scripts\probar-gradle.ps1 build
#   powershell -ExecutionPolicy Bypass -File .\scripts\probar-gradle.ps1 runClient 120
param(
    [Parameter(Mandatory = $true)][string]$Tarea,
    [int]$DetenerA = 0,
    [string]$Proyecto = 'examples\rubymod'
)
$ErrorActionPreference = 'Stop'
# Recarga el PATH por si Java o Rust se instalaron después de abrir esta terminal.
$env:Path = [Environment]::GetEnvironmentVariable('Path', 'Machine') + ';' + [Environment]::GetEnvironmentVariable('Path', 'User')

$raiz = Resolve-Path (Join-Path $PSScriptRoot '..')
Push-Location $raiz
try {
    $args = @('run', '-q', '-p', 'mcgod-gradle', '--example', 'gradle', '--', $Proyecto, $Tarea)
    if ($DetenerA -gt 0) { $args += "$DetenerA" }
    cargo @args
} finally {
    Pop-Location
}
