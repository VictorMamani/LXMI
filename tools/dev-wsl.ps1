# Runs the real LXMI desktop app from Windows: Vite on Windows, Tauri in WSL.
# Needs WSL with mirrored networking (shared localhost), Rust and the Tauri
# system packages in the default distro. See README «Desarrollo en Windows».
#
#   powershell -ExecutionPolicy Bypass -File tools\dev-wsl.ps1

$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
$desktop = Join-Path $repo "apps\lxmi-desktop"
$devUrl = "http://127.0.0.1:1420"

function Test-Vite {
  try { (Invoke-WebRequest $devUrl -UseBasicParsing -TimeoutSec 2).StatusCode -eq 200 } catch { $false }
}

# Reuse a Vite already running; otherwise start one and stop it on exit.
$vite = $null
if (-not (Test-Vite)) {
  Write-Host "Iniciando Vite en $devUrl..."
  $vite = Start-Process npm.cmd -ArgumentList "run", "dev" -WorkingDirectory $desktop -PassThru -WindowStyle Minimized
  for ($i = 0; $i -lt 60 -and -not (Test-Vite); $i++) { Start-Sleep -Milliseconds 500 }
  if (-not (Test-Vite)) { throw "Vite no respondió en $devUrl" }
}

$wslRepo = (wsl -e wslpath -a ($repo -replace '\\', '/')).Trim()
# Build output stays on the Linux filesystem: much faster than /mnt.
$command = @"
set -e
cd '$wslRepo'
export CARGO_TARGET_DIR=`$HOME/.cache/lxmi-target CARGO_BUILD_JOBS=6
~/.cargo/bin/cargo run -p lxmi-desktop
"@

try {
  wsl -e bash -lc $command
} finally {
  if ($vite) { taskkill /PID $vite.Id /T /F | Out-Null }
}
