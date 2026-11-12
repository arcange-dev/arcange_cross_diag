[CmdletBinding()]
param(
    [string]$OutputDirectory = ""
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
    $OutputDirectory = Join-Path $projectRoot "test-results"
}
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$reportPath = Join-Path $OutputDirectory "first-pc-test-$stamp.txt"
$lines = New-Object System.Collections.Generic.List[string]
$failures = 0
$warnings = 0

function Add-Result {
    param([string]$Status, [string]$Name, [string]$Detail)
    $line = "[{0}] {1}: {2}" -f $Status, $Name, $Detail
    $script:lines.Add($line)
    Write-Host $line
    if ($Status -eq "FAIL") { $script:failures++ }
    if ($Status -eq "WARN") { $script:warnings++ }
}

function Get-CommandVersion {
    param([string]$Name, [string[]]$Arguments = @("--version"))
    $command = Get-Command $Name -ErrorAction SilentlyContinue | Select-Object -First 1
    if (-not $command) { return $null }
    try {
        $output = & $command.Source @Arguments 2>&1 | Select-Object -First 1
        return [string]$output
    } catch {
        return "Found at $($command.Source); version query failed"
    }
}

$lines.Add("Arcange CrossDiag - First PC Test")
$lines.Add("Date: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss K')")
$lines.Add("Mode: read-only preflight; do not run as Administrator")
$lines.Add("Project: $projectRoot")
$lines.Add("")
Write-Host "`nArcange CrossDiag - First PC Test"
Write-Host "Read-only preflight. Run from the extracted project folder; Administrator is not required.`n"

# Verify that this is the expected project folder and that release versions agree.
$requiredFiles = @("package.json", "src-tauri/Cargo.toml", "src-tauri/tauri.conf.json", "src/App.tsx", "src-tauri/src/lib.rs")
$missingFiles = @($requiredFiles | Where-Object { -not (Test-Path (Join-Path $projectRoot $_)) })
if ($missingFiles.Count -eq 0) {
    Add-Result "PASS" "Project files" "All core frontend and Tauri source files are present."
} else {
    Add-Result "FAIL" "Project files" ("Missing: " + ($missingFiles -join ", "))
}

try {
    $package = Get-Content (Join-Path $projectRoot "package.json") -Raw | ConvertFrom-Json
    $tauri = Get-Content (Join-Path $projectRoot "src-tauri/tauri.conf.json") -Raw | ConvertFrom-Json
    $cargoText = Get-Content (Join-Path $projectRoot "src-tauri/Cargo.toml") -Raw
    $cargoVersion = [regex]::Match($cargoText, '(?m)^version\s*=\s*"([^"]+)"').Groups[1].Value
    if ($package.version -eq $tauri.version -and $package.version -eq $cargoVersion) {
        Add-Result "PASS" "Release version" "Package, Tauri and Cargo all report $($package.version)."
    } else {
        Add-Result "FAIL" "Release version" "package=$($package.version); tauri=$($tauri.version); cargo=$cargoVersion"
    }
} catch {
    Add-Result "FAIL" "Release configuration" $_.Exception.Message
}

# Verify developer toolchains. Version commands are the only external commands run.
$nodeVersion = Get-CommandVersion "node"
if ($nodeVersion -and $nodeVersion -match 'v?(\d+)\.') {
    if ([int]$Matches[1] -ge 22) { Add-Result "PASS" "Node.js" $nodeVersion }
    else { Add-Result "FAIL" "Node.js" "$nodeVersion; version 22 or newer is required." }
} else {
    Add-Result "FAIL" "Node.js" "Not found. Install Node.js 22 or newer and reopen PowerShell."
}

$npmVersion = Get-CommandVersion "npm.cmd" @("--version")
if ($npmVersion -and $npmVersion -match '(\d+)\.') {
    if ([int]$Matches[1] -ge 10) { Add-Result "PASS" "npm" $npmVersion }
    else { Add-Result "FAIL" "npm" "$npmVersion; npm 10 or newer is required." }
} else {
    Add-Result "FAIL" "npm" "Not found or version could not be read. Reinstall Node.js with npm included, then reopen PowerShell."
}

$rustVersion = Get-CommandVersion "rustc"
if ($rustVersion -and $rustVersion -match 'rustc\s+(\d+)\.(\d+)') {
    $rustMajor = [int]$Matches[1]
    $rustMinor = [int]$Matches[2]
    if ($rustMajor -gt 1 -or ($rustMajor -eq 1 -and $rustMinor -ge 90)) { Add-Result "PASS" "Rust" $rustVersion }
    else { Add-Result "FAIL" "Rust" "$rustVersion; Rust 1.90 or newer is required." }
} elseif ($rustVersion) {
    Add-Result "WARN" "Rust" "Found but could not parse version: $rustVersion"
} else {
    Add-Result "FAIL" "Rust" "Not found. Install the stable MSVC Rust toolchain from https://rustup.rs/."
}

$cargoVersionOutput = Get-CommandVersion "cargo"
if ($cargoVersionOutput) { Add-Result "PASS" "Cargo" $cargoVersionOutput }
else { Add-Result "FAIL" "Cargo" "Not found. Reopen PowerShell after installing Rust via rustup." }

if (Test-Path (Join-Path $projectRoot "node_modules")) {
    Add-Result "PASS" "Frontend dependencies" "node_modules folder exists."
} else {
    Add-Result "WARN" "Frontend dependencies" "Not installed yet. Run npm install from the project root."
}

# Read-only Windows inventory baseline. No serial numbers, MAC addresses or usernames are collected.
try {
    $os = Get-CimInstance Win32_OperatingSystem
    Add-Result "PASS" "Windows" "$($os.Caption); version $($os.Version); build $($os.BuildNumber); $($os.OSArchitecture)."
} catch {
    Add-Result "FAIL" "Windows inventory" "Could not query operating-system information: $($_.Exception.Message)"
}

try {
    $computer = Get-CimInstance Win32_ComputerSystem
    Add-Result "PASS" "PC model" "$($computer.Manufacturer) $($computer.Model)"
    Add-Result "PASS" "Installed RAM" ("{0:N2} GB" -f ($computer.TotalPhysicalMemory / 1GB))
} catch {
    Add-Result "WARN" "PC model and RAM" "System information unavailable: $($_.Exception.Message)"
}

try {
    $processors = @(Get-CimInstance Win32_Processor)
    $cpuNames = @($processors | ForEach-Object { $_.Name } | Select-Object -Unique)
    $logical = ($processors | Measure-Object -Property NumberOfLogicalProcessors -Sum).Sum
    $physical = ($processors | Measure-Object -Property NumberOfCores -Sum).Sum
    Add-Result "PASS" "Processor" ((($cpuNames -join "; ") + "; $physical physical cores; $logical logical processors"))
} catch {
    Add-Result "WARN" "Processor inventory" $_.Exception.Message
}

try {
    $disks = @(Get-CimInstance Win32_LogicalDisk -Filter "DriveType=3")
    if ($disks.Count -gt 0) {
        foreach ($disk in $disks) {
            $total = if ($disk.Size) { "{0:N1} GB" -f ($disk.Size / 1GB) } else { "unknown size" }
            $free = if ($disk.FreeSpace) { "{0:N1} GB free" -f ($disk.FreeSpace / 1GB) } else { "free space unavailable" }
            Add-Result "PASS" "Volume $($disk.DeviceID)" "$total; $free; filesystem $($disk.FileSystem)"
        }
    } else {
        Add-Result "WARN" "Local volumes" "No fixed local volumes were returned by Windows inventory."
    }
} catch {
    Add-Result "WARN" "Local volumes" $_.Exception.Message
}

$tools = @(
    @{ Name = "smartctl"; Purpose = "SMART storage diagnostics" },
    @{ Name = "nvidia-smi"; Purpose = "NVIDIA GPU telemetry" },
    @{ Name = "amd-smi"; Purpose = "AMD GPU telemetry" },
    @{ Name = "intel_gpu_top"; Purpose = "Intel Linux GPU telemetry; usually not installed on Windows" }
)
foreach ($tool in $tools) {
    $found = Get-Command $tool.Name -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($found) { Add-Result "PASS" "Optional tool $($tool.Name)" "$($tool.Purpose); found at $($found.Source)." }
    else { Add-Result "WARN" "Optional tool $($tool.Name)" "Not found; $($tool.Purpose) may be unavailable." }
}

try {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object -TypeName System.Security.Principal.WindowsPrincipal -ArgumentList $identity
    $isAdmin = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if ($isAdmin) { Add-Result "WARN" "PowerShell privilege" "Running elevated. Close this window and repeat as a normal user for the first test." }
    else { Add-Result "PASS" "PowerShell privilege" "Running as a standard user; recommended for the first read-only test." }
} catch {
    Add-Result "WARN" "PowerShell privilege" "Could not determine current privilege level."
}

$lines.Add("")
$lines.Add("Summary: $($script:failures) failure(s), $($script:warnings) warning(s).")
$lines.Add("This script did not run SMART scans, stress tests, repairs, updates, driver changes, firmware operations or disk writes.")
$lines.Add("No serial numbers, MAC addresses, usernames, passwords or recovery keys were collected.")
$lines | Set-Content -Path $reportPath -Encoding UTF8
Write-Host "`nSummary: $($script:failures) failure(s), $($script:warnings) warning(s)."
Write-Host "Report saved to: $reportPath"
if ($script:failures -gt 0) { exit 1 }
exit 0
