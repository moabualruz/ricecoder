#!/usr/bin/env pwsh
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Write-Host "=== Updating all dependencies to latest ==="
cargo update

Write-Host ""
Write-Host "=== Checking for duplicate dependencies ==="

if (-not (Test-Path -LiteralPath "target")) {
    New-Item -ItemType Directory -Path "target" | Out-Null
}

cargo tree -d | Set-Content -Path "target/duplicates.txt" -Encoding UTF8

if ((Get-Item -LiteralPath "target/duplicates.txt").Length -gt 0) {
    Write-Warning "Duplicate versions detected. See target/duplicates.txt"
} else {
    Write-Host "No duplicate versions detected"
}

Write-Host ""
Write-Host "=== Generating dependency report ==="

$pythonScriptPath = Join-Path ([System.IO.Path]::GetTempPath()) "resolve_deps_parse.py"
$pythonSource = @'
import json
import sys
from pathlib import Path

def load_metadata(path: str | None):
    if path:
        with Path(path).open("r", encoding="utf-8") as handle:
            return json.load(handle)
    return json.load(sys.stdin)

metadata = load_metadata(sys.argv[1] if len(sys.argv) > 1 else None)
rows = []
for pkg in metadata.get("packages", []):
    source = pkg.get("source")
    if source:
        rows.append((pkg["name"], pkg["version"]))
for name, version in sorted(rows):
    print(f"      - {name} → resolved {version}")
'@
Set-Content -Path $pythonScriptPath -Value $pythonSource -Encoding UTF8
$metadataPath = Join-Path ([System.IO.Path]::GetTempPath()) "resolve_deps_metadata.json"

try {
    $metadataJson = cargo metadata --format-version=1 --quiet 2>$null
    $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($metadataPath, $metadataJson, $utf8NoBom)
    $report = python $pythonScriptPath $metadataPath
    $report | Set-Content -Path "target/deps-report.txt" -Encoding UTF8
}
finally {
    Remove-Item -LiteralPath $pythonScriptPath -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $metadataPath -ErrorAction SilentlyContinue
}

Write-Host ""
Write-Host "=== Report written to target/deps-report.txt ==="
