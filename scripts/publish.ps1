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
    param([string]$CratePath, [string]$CrateName)
    
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
        # Exclude the crate's own name and duplicates
        if ($dep -ne $CrateName -and $dep -notin $deps) {
            $deps += $dep
        }
    }
    
    return $deps
}

function Get-CrateInfo {
    param([string]$CratePath)
    
    $cargoToml = Join-Path $CratePath "Cargo.toml"
    if (-not (Test-Path $cargoToml)) {
        return $null
    }
    
    $content = Get-Content $cargoToml -Raw
    
    # Extract crate name
    $nameMatch = [regex]::Match($content, 'name\s*=\s*"([^"]+)"')
    $name = $nameMatch.Groups[1].Value
    
    if ([string]::IsNullOrEmpty($name)) {
        return $null
    }
    
    # Extract version (handle both direct and workspace versions)
    $version = ""
    
    # First check for version = "x.y.z" format (direct version)
    $versionMatch = [regex]::Match($content, '^\s*version\s*=\s*"([^"]+)"', [System.Text.RegularExpressions.RegexOptions]::Multiline)
    if ($versionMatch.Success) {
        $version = $versionMatch.Groups[1].Value
    }
    
    # If no direct version, check for version.workspace = true format
    if ([string]::IsNullOrEmpty($version)) {
        if ($content -match 'version\.workspace\s*=\s*true') {
            # Get workspace version from root Cargo.toml
            $rootCargoToml = Join-Path (Get-Location) "Cargo.toml"
            if (Test-Path $rootCargoToml) {
                $rootContent = Get-Content $rootCargoToml -Raw
                # Look for [workspace.package] section and extract version
                $workspaceMatch = [regex]::Match($rootContent, '\[workspace\.package\].*?version\s*=\s*"([^"]+)"', [System.Text.RegularExpressions.RegexOptions]::Singleline)
                if ($workspaceMatch.Success) {
                    $version = $workspaceMatch.Groups[1].Value
                }
            }
        }
    }
    
    # Check if publishable
    $publishable = $content -notmatch 'publish\s*=\s*false'
    
    return @{
        Name = $name
        Path = $CratePath
        Version = $version
        Publishable = $publishable
        Dependencies = Get-CrateDependencies $CratePath $name
    }
}



function Get-PublishedRiceoderCrates {
    # Search for all ricecoder crates in one go
    Write-Log "Checking published crates on crates.io..." -Level Info
    
    try {
        $response = & cargo search ricecoder --limit 200 2>&1
        $published = @{}
        
        # Parse output: each line is "crate-name = "version" # description"
        foreach ($line in $response) {
            if ($line -match "^(ricecoder-[\w-]+)\s*=\s*""([^""]+)""") {
                $crateName = $matches[1]
                $version = $matches[2]
                $published[$crateName] = $version
                if ($Verbose) {
                    Write-Log "  Found published: $crateName = $version" -Level Info
                }
            }
        }
        
        Write-Log "Found $($published.Count) published ricecoder crates" -Level Info
        return $published
    }
    catch {
        Write-Log "Warning: Could not check published crates, will attempt all" -Level Warning
        return @{}
    }
}

function Is-CratePublished {
    param(
        [string]$CrateName,
        [hashtable]$PublishedCrates
    )
    
    return $PublishedCrates.ContainsKey($CrateName)
}

function Should-PublishCrate {
    param(
        [string]$CrateName,
        [string]$LocalVersion,
        [hashtable]$PublishedCrates
    )
    
    # If not published, should publish
    if (-not $PublishedCrates.ContainsKey($CrateName)) {
        if ($Verbose) {
            Write-Log "  ${CrateName}: Not published yet, will publish" -Level Info
        }
        return $true
    }
    
    # If published, compare versions
    $publishedVersion = $PublishedCrates[$CrateName]
    
    # Simple string comparison for versions (works for semver)
    # If versions are equal, don't publish
    if ($LocalVersion -eq $publishedVersion) {
        if ($Verbose) {
            Write-Log "  ${CrateName}: Local version $LocalVersion equals published version $publishedVersion, skipping" -Level Info
        }
        return $false
    }
    
    # Parse versions for comparison (extract numeric parts only)
    $localParts = @()
    $publishedParts = @()
    
    # Extract numeric parts from local version
    foreach ($part in ($LocalVersion -split '\.')) {
        if ($part -match '^(\d+)') {
            $localParts += [int]$matches[1]
        }
    }
    
    # Extract numeric parts from published version
    foreach ($part in ($publishedVersion -split '\.')) {
        if ($part -match '^(\d+)') {
            $publishedParts += [int]$matches[1]
        }
    }
    
    # Compare numeric parts
    for ($i = 0; $i -lt [Math]::Max($localParts.Count, $publishedParts.Count); $i++) {
        $localPart = if ($i -lt $localParts.Count) { $localParts[$i] } else { 0 }
        $publishedPart = if ($i -lt $publishedParts.Count) { $publishedParts[$i] } else { 0 }
        
        if ($localPart -gt $publishedPart) {
            if ($Verbose) {
                Write-Log "  ${CrateName}: Local version $LocalVersion is newer than published $publishedVersion, will publish" -Level Info
            }
            return $true  # Local version is newer
        }
        elseif ($localPart -lt $publishedPart) {
            if ($Verbose) {
                Write-Log "  ${CrateName}: Published version $publishedVersion is newer than local $LocalVersion, skipping" -Level Info
            }
            return $false  # Published version is newer
        }
    }
    
    # Versions are equal
    if ($Verbose) {
        Write-Log "  ${CrateName}: Versions are equal ($LocalVersion), skipping" -Level Info
    }
    return $false
}

function Parse-RateLimitTime {
    param([string]$ErrorMsg)
    
    # Extract timestamp like "Tue, 09 Dec 2025 16:27:40 GMT"
    if ($ErrorMsg -match "after\s+([A-Za-z]+,\s+\d+\s+[A-Za-z]+\s+\d+\s+\d+:\d+:\d+\s+GMT)") {
        $gmtTimeStr = $matches[1]
        try {
            # Parse GMT time
            $gmtTime = [DateTime]::ParseExact($gmtTimeStr, "ddd, dd MMM yyyy HH:mm:ss 'GMT'", [System.Globalization.CultureInfo]::InvariantCulture)
            # Convert to local time
            $localTime = $gmtTime.ToLocalTime()
            return $localTime
        }
        catch {
            Write-Log "Failed to parse rate limit time: $gmtTimeStr" -Level Warning
            return $null
        }
    }
    return $null
}

function Wait-ForRateLimit {
    param([DateTime]$RetryTime)
    
    $now = Get-Date
    if ($RetryTime -le $now) {
        return
    }
    
    $waitSeconds = [Math]::Ceiling(($RetryTime - $now).TotalSeconds)
    Write-Log "Rate limit detected. Retry available at: $RetryTime (local time)" -Level Warning
    Write-Log "Waiting $waitSeconds seconds before retrying..." -Level Info
    
    # Show countdown every 30 seconds
    $elapsed = 0
    while ($elapsed -lt $waitSeconds) {
        $remaining = $waitSeconds - $elapsed
        if ($remaining -le 0) { break }
        
        $sleepTime = [Math]::Min(30, $remaining)
        Start-Sleep -Seconds $sleepTime
        $elapsed += $sleepTime
        
        if ($remaining -gt 30) {
            $remainingMin = [Math]::Ceiling($remaining / 60)
            Write-Log "Still waiting... $remainingMin minutes remaining" -Level Info
        }
    }
    
    Write-Log "Rate limit period expired, resuming publishing..." -Level Success
}

function Publish-Crate {
    param(
        [string]$CratePath,
        [string]$CrateName,
        [string]$LocalVersion,
        [bool]$DryRun,
        [hashtable]$PublishedCrates
    )
    
    # Check if we should publish (new crate or newer version)
    if (-not (Should-PublishCrate $CrateName $LocalVersion $PublishedCrates)) {
        Write-Log "Crate already published with same or newer version on crates.io - skipping" -Level Warning
        return 0  # Success (already published, no need to publish)
    }
    
    Write-Log "Publishing $CrateName..." -Level Info
    
    Push-Location $CratePath
    try {
        if ($DryRun) {
            Write-Log "DRY RUN: Skipping actual publish (would run: cargo publish)" -Level Warning
            return 0  # Success
        }
        
        Write-Log "Running: cargo publish" -Level Info
        $output = & cargo publish 2>&1
        $exitCode = $LASTEXITCODE
        
        if ($exitCode -ne 0) {
            $errorMsg = $output -join "`n"
            Write-Log "ERROR: cargo publish failed with exit code $exitCode" -Level Error
            Write-Log "Output: $errorMsg" -Level Error
            
            # Check if it's a rate limit error
            if ($errorMsg -match "429 Too Many Requests" -or $errorMsg -match "published too many new crates") {
                $retryTime = Parse-RateLimitTime $errorMsg
                if ($null -ne $retryTime) {
                    Write-Log "Reason: Rate limit - waiting and retrying once" -Level Warning
                    Wait-ForRateLimit $retryTime
                    
                    # Retry once after waiting
                    Write-Log "Retrying after rate limit wait..." -Level Info
                    $output = & cargo publish 2>&1
                    $exitCode = $LASTEXITCODE
                    
                    if ($exitCode -eq 0) {
                        Write-Log "SUCCESS: $CrateName published to crates.io (after rate limit wait)" -Level Success
                        return 0  # Success
                    }
                    
                    # Still failed after retry - check if it's another rate limit or different error
                    $errorMsg = $output -join "`n"
                    Write-Log "ERROR: cargo publish still failed after rate limit wait" -Level Error
                    Write-Log "Output: $errorMsg" -Level Error
                    
                    # Check if it's another rate limit
                    if ($errorMsg -match "429 Too Many Requests" -or $errorMsg -match "published too many new crates") {
                        Write-Log "Reason: Rate limit again - will retry later" -Level Warning
                        return 2  # Retry (add to retry list)
                    }
                    
                    # Check if it's a dependency error
                    if ($errorMsg -match "no matching package named" -or $errorMsg -match "not found in registry") {
                        Write-Log "Reason: Dependency not yet published - will retry individually" -Level Warning
                        return 2  # Retry
                    }
                    
                    # Other error - add to retry list
                    Write-Log "Reason: Unknown error - will retry" -Level Warning
                    return 2  # Retry
                }
            }
            
            # Check if it's a dependency error
            if ($errorMsg -match "no matching package named" -or $errorMsg -match "not found in registry") {
                Write-Log "Reason: Dependency not yet published - will retry individually" -Level Warning
                return 2  # Retry
            }
            
            # Check for other common errors
            if ($errorMsg -match "already exists") {
                Write-Log "Reason: Crate version already published" -Level Warning
                return 0  # Success
            }
            
            # Other errors - add to retry list
            Write-Log "Reason: Unknown error - will retry" -Level Warning
            return 2  # Retry
        }
        
        Write-Log "SUCCESS: $CrateName published to crates.io" -Level Success
        return 0  # Success
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
        if ($null -ne $crateInfo) {
            $crates += $crateInfo
            Write-Log "Found: $($crateInfo.Name) v$($crateInfo.Version)" -Level Info
        }
    }
    
    # Filter publishable crates
    Write-Log "Filtering publishable crates..." -Level Info
    $publishable = $crates | Where-Object { $_.Publishable }
    
    # Build dependency graph
    $depGraph = @{}
    foreach ($crate in $publishable) {
        $depGraph[$crate.Name] = @{
            Crate = $crate
            Dependencies = @()
        }
    }
    
    # Populate dependencies (only ricecoder crates)
    foreach ($crate in $publishable) {
        foreach ($dep in $crate.Dependencies) {
            if ($depGraph.ContainsKey($dep)) {
                $depGraph[$crate.Name].Dependencies += $dep
            }
        }
    }
    
    # Sort by number of internal dependencies (ascending)
    # Crates with 0 dependencies first, then 1, then 2, etc.
    Write-Log "Sorting crates by internal dependency count..." -Level Info
    
    $ordered = $publishable | Sort-Object {
        $depCount = $depGraph[$_.Name].Dependencies.Count
        $depCount
    }
    
    Write-Log "Crates to publish: $($ordered.Count)" -Level Info
    Write-Log "Publish order (by dependency count):" -Level Info
    $ordered | ForEach-Object { 
        $depList = $depGraph[$_.Name].Dependencies
        $depCount = $depList.Count
        if ($depCount -gt 0) {
            Write-Log "  $($_.Name) v$($_.Version) [$depCount deps: $($depList -join ', ')]" -Level Info
        } else {
            Write-Log "  $($_.Name) v$($_.Version) [0 deps]" -Level Info
        }
    }
    
    # Get all published ricecoder crates in one go
    $publishedCrates = @{}
    if (-not $DryRun) {
        $publishedCrates = Get-PublishedRiceoderCrates
    }
    
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
    $retryList = @()
    
    # First pass: try to publish all crates (only if all dependencies are published)
    Write-Log "First pass: attempting to publish all crates..." -Level Info
    foreach ($crate in $ordered) {
        Write-Log "======================================" -Level Info
        
        # On dry-run, skip dependency checks - just show what would be published
        if (-not $DryRun) {
            # Check if all dependencies are published
            $allDepsPublished = $true
            foreach ($dep in $crate.Dependencies) {
                if (-not (Is-CratePublished $dep $publishedCrates)) {
                    $allDepsPublished = $false
                    Write-Log "⏳ $($crate.Name) - skipping, waiting for dependency: $dep" -Level Warning
                    $retryList += $crate
                    break
                }
            }
            
            if (-not $allDepsPublished) {
                continue
            }
        }
        
        # Publish crate
        $result = Publish-Crate $crate.Path $crate.Name $crate.Version $DryRun $publishedCrates
        
        if ($DryRun) {
            # On dry-run, just mark as published
            Write-Log "✓ $($crate.Name) would be published" -Level Success
            $published += $crate.Name
        }
        elseif ($result -eq 2) {
            Write-Log "⟳ $($crate.Name) - dependency error, will retry" -Level Warning
            $retryList += $crate
        }
        elseif ($result -eq 0) {
            Write-Log "✓ $($crate.Name) - published successfully" -Level Success
            $published += $crate.Name
            
            # Only wait if we actually published (not skipped)
            # Check if this was a skip or actual publish by checking if it was in published crates
            if ($publishedCrates.ContainsKey($crate.Name)) {
                # Was already published, no need to wait
            }
            else {
                # Was newly published, wait for crates.io to index
                Write-Log "Waiting $WaitSeconds seconds for crates.io indexing..." -Level Info
                Start-Sleep -Seconds $WaitSeconds
            }
        }
        else {
            Write-Log "✗ $($crate.Name) - publish failed" -Level Error
            $failed += $crate.Name
        }
    }
    
    # Second pass: retry failed crates - only send request if ALL dependencies are published
    if ($retryList.Count -gt 0) {
        Write-Log "======================================" -Level Info
        Write-Log "Second pass: retrying crates when dependencies are published..." -Level Warning
        Write-Log "======================================" -Level Info
        
        # Check if we hit a rate limit and need to wait
        if ($null -ne $global:RateLimitRetryTime) {
            Wait-ForRateLimit $global:RateLimitRetryTime
            $global:RateLimitRetryTime = $null
        }
        
        $maxRetryAttempts = 10
        $retryAttempt = 0
        
        while ($retryList.Count -gt 0 -and $retryAttempt -lt $maxRetryAttempts) {
            $retryAttempt++
            $newRetryList = @()
            
            foreach ($crate in $retryList) {
                # Check if all dependencies are published BEFORE sending request
                $allDepsPublished = $true
                foreach ($dep in $crate.Dependencies) {
                    if (-not (Is-CratePublished $dep $publishedCrates)) {
                        $allDepsPublished = $false
                        Write-Log "⏳ $($crate.Name) - waiting for dependency: $dep" -Level Info
                        break
                    }
                }
                
                if (-not $allDepsPublished) {
                    # Keep in retry list, don't send request yet
                    $newRetryList += $crate
                    continue
                }
                
                # All dependencies published, now try to publish
                Write-Log "⟳ Retrying: $($crate.Name)" -Level Warning
                $result = Publish-Crate $crate.Path $crate.Name $crate.Version $DryRun $publishedCrates
                
                if ($result -eq 0) {
                    Write-Log "✓ $($crate.Name) - published on retry" -Level Success
                    $published += $crate.Name
                    
                    # Wait for crates.io to index
                    Write-Log "Waiting $WaitSeconds seconds for crates.io indexing..." -Level Info
                    Start-Sleep -Seconds $WaitSeconds
                }
                elseif ($result -eq 3) {
                    # Rate limit hit again - wait and keep in retry list
                    Write-Log "⏳ $($crate.Name) - rate limited again, will wait and retry" -Level Warning
                    $newRetryList += $crate
                    if ($null -ne $global:RateLimitRetryTime) {
                        Wait-ForRateLimit $global:RateLimitRetryTime
                        $global:RateLimitRetryTime = $null
                    }
                }
                elseif ($result -eq 2) {
                    # Still has issues, keep in retry list
                    Write-Log "⏳ $($crate.Name) - still has issues, will retry" -Level Warning
                    $newRetryList += $crate
                }
                else {
                    # Failed permanently
                    Write-Log "✗ $($crate.Name) - failed on retry" -Level Error
                    $failed += $crate.Name
                }
            }
            
            $retryList = $newRetryList
            
            if ($retryList.Count -gt 0) {
                Write-Log "Waiting 30 seconds before next retry attempt ($retryAttempt/$maxRetryAttempts)..." -Level Info
                Start-Sleep -Seconds 30
            }
        }
        
        # Any remaining crates in retry list are failures
        foreach ($crate in $retryList) {
            Write-Log "✗ $($crate.Name) - failed after $maxRetryAttempts retry attempts" -Level Error
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
