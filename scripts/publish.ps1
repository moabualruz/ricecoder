# RiceCoder Automated Publishing Script
# Analyzes dependencies and publishes crates in correct order

param(
    [switch]$DryRun = $false,
    [switch]$Verbose = $false,
    [int]$WaitSeconds = 10
)

# Colors for output
$Colors = @{
    Success = 'Green'
    Error = 'Red'
    Warning = 'Yellow'
    Info = 'Cyan'
}

function Write-Log {
    param(
        [string]$Message,
        [string]$Level = 'Info'
    )
    $color = $Colors[$Level]
    Write-Host "[$Level] $Message" -ForegroundColor $color
}

function Get-CrateDependencies {
    param([string]$CratePath)
    
    $cargoToml = Join-Path $CratePath "Cargo.toml"
    if (-not (Test-Path $cargoToml)) {
        return @()
    }
    
    $content = Get-Content $cargoToml -Raw
    $deps = @()
    
    # Find all ricecoder dependencies
    $pattern = 'ricecoder-[\w-]+'
    $matches = [regex]::Matches($content, $pattern)
    
    foreach ($match in $matches) {
        $dep = $match.Value
        if ($dep -notin $deps) {
            $deps += $dep
        }
    }
    
    return $deps
}

function Get-CrateInfo {
    param([string]$CratePath)
    
    $cargoToml = Join-Path $CratePath "Cargo.toml"
    $content = Get-Content $cargoToml -Raw
    
    # Extract crate name
    $nameMatch = [regex]::Match($content, 'name\s*=\s*"([^"]+)"')
    $name = $nameMatch.Groups[1].Value
    
    # Extract version
    $versionMatch = [regex]::Match($content, 'version\s*=\s*"([^"]+)"')
    $version = $versionMatch.Groups[1].Value
    
    # Check if publishable
    $publishable = $content -notmatch 'publish\s*=\s*false'
    
    return @{
        Name = $name
        Path = $CratePath
        Version = $version
        Publishable = $publishable
        Dependencies = Get-CrateDependencies $CratePath
    }
}

function Resolve-PublishOrder {
    param([hashtable[]]$Crates)
    
    $ordered = @()
    $processed = @()
    $processing = @()
    
    function Resolve-Crate {
        param([string]$CrateName)
        
        if ($CrateName -in $processed) {
            return
        }
        
        if ($CrateName -in $processing) {
            Write-Log "Circular dependency detected: $CrateName" -Level Warning
            return
        }
        
        $processing += $CrateName
        
        $crate = $Crates | Where-Object { $_.Name -eq $CrateName }
        if ($null -eq $crate) {
            $processing = $processing | Where-Object { $_ -ne $CrateName }
            return
        }
        
        # Resolve dependencies first
        foreach ($dep in $crate.Dependencies) {
            Resolve-Crate $dep
        }
        
        if ($crate.Publishable -and $CrateName -notin $processed) {
            $ordered += $crate
            $processed += $CrateName
        }
        
        $processing = $processing | Where-Object { $_ -ne $CrateName }
    }
    
    foreach ($crate in $Crates) {
        if ($crate.Publishable) {
            Resolve-Crate $crate.Name
        }
    }
    
    return $ordered
}

function Test-Crate {
    param([string]$CratePath)
    
    Write-Log "Testing $CratePath..." -Level Info
    
    Push-Location $CratePath
    try {
        # Run tests
        & cargo test --release 2>&1 | Out-Null
        if ($LASTEXITCODE -ne 0) {
            Write-Log "Tests failed for $CratePath" -Level Error
            return $false
        }
        
        # Check clippy
        & cargo clippy --release 2>&1 | Out-Null
        if ($LASTEXITCODE -ne 0) {
            Write-Log "Clippy warnings for $CratePath" -Level Warning
        }
        
        return $true
    }
    finally {
        Pop-Location
    }
}

function Publish-Crate {
    param(
        [string]$CratePath,
        [string]$CrateName,
        [bool]$DryRun
    )
    
    Write-Log "Publishing $CrateName..." -Level Info
    
    Push-Location $CratePath
    try {
        if ($DryRun) {
            Write-Log "DRY RUN: cargo publish --dry-run" -Level Warning
            & cargo publish --dry-run 2>&1
        }
        else {
            Write-Log "Publishing to crates.io..." -Level Info
            & cargo publish 2>&1
        }
        
        if ($LASTEXITCODE -ne 0) {
            Write-Log "Failed to publish $CrateName" -Level Error
            return $false
        }
        
        Write-Log "Successfully published $CrateName" -Level Success
        return $true
    }
    finally {
        Pop-Location
    }
}

# Main script
function Main {
    Write-Log "RiceCoder Automated Publishing Script" -Level Info
    Write-Log "======================================" -Level Info
    
    # Find all crates
    $cratesDir = Join-Path (Get-Location) "crates"
    if (-not (Test-Path $cratesDir)) {
        Write-Log "crates directory not found" -Level Error
        exit 1
    }
    
    Write-Log "Scanning crates..." -Level Info
    $crates = @()
    Get-ChildItem $cratesDir -Directory | ForEach-Object {
        $crateInfo = Get-CrateInfo $_.FullName
        $crates += $crateInfo
        Write-Log "Found: $($crateInfo.Name) v$($crateInfo.Version)" -Level Info
    }
    
    # Resolve publish order
    Write-Log "Resolving publish order..." -Level Info
    $ordered = Resolve-PublishOrder $crates
    
    Write-Log "Publish order:" -Level Info
    $ordered | ForEach-Object { Write-Log "  $($_.Name) v$($_.Version)" -Level Info }
    
    # Confirm before publishing
    if (-not $DryRun) {
        $confirm = Read-Host "Proceed with publishing? (yes/no)"
        if ($confirm -ne "yes") {
            Write-Log "Publishing cancelled" -Level Warning
            exit 0
        }
    }
    
    # Publish crates
    $failed = @()
    $published = @()
    
    foreach ($crate in $ordered) {
        Write-Log "======================================" -Level Info
        
        # Test crate
        if (-not (Test-Crate $crate.Path)) {
            Write-Log "Skipping $($crate.Name) due to test failures" -Level Warning
            $failed += $crate.Name
            continue
        }
        
        # Publish crate
        if (Publish-Crate $crate.Path $crate.Name $DryRun) {
            $published += $crate.Name
            
            # Wait for crates.io to index
            if (-not $DryRun) {
                Write-Log "Waiting $WaitSeconds seconds for crates.io indexing..." -Level Info
                Start-Sleep -Seconds $WaitSeconds
            }
        }
        else {
            $failed += $crate.Name
        }
    }
    
    # Summary
    Write-Log "======================================" -Level Info
    Write-Log "Publishing Summary" -Level Info
    Write-Log "======================================" -Level Info
    Write-Log "Published: $($published.Count)" -Level Success
    $published | ForEach-Object { Write-Log "  [+] $_" -Level Success }
    
    if ($failed.Count -gt 0) {
        Write-Log "Failed: $($failed.Count)" -Level Error
        $failed | ForEach-Object { Write-Log "  [-] $_" -Level Error }
        exit 1
    }
    
    Write-Log "All crates published successfully!" -Level Success
    exit 0
}

# Run main
Main
