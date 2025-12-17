# RiceCoder Homebrew Tap

This directory contains the Homebrew formula for RiceCoder.

## Setup

To use this formula, you need to add the RiceCoder tap:

```bash
# Add the tap
brew tap moabualruz/ricecoder

# Install RiceCoder
brew install ricecoder
```

## Automated Updates

The `ricecoder.rb` formula is automatically updated when new releases are published via GitHub Actions. The update script:

1. Fetches the latest release information from GitHub API
2. Downloads release assets and calculates SHA256 checksums
3. Updates the formula with new URLs and checksums
4. Commits the changes to the repository

## Manual Updates

To manually update the formula:

```bash
cd scripts
ruby update_homebrew_formula.rb
```

## Formula Structure

The formula supports:
- macOS x86_64 and ARM64 architectures
- Linux x86_64 and ARM64 architectures
- Automatic checksum verification
- Binary installation (no compilation required)

## Enterprise Features

- Supports enterprise deployment scenarios
- Compatible with enterprise security policies
- Integrates with existing Homebrew infrastructure