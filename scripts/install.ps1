# RiceCoder Installation Script for Windows (PowerShell)
#
# This script compiles and installs ricecoder from source on Windows.
# Supports: Windows 10/11 (x86_64, aarch64)
#
# Usage:
#   .\scripts\install.ps1 [OPTIONS]
#
# Can also be called via curl (PowerShell):
#   iex (New-Object Net.WebClient).DownloadString('https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install.ps1')
#
# Options:
#   -Prefix PATH       Installation prefix (default: $env:LOCALAPPDATA\ricecoder)
#   -Release           Build in release mode (default)
#   -Debug             Build in debug mode
#   -Help              Show this help message
#
# Examples:
#   .\scripts\install.ps1
#   .\scripts\install.ps1 -Prefix "C:\Program Files\ricecoder"
#   .\scripts\install.ps1 -Debug
#
# Requirements:
#   - PowerShell 5.0+
#   - Rust (https://rustup.rs/)
#   - Git (https://git-scm.com/)
#   - Visual Studio Build Tools or MSVC compiler
#

param(
    [string]$Prefix = "$env:LOCALAPPDATA\ricecoder",
    [ValidateSet("release", "debug")]
    [string]$BuildMode = "release",
    [switch]$Verbose = $false,
    [switch]$Help = $false
)

$ErrorActionPreference = "Stop"

# Colors for output
$Colors = @{
    Red     = "`e[0;31m"
    Green   = "`e[0;32m"
    Yellow  = "`e[1;33m"
    Blue    = "`e[0;34m"
    Reset   = "`e[0m"
}

# Configuration
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = if ($ScriptDir) { Split-Path -Parent $ScriptDir } else { Get-Location }
$BinDir = Join-Path $Prefix "bin"
$ConfigDir = Join-Path $Prefix "etc\ricecoder"
$DocDir = Join-Path $Prefix "share\doc\ricecoder"

# Detect architecture
$Arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "aarch64" }

# Logging functions
function Write-Info {
    param([string]$Message)
    Write-Host "$($Colors.Blue)[INFO]$($Colors.Reset) $Message"
}

function Write-Success {
    param([string]$Message)
    Write-Host "$($Colors.Green)[OK]$($Colors.Reset) $Message"
}

function Write-ErrorMsg {
    param([string]$Message)
    Write-Host "$($Colors.Red)[ERROR]$($Colors.Reset) $Message" -ForegroundColor Red
}

function Write-Warning {
    param([string]$Message)
    Write-Host "$($Colors.Yellow)[WARN]$($Colors.Reset) $Message"
}

function Show-Help {
    Write-Host "RiceCoder Installation Script for Windows"
    Write-Host ""
    Write-Host "Usage:"
    Write-Host "  .\scripts\install.ps1 [OPTIONS]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Prefix PATH       Installation prefix"
    Write-Host "  -Release           Build in release mode (default)"
    Write-Host "  -Debug             Build in debug mode"
    Write-Host "  -Verbose           Show verbose output"
    Write-Host "  -Help              Show this help message"
    Write-Host ""
    Write-Host "Remote Installation:"
    Write-Host "  iex (New-Object Net.WebClient).DownloadString('https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install.ps1')"
}

# Check prerequisites
function Test-Prerequisites {
    Write-Info "Checking prerequisites..."
    
    if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
        Write-ErrorMsg "Rust is not installed"
        Write-Host "Install from: https://rustup.rs/"
        exit 1
    }
    
    $rustVersion = rustc --version
    Write-Success "Found: $rustVersion"
    
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-ErrorMsg "Cargo is not installed"
        exit 1
    }
    
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        Write-ErrorMsg "Git is not installed"
        exit 1
    }
    
    Write-Success "All prerequisites met"
}

# Update Rust toolchain
function Update-Rust {
    Write-Info "Updating Rust toolchain..."
    rustup update
    Write-Success "Rust toolchain updated"
}

# Build ricecoder
function Build-RiceCoder {
    Write-Info "Building ricecoder ($BuildMode mode)..."
    
    Push-Location $ProjectRoot
    
    try {
        $cargoArgs = @()
        
        if ($BuildMode -eq "release") {
            $cargoArgs += "--release"
        }
        
        if ($Verbose) {
            $cargoArgs += "--verbose"
        }
        
        & cargo build @cargoArgs
        
        if ($LASTEXITCODE -ne 0) {
            Write-ErrorMsg "Build failed with exit code $LASTEXITCODE"
            exit 1
        }
        
        Write-Success "Build completed successfully"
    }
    finally {
        Pop-Location
    }
}

# Install binaries
function Install-Binaries {
    Write-Info "Installing binaries to $BinDir..."
    
    if (-not (Test-Path $BinDir)) {
        New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
    }
    
    $buildDir = if ($BuildMode -eq "release") {
        Join-Path $ProjectRoot "target\release"
    }
    else {
        Join-Path $ProjectRoot "target\debug"
    }
    
    $binaries = Get-ChildItem -Path $buildDir -Filter "*.exe" -ErrorAction SilentlyContinue
    
    if ($binaries.Count -eq 0) {
        Write-Warning "No binaries found in $buildDir"
        return
    }
    
    foreach ($binary in $binaries) {
        Write-Info "Installing $($binary.Name)..."
        $destPath = Join-Path $BinDir $binary.Name
        Copy-Item -Path $binary.FullName -Destination $destPath -Force
        Write-Success "Installed $($binary.Name)"
    }
}

# Install configuration files
function Install-Config {
    Write-Info "Installing configuration files..."
    
    if (-not (Test-Path $ConfigDir)) {
        New-Item -ItemType Directory -Path $ConfigDir -Force | Out-Null
    }
    
    $configSource = Join-Path $ProjectRoot "config"
    if (Test-Path $configSource) {
        Copy-Item -Path "$configSource\*" -Destination $ConfigDir -Recurse -Force -ErrorAction SilentlyContinue
        Write-Success "Configuration files installed to $ConfigDir"
    }
}

# Install documentation
function Install-Docs {
    Write-Info "Installing documentation..."
    
    if (-not (Test-Path $DocDir)) {
        New-Item -ItemType Directory -Path $DocDir -Force | Out-Null
    }
    
    $readmePath = Join-Path $ProjectRoot "README.md"
    if (Test-Path $readmePath) {
        Copy-Item -Path $readmePath -Destination $DocDir -Force
    }
    
    $licensePath = Join-Path $ProjectRoot "LICENSE.md"
    if (Test-Path $licensePath) {
        Copy-Item -Path $licensePath -Destination $DocDir -Force
    }
    
    Write-Success "Documentation installed to $DocDir"
}

# Verify installation
function Test-Installation {
    Write-Info "Verifying installation..."
    
    if (-not (Test-Path $BinDir)) {
        Write-ErrorMsg "Installation directory not found: $BinDir"
        return $false
    }
    
    $binaries = Get-ChildItem -Path $BinDir -Filter "ricecoder*.exe" -ErrorAction SilentlyContinue
    
    if ($binaries.Count -eq 0) {
        Write-Warning "No ricecoder binaries found in $BinDir"
        return $false
    }
    
    foreach ($binary in $binaries) {
        Write-Success "Found: $($binary.Name)"
    }
    
    Write-Success "Installation verified ($($binaries.Count) binaries)"
    return $true
}

# Print summary
function Print-Summary {
    Write-Host ""
    Write-Host "Installation Details:"
    Write-Host "  OS:              Windows"
    Write-Host "  Architecture:    $Arch"
    Write-Host "  Build Mode:      $BuildMode"
    Write-Host "  Install Prefix:  $Prefix"
    Write-Host "  Binaries:        $BinDir"
    Write-Host "  Config:          $ConfigDir"
    Write-Host "  Documentation:   $DocDir"
    Write-Host ""
    Write-Host "Next Steps:"
    Write-Host "  1. Add to PATH: `$env:PATH += ';$BinDir'"
    Write-Host "  2. Verify:      ricecoder --version"
    Write-Host "  3. Get help:    ricecoder --help"
    Write-Host ""
}

# Main installation flow
function Main {
    if ($Help) {
        Show-Help
        exit 0
    }
    
    Write-Host "RiceCoder Installation Script"
    Write-Host ""
    
    Write-Info "System Information:"
    Write-Info "  OS:           Windows"
    Write-Info "  Architecture: $Arch"
    Write-Info "  Project Root: $ProjectRoot"
    Write-Host ""
    
    Test-Prerequisites
    Update-Rust
    Build-RiceCoder
    Install-Binaries
    Install-Config
    Install-Docs
    Test-Installation
    Print-Summary
}

# Run main function
Main
