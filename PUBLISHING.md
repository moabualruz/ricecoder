# Publishing RiceCoder to Crates.io

Quick reference for publishing RiceCoder crates using automated scripts.

## Prerequisites

### Crates.io Setup

1. Create crates.io account: https://crates.io
2. Generate API token: https://crates.io/me
3. Login: `cargo login YOUR_TOKEN`

### Per-Crate Requirements

Each crate must have:
- `README.md` - Crate documentation
- `CHANGELOG.md` - Change history (Keep a Changelog format)
- Valid `Cargo.toml` with all required fields

## Publishing Scripts

### Available Scripts

#### `scripts/publish.ps1` (Windows PowerShell)

Automated publishing script that handles dependency resolution and publishes crates in correct order.

**Features**:
- Scans all crates and analyzes dependencies
- Performs topological sort for correct publishing order
- Tests each crate before publishing (`cargo test --release`, `cargo clippy --release`)
- Publishes crates in dependency order
- Waits for crates.io indexing between publishes
- Colored output for easy reading
- Confirmation prompt before publishing
- Summary report with success/failure counts

**Usage**:

```powershell
# Dry run (test without publishing)
.\scripts\publish.ps1 -DryRun

# Publish all crates
.\scripts\publish.ps1

# Publish with custom wait time (seconds)
.\scripts\publish.ps1 -WaitSeconds 15

# Verbose output
.\scripts\publish.ps1 -Verbose
```

#### `scripts/publish.sh` (Unix/Linux/macOS)

Same functionality as `publish.ps1` but for Unix-like systems.

**Usage**:

```bash
# Dry run (test without publishing)
./scripts/publish.sh --dry-run

# Publish all crates
./scripts/publish.sh

# Publish with custom wait time (seconds)
./scripts/publish.sh --wait 15

# Verbose output
./scripts/publish.sh --verbose
```

#### `scripts/create-readme-changelog.ps1` (Windows PowerShell)

Creates README.md and CHANGELOG.md files for all crates that don't have them.

**Features**:
- Scans all crates
- Extracts crate name and description from Cargo.toml
- Generates README.md with standard structure
- Generates CHANGELOG.md in Keep a Changelog format
- Skips existing files (won't overwrite)
- Shows progress with colored output

**Usage**:

```powershell
# Generate README and CHANGELOG for all crates
.\scripts\create-readme-changelog.ps1
```

#### `scripts/create-readme-changelog.sh` (Unix/Linux/macOS)

Same functionality as `create-readme-changelog.ps1` but for Unix-like systems.

**Usage**:

```bash
# Generate README and CHANGELOG for all crates
./scripts/create-readme-changelog.sh
```

## Quick Start

### 1. Generate README and CHANGELOG (if needed)

If crates are missing README.md or CHANGELOG.md files:

```bash
# Windows
.\scripts\create-readme-changelog.ps1

# Unix
./scripts/create-readme-changelog.sh
```

Then edit the generated files to add actual content.

### 2. Update Version

```bash
# Update workspace version
vim Cargo.toml
# Change [workspace.package] version = "0.1.72"

# Update CHANGELOG.md for each crate
vim crates/*/CHANGELOG.md
```

### 3. Commit Changes

```bash
git add Cargo.toml crates/*/CHANGELOG.md
git commit -m "Bump version to 0.1.72"
```

### 4. Test Publishing (Dry Run)

Always test before publishing:

```bash
# Windows
.\scripts\publish.ps1 -DryRun

# Unix
./scripts/publish.sh --dry-run
```

### 5. Publish (Automatic Dependency Order)

```bash
# Windows
.\scripts\publish.ps1

# Unix
./scripts/publish.sh
```

The script automatically:
- Scans all crates and dependencies
- Determines correct publishing order
- Tests each crate before publishing
- Publishes in dependency order
- Waits for crates.io indexing

### 6. Create Release

```bash
git tag -a v0.1.72 -m "Release v0.1.72"
git push origin v0.1.72
```

## How the Publishing Script Works

### Dependency Resolution

The `publish.ps1` and `publish.sh` scripts:

1. **Scan all crates** - Read all `Cargo.toml` files in `crates/` directory
2. **Extract dependencies** - Find all ricecoder dependencies in each crate
3. **Build dependency graph** - Map which crates depend on which
4. **Topological sort** - Determine correct publishing order
5. **Publish in order** - Publish crates so dependencies are available first

### Publishing Process

For each crate in dependency order:

1. **Test** - Run `cargo test --release` and `cargo clippy --release`
2. **Verify** - Check that all dependencies are available on crates.io
3. **Publish** - Run `cargo publish` (or `cargo publish --dry-run` in dry-run mode)
4. **Wait** - Wait for crates.io to index the crate (default: 10 seconds)
5. **Continue** - Move to next crate in order

### Error Handling

If a crate fails:
- **Test failure** - Skip crate, continue with others
- **Publish failure** - Mark as failed, continue with others
- **Summary** - Show all failures at end

## Example Publishing Order

For a typical RiceCoder setup:

```
1. ricecoder-storage (no internal deps)
2. ricecoder-specs (no internal deps)
3. ricecoder-research (no internal deps)
4. ricecoder-providers (depends on storage)
5. ricecoder-permissions (depends on storage)
6. ricecoder-files (depends on storage)
7. ricecoder-generation (depends on providers)
8. ricecoder-lsp (depends on generation)
9. ricecoder-completion (depends on lsp)
... and so on
```

The script determines this order automatically!

## Troubleshooting

### "no matching package named `ricecoder-X` found"

**Cause**: A crate depends on another ricecoder crate that hasn't been published yet.

**Solution**:
- The script handles this automatically by publishing in correct order
- Ensure all crates have README.md and CHANGELOG.md
- Run dry-run first to verify order: `./scripts/publish.sh --dry-run`

### "Version already exists"

**Cause**: The version has already been published to crates.io.

**Solution**:
1. Increment version in `Cargo.toml`
2. Update `CHANGELOG.md` for affected crates
3. Commit changes
4. Try publishing again

### "Unauthorized"

**Cause**: API token is invalid or expired.

**Solution**:
1. Generate new API token: https://crates.io/me
2. Run: `cargo login NEW_TOKEN`
3. Try publishing again

### "Missing README.md or CHANGELOG.md"

**Cause**: A crate doesn't have required documentation files.

**Solution**:
1. Generate files: `./scripts/create-readme-changelog.sh`
2. Edit files to add actual content
3. Commit changes
4. Try publishing again

### "Tests failed for crate-name"

**Cause**: `cargo test --release` failed for a crate.

**Solution**:
1. Fix the failing tests
2. Run tests locally: `cd crates/crate-name && cargo test --release`
3. Commit fixes
4. Try publishing again

### "Clippy warnings for crate-name"

**Cause**: `cargo clippy --release` found warnings.

**Solution**:
1. Fix clippy warnings: `cd crates/crate-name && cargo clippy --release`
2. Commit fixes
3. Try publishing again

### Script won't run (Unix)

**Cause**: Script doesn't have execute permission.

**Solution**:
```bash
chmod +x scripts/publish.sh
chmod +x scripts/create-readme-changelog.sh
```

### Crates.io indexing is slow

**Cause**: crates.io is taking longer than expected to index a crate.

**Solution**:
- Increase wait time: `./scripts/publish.sh --wait 30`
- Default wait time is 10 seconds
- Increase if you see "no matching package" errors

## Script Comparison

| Script | Purpose | When to Use |
|--------|---------|------------|
| `publish.ps1` / `publish.sh` | Publish all crates in dependency order | Main publishing workflow |
| `create-readme-changelog.ps1` / `create-readme-changelog.sh` | Generate README and CHANGELOG templates | First time setup or new crates |

## Complete Workflow Example

```bash
# 1. Generate templates for new crates (if needed)
./scripts/create-readme-changelog.sh

# 2. Edit README.md and CHANGELOG.md files
vim crates/*/README.md
vim crates/*/CHANGELOG.md

# 3. Update version
vim Cargo.toml
# Change [workspace.package] version = "0.1.72"

# 4. Update changelogs
vim crates/*/CHANGELOG.md

# 5. Commit
git add Cargo.toml crates/*/CHANGELOG.md crates/*/README.md
git commit -m "Bump version to 0.1.72"

# 6. Test (dry run)
./scripts/publish.sh --dry-run

# 7. Publish
./scripts/publish.sh

# 8. Create release
git tag -a v0.1.72 -m "Release v0.1.72"
git push origin v0.1.72

# 9. Create GitHub release
# Go to https://github.com/moabualruz/ricecoder/releases
# Create release from tag v0.1.72
```

## Full Guide

See `.kiro/specs/ricecoder-advanced/docs/CARGO_PUBLISHING_GUIDE.md` for complete documentation.

See `scripts/README.md` for detailed script documentation.
