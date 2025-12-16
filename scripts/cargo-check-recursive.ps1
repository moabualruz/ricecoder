param (
    [string]$StartDirectory = "."
)

Set-Location "$PSScriptRoot\.."

$ErrorActionPreference = "Stop"

# Resolve and validate start directory
$Root = Resolve-Path $StartDirectory -ErrorAction Stop

# Ensure cargo exists
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "cargo executable not found in PATH."
    exit 1
}

Write-Host "Starting Cargo check from: $Root"

# Find all Cargo.toml files under the start directory
$cargoTomls = Get-ChildItem -Path $Root -Recurse -Filter Cargo.toml -File

foreach ($cargoToml in $cargoTomls) {
    $crateDir = $cargoToml.Directory.FullName

    Write-Host "Checking crate: $crateDir"

    Push-Location $crateDir
    try {
        cargo check
    }
    finally {
        Pop-Location
    }
}
