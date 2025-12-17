# RiceCoder Scoop Bucket

This directory contains the Scoop manifest for RiceCoder.

## Setup

To use this manifest, you need to add the RiceCoder bucket:

```powershell
# Add the bucket
scoop bucket add ricecoder https://github.com/moabualruz/ricecoder

# Install RiceCoder
scoop install ricecoder
```

## Automated Updates

The `ricecoder.json` manifest is automatically updated when new releases are published via GitHub Actions. The update script:

1. Fetches the latest release information from GitHub API
2. Downloads release assets and calculates SHA256 checksums
3. Updates the manifest with new URLs and checksums
4. Commits the changes to the repository

## Manual Updates

To manually update the manifest:

```bash
cd scripts
node update_scoop_manifest.js
```

## Manifest Structure

The manifest supports:
- Windows x86_64 architecture
- Automatic checksum verification
- Binary installation (no compilation required)
- Desktop shortcuts and PATH integration

## Enterprise Features

- Supports enterprise deployment scenarios
- Compatible with enterprise security policies
- Integrates with existing Scoop infrastructure
- Suitable for enterprise distribution