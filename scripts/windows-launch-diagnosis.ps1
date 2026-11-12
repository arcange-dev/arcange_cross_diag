# Arcange CrossDiag - Windows launch diagnosis
#
# Read-only evidence collector for "the application does not open" reports.
# It changes no system setting, installs nothing, disables nothing and deletes nothing.
# Everything it finds is written to a single report file you can send back.
#
#   powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/windows-launch-diagnosis.ps1
#   powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/windows-launch-diagnosis.ps1 -Launch
#
# -Launch additionally starts `npm run start` (the native Tauri development application),
# records its output for -LaunchSeconds, checks whether a window process appears, and then
# stops only the processes it started itself.

[CmdletBinding()]
param(
  [switch]$Launch,
  [int]$LaunchSeconds = 75,
  [string]$ReportDirectory
)

$ErrorActionPreference = "Continue"
$repoRoot = Split-Path -Parent $PSScriptRoot
if (-not $ReportDirectory) { $ReportDirectory = Join-Path $repoRoot "reports" }
if (-not (Test-Path -LiteralPath $ReportDirectory)) {
  New-Item -ItemType Directory -Path $ReportDirectory -Force | Out-Null
}
$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$reportPath = Join-Path $ReportDirectory "windows-launch-diagnosis-$stamp.log"
$lines = New-Object System.Collections.Generic.List[string]

function Add-Line([string]$text = "") { $lines.Add($text); Write-Host $text }
function Add-Section([string]$title) { Add-Line ""; Add-Line ("=" * 68); Add-Line $title; Add-Line ("=" * 68) }
function Add-CommandInfo([string]$name, [string]$displayName) {
  $command = Get-Command $name -ErrorAction SilentlyContinue
  if (-not $command) { Add-Line ("{0,-22}: NOT FOUND" -f $displayName); return }
  $version = ""
  try { $version = (& $name --version 2>&1 | Select-Object -First 1) } catch { $version = "(version query failed)" }
  Add-Line ("{0,-22}: {1}" -f $displayName, $version)
  Add-Line ("{0,-22}: {1}" -f "  resolved", $command.Source)
}

Add-Line "Arcange CrossDiag - Windows launch diagnosis"
Add-Line "Generated: $(Get-Date -Format o)"
Add-Line "Report:    $reportPath"

Add-Section "1. Operating system and architecture"
try {
  $os = Get-CimInstance Win32_OperatingSystem
  Add-Line ("Caption             : {0}" -f $os.Caption)
  Add-Line ("Version             : {0}" -f $os.Version)
  Add-Line ("Build               : {0}" -f $os.BuildNumber)
  Add-Line ("OS architecture     : {0}" -f $os.OSArchitecture)
} catch { Add-Line "Operating system query failed: $($_.Exception.Message)" }
Add-Line ("Process architecture: {0}" -f $env:PROCESSOR_ARCHITECTURE)
Add-Line ("Is 64-bit OS        : {0}" -f [Environment]::Is64BitOperatingSystem)
Add-Line ("PowerShell          : {0}" -f $PSVersionTable.PSVersion)

Add-Section "2. Required development tools"
Add-CommandInfo "node"  "Node.js"
Add-CommandInfo "npm"   "npm"
Add-CommandInfo "rustc" "Rust compiler"
Add-CommandInfo "cargo" "Cargo"
if (Get-Command rustup -ErrorAction SilentlyContinue) {
  Add-Line ("rustup toolchain    : {0}" -f ((& rustup show active-toolchain 2>&1) -join " "))
  Add-Line ("rustup targets      : {0}" -f (((& rustup target list --installed 2>&1) -join ", ")))
}
$msvcTarget = (& rustc --print target-libdir 2>&1) -join " "
Add-Line ("rustc target dir    : {0}" -f $msvcTarget)
Add-Line ("C++ linker (link.exe): {0}" -f $(if (Get-Command link.exe -ErrorAction SilentlyContinue) { "found" } else { "not on PATH (normal outside a developer prompt)" }))
$vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path -LiteralPath $vswhere) {
  $vs = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>&1
  Add-Line ("Visual Studio C++   : {0}" -f $(if ($vs) { $vs } else { "MSVC C++ build tools NOT installed" }))
} else {
  Add-Line "Visual Studio C++   : vswhere not found (Build Tools may be absent)"
}

Add-Section "3. Microsoft WebView2 runtime"
$webViewKeys = @(
  "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
  "HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
  "HKCU:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
)
$webViewFound = $false
foreach ($key in $webViewKeys) {
  $item = Get-ItemProperty -Path $key -ErrorAction SilentlyContinue
  if ($item -and $item.pv) { Add-Line ("WebView2 runtime    : {0} ({1})" -f $item.pv, $key); $webViewFound = $true }
}
if (-not $webViewFound) { Add-Line "WebView2 runtime    : NOT DETECTED - the window cannot render without it" }
foreach ($dll in @("vcruntime140.dll", "vcruntime140_1.dll", "msvcp140.dll")) {
  $path = Join-Path $env:SystemRoot "System32\$dll"
  Add-Line ("{0,-20}: {1}" -f $dll, $(if (Test-Path -LiteralPath $path) { "present" } else { "MISSING" }))
}

Add-Section "4. Project state"
Add-Line ("Repository root     : {0}" -f $repoRoot)
Add-Line ("package.json        : {0}" -f $(if (Test-Path (Join-Path $repoRoot "package.json")) { "present" } else { "MISSING" }))
Add-Line ("Frontend installed  : {0}" -f $(if (Test-Path (Join-Path $repoRoot "node_modules")) { "node_modules present" } else { "node_modules MISSING - run npm install" }))
Add-Line ("Tauri CLI installed : {0}" -f $(if (Test-Path (Join-Path $repoRoot "node_modules\.bin\tauri.cmd")) { "yes (node_modules/.bin/tauri.cmd)" } else { "not installed" }))
Add-Line ("Rust sources        : {0}" -f $(if (Test-Path (Join-Path $repoRoot "src-tauri\src\lib.rs")) { "src-tauri/src/lib.rs present" } else { "MISSING" }))
$tauriConfigPath = Join-Path $repoRoot "src-tauri\tauri.conf.json"
if (Test-Path -LiteralPath $tauriConfigPath) {
  try {
    $config = Get-Content -LiteralPath $tauriConfigPath -Raw | ConvertFrom-Json
    Add-Line ("Tauri devUrl        : {0}" -f $config.build.devUrl)
    Add-Line ("Tauri frontendDist  : {0}" -f $config.build.frontendDist)
    Add-Line ("Tauri beforeDev     : {0}" -f $config.build.beforeDevCommand)
    Add-Line ("Tauri window label  : {0}" -f $(if ($config.app.windows[0].label) { $config.app.windows[0].label } else { "main (default)" }))
  } catch { Add-Line "tauri.conf.json could not be parsed: $($_.Exception.Message)" }
} else { Add-Line "tauri.conf.json MISSING" }
Add-Line ("Built executable    : {0}" -f $(if (Test-Path (Join-Path $repoRoot "src-tauri\target\release\Arcange CrossDiag.exe")) { "src-tauri/target/release/Arcange CrossDiag.exe" } else { "not built yet" }))
Add-Line ("Startup error log   : {0}" -f $(if (Test-Path (Join-Path $env:LOCALAPPDATA "Arcange CrossDiag\startup-error.log")) { Join-Path $env:LOCALAPPDATA "Arcange CrossDiag\startup-error.log" } else { "none recorded" }))

Add-Section "5. Recent Windows application errors (last 24 hours)"
try {
  $events = Get-WinEvent -FilterHashtable @{ LogName = "Application"; Level = 2; StartTime = (Get-Date).AddDays(-1) } -MaxEvents 15 -ErrorAction Stop
  foreach ($event in $events) {
    Add-Line ("{0} | {1} | {2}" -f $event.TimeCreated.ToString("s"), $event.ProviderName, (($event.Message -split "`r?`n")[0]))
  }
} catch { Add-Line "No application error events could be read (this is not an error by itself)." }

if ($Launch) {
  Add-Section "6. Native startup attempt (npm run start)"
  Add-Line ("Timeout             : {0} seconds" -f $LaunchSeconds)
  $stdoutPath = Join-Path $ReportDirectory "windows-launch-stdout-$stamp.log"
  $stderrPath = Join-Path $ReportDirectory "windows-launch-stderr-$stamp.log"
  $startedAt = Get-Date
  try {
    $process = Start-Process -FilePath "npm.cmd" -ArgumentList "run", "start" -WorkingDirectory $repoRoot `
      -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath -PassThru -WindowStyle Hidden
    Add-Line ("Launcher process id : {0}" -f $process.Id)
    $deadline = (Get-Date).AddSeconds($LaunchSeconds)
    $windowProcess = $null
    while ((Get-Date) -lt $deadline) {
      Start-Sleep -Seconds 3
      $windowProcess = Get-Process -Name "Arcange CrossDiag", "arcange-crossdiag" -ErrorAction SilentlyContinue | Select-Object -First 1
      if ($windowProcess) { break }
      if ($process.HasExited) { break }
    }
    if ($windowProcess) {
      Add-Line ("Native process      : RUNNING (pid {0}, main window title: '{1}')" -f $windowProcess.Id, $windowProcess.MainWindowTitle)
      Add-Line ("Window visible      : {0}" -f ($windowProcess.MainWindowHandle -ne 0))
    } else {
      Add-Line "Native process      : NOT RUNNING - the desktop application never reached a window"
    }
    Add-Line ("Launcher exit code  : {0}" -f $(if ($process.HasExited) { $process.ExitCode } else { "still running" }))
  } catch {
    Add-Line "Startup attempt failed: $($_.Exception.Message)"
  } finally {
    foreach ($name in @("Arcange CrossDiag", "arcange-crossdiag")) {
      Get-Process -Name $name -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    }
    Get-Process -Name "cargo", "node" -ErrorAction SilentlyContinue |
      Where-Object { $_.StartTime -gt $startedAt } |
      Stop-Process -Force -ErrorAction SilentlyContinue
  }
  foreach ($file in @($stdoutPath, $stderrPath)) {
    if (Test-Path -LiteralPath $file) {
      Add-Line ""
      Add-Line ("--- {0} (last 60 lines) ---" -f (Split-Path -Leaf $file))
      Get-Content -LiteralPath $file -Tail 60 | ForEach-Object { Add-Line $_ }
    }
  }
  Add-Line ""
  Add-Line "Note: 'npm run dev' only starts the Vite web server. It never opens a desktop window."
  Add-Line "      Use 'npm run start' (tauri dev) for the native application."
} else {
  Add-Section "6. Native startup attempt"
  Add-Line "Skipped. Re-run with -Launch to start the native application and capture its output."
}

$lines | Set-Content -LiteralPath $reportPath -Encoding UTF8
Write-Host ""
Write-Host "[OK] Diagnosis written to $reportPath"
