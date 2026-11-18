# Empaqueta la versión portable: Apunta.exe + marcador "portable" en un .zip.
# Uso: npm run release   (o ejecutar este script tras "npm run tauri build")
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root "src-tauri\target\release\apunta.exe"
if (-not (Test-Path $exe)) { throw "No existe $exe. Ejecuta antes: npm run tauri build" }

$version = (Get-Content (Join-Path $root "package.json") -Raw | ConvertFrom-Json).version
$out = Join-Path $root "release"
New-Item -ItemType Directory -Force $out | Out-Null
# Carpeta temporal para no chocar con una copia portable que esté abierta en release/
$stage = Join-Path $env:TEMP "apunta-portable-stage"
if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
New-Item -ItemType Directory -Force $stage | Out-Null

Copy-Item $exe (Join-Path $stage "Apunta.exe")
Set-Content -Path (Join-Path $stage "portable") -Value "Este archivo activa el modo portable: los datos se guardan en la carpeta 'data' junto a Apunta.exe." -Encoding utf8

$zip = Join-Path $out "Apunta_${version}_x64_portable.zip"
if (Test-Path $zip) { Remove-Item -Force $zip }
Compress-Archive -Path (Join-Path $stage "*") -DestinationPath $zip

$nsis = Get-ChildItem (Join-Path $root "src-tauri\target\release\bundle\nsis\Apunta_${version}_*-setup.exe") -ErrorAction SilentlyContinue | Select-Object -First 1
if ($nsis) { Copy-Item $nsis.FullName $out -Force }

Write-Host "Listo:" -ForegroundColor Green
Get-ChildItem $out -File | ForEach-Object { "  {0}  ({1:N1} MB)" -f $_.Name, ($_.Length / 1MB) }
