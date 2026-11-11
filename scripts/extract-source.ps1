param(
  [string]$Archive = "source/arcange-crossdiag-v1.0.0-rc.2.zip",
  [string]$Destination = "app"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$archivePath = Join-Path $root $Archive
$destinationPath = Join-Path $root $Destination

if (-not (Test-Path -LiteralPath $archivePath -PathType Leaf)) {
  throw "Source archive not found: $archivePath"
}

if (Test-Path -LiteralPath $destinationPath) {
  throw "Destination already exists: $destinationPath. Rename or remove it before extracting."
}

New-Item -ItemType Directory -Path $destinationPath | Out-Null
try {
  Expand-Archive -LiteralPath $archivePath -DestinationPath $destinationPath -Force
  $project = Get-ChildItem -LiteralPath $destinationPath -Directory |
    Where-Object { Test-Path (Join-Path $_.FullName "package.json") } |
    Select-Object -First 1
  if (-not $project) {
    throw "Archive did not contain a project root with package.json."
  }
  Write-Host "CrossDiag source extracted to: $($project.FullName)"
  Write-Host "Next: Set-Location '$($project.FullName)'; npm install; npm run tauri dev"
}
catch {
  Remove-Item -LiteralPath $destinationPath -Recurse -Force -ErrorAction SilentlyContinue
  throw
}
