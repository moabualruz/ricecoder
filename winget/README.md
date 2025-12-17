# RiceCoder Winget Manifests

This directory contains the Winget manifests for RiceCoder.

## Setup

To install RiceCoder using Winget:

```powershell
# Install RiceCoder
winget install RiceCoder.RiceCoder

# Verify installation
rice --version

# Update
winget upgrade RiceCoder.RiceCoder
```

## Automated Updates

The Winget manifests are automatically updated when new releases are published via GitHub Actions. The update script:

1. Fetches the latest release information from GitHub API
2. Downloads release assets and calculates SHA256 checksums
3. Updates the installer manifest with new URLs and checksums
4. Updates the locale manifest with version and release notes
5. Commits the changes to the repository

## Manual Updates

To manually update the manifests:

```bash
cd scripts
node update_winget_manifests.js
```

## Manifest Structure

- `RiceCoder.RiceCoder.installer.yaml`: Defines installation parameters, URLs, and checksums
- `RiceCoder.RiceCoder.locale.en-US.yaml`: Defines package metadata, descriptions, and tags

## Enterprise Features

- Official Microsoft Windows Package Manager integration
- Enterprise-ready deployment
- Compatible with enterprise security policies
- Supports enterprise distribution scenarios
- Integrated with Windows Update mechanisms