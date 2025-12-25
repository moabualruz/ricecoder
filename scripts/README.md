# RiceCoder Publishing Scripts

Automated scripts for publishing RiceCoder crates to crates.io based on dependency requirements.

## Prerequisites

Before publishing, each crate must have:
- `README.md` - Crate documentation
- `CHANGELOG.md` - Change history in Keep a Changelog format
- Valid `Cargo.toml` with all required fields

These files must be added manually to each crate before running the publish script.

## Scripts

### `publish.ps1` (Windows PowerShell)

Automated publishing script that:
- Scans all crates and their dependencies
- Resolves correct publishing order
- Tests each crate before publishing
- Publishes crates in dependency order
- Waits for crates.io indexing between publishes

**Usage**:

```powershell
# Dry run (test without publishing)
.\scripts\publish.ps1 -DryRun

# Publish all crates
.\scripts\publish.ps1

# Publish with custom wait time
.\scripts\publish.ps1 -WaitSeconds 15

# Verbose output
.\scripts\publish.ps1 -Verbose
```

**Features**:
- Automatic dependency resolution
- Topological sorting for correct publish order
- Pre-publish testing (cargo test, cargo clippy)
- Colored output for easy reading
- Confirmation prompt before publishing
- Summary report with success/failure counts

### `publish.sh` (Unix/Linux/macOS)

Same functionality as `publish.ps1` but for Unix-like systems.

**Usage**:

```bash
# Dry run (test without publishing)
./scripts/publish.sh --dry-run

# Publish all crates
./scripts/publish.sh

# Publish with custom wait time
./scripts/publish.sh --wait 15

# Verbose output
./scripts/publish.sh --verbose
```

## Workflow

### Prerequisites

Before publishing, ensure each crate has:

1. **README.md** - Documentation with:
   - Crate description
   - Features list
   - Installation instructions
   - Usage examples
   - License

2. **CHANGELOG.md** - Change history in [Keep a Changelog](https://keepachangelog.com/) format with:
   - Version number
   - Release date
   - Added/Changed/Fixed/Removed sections

3. **Valid Cargo.toml** with:
   - name, version, edition
   - authors, license
   - description (max 160 chars)
   - repository, homepage, documentation
   - keywords (max 5), categories (max 5)

### Publishing Workflow

1. Update version in workspace `Cargo.toml`:

```toml
[workspace.package]
version = "0.1.72"
```

2. Update CHANGELOG.md files for each crate with release notes

3. Commit changes:

```bash
git add Cargo.toml crates/*/CHANGELOG.md
git commit -m "Bump version to 0.1.72"
```

4. Run publish script (handles dependency order automatically):

```bash
# Windows
.\scripts\publish.ps1

# Unix
./scripts/publish.sh
```

5. Create git tag:

```bash
git tag -a v0.1.72 -m "Release v0.1.72"
git push origin v0.1.72
```

6. Create GitHub release

## How It Works

### Dependency Resolution

The scripts analyze `Cargo.toml` files to:
1. Find all ricecoder dependencies
2. Build a dependency graph
3. Perform topological sort
4. Generate correct publishing order

**Example**:
- `ricecoder-storage` (no deps) → published first
- `ricecoder-generation` (depends on storage) → published second
- `ricecoder-lsp` (depends on generation) → published third

### Publishing Process

For each crate in order:
1. Run `cargo test --release`
2. Run `cargo clippy --release`
3. Run `cargo publish --dry-run` (if dry-run mode)
4. Run `cargo publish` (if not dry-run)
5. Wait for crates.io indexing
6. Continue to next crate

### Error Handling

If a crate fails:
- Testing fails → skip crate, continue with others
- Publishing fails → mark as failed, continue with others
- Summary shows all failures at end

## Requirements

### Windows (PowerShell)

- PowerShell 5.0+
- Cargo installed
- Git installed

### Unix/Linux/macOS (Bash)

- Bash 4.0+
- Cargo installed
- Git installed
- Standard Unix tools (grep, sed, etc.)

## Configuration

### Wait Time

Adjust wait time between publishes (default: 10 seconds):

```bash
# Windows
.\scripts\publish.ps1 -WaitSeconds 15

# Unix
./scripts/publish.sh --wait 15
```

Increase if crates.io is slow to index.

### Dry Run

Always test with dry-run first:

```bash
# Windows
.\scripts\publish.ps1 -DryRun

# Unix
./scripts/publish.sh --dry-run
```

This verifies:
- All dependencies are available
- Cargo.toml is valid
- No publishing issues

## Troubleshooting

### "no matching package named `ricecoder-X` found"

The dependency hasn't been published yet. The script should handle this automatically by publishing in correct order.

### "Version already exists"

Update version in `Cargo.toml` and try again.

### "Unauthorized"

Generate new API token:
1. Go to https://crates.io/me
2. Click "API Tokens"
3. Generate new token
4. Run: `cargo login NEW_TOKEN`

### Script won't run (Unix)

Make sure script is executable:

```bash
chmod +x scripts/publish.sh
chmod +x scripts/generate-templates.sh
```

## Examples

### Example 1: Dry Run (Test Before Publishing)

```bash
# Windows
.\scripts\publish.ps1 -DryRun

# Unix
./scripts/publish.sh --dry-run
```

Output:
```
[INFO] RiceCoder Automated Publishing Script
[INFO] ======================================
[INFO] Scanning crates...
[INFO] Found: ricecoder-storage v0.1.0
[INFO] Found: ricecoder-generation v0.1.71
[INFO] Resolving publish order...
[INFO] Publish order:
[INFO]   ricecoder-storage v0.1.0
[INFO]   ricecoder-generation v0.1.71
[WARNING] DRY RUN: cargo publish --dry-run
[SUCCESS] Successfully published ricecoder-storage
[INFO] Waiting 10 seconds for crates.io indexing...
[SUCCESS] Successfully published ricecoder-generation
[INFO] ======================================
[INFO] Publishing Summary
[INFO] ======================================
[SUCCESS] Published: 2
[SUCCESS]   ✓ ricecoder-storage
[SUCCESS]   ✓ ricecoder-generation
[SUCCESS] All crates published successfully!
```

### Example 2: Full Publish (Automatic Dependency Order)

```bash
# Windows
.\scripts\publish.ps1

# Unix
./scripts/publish.sh
```

Output:
```
[INFO] RiceCoder Automated Publishing Script
[INFO] ======================================
[INFO] Scanning crates...
[INFO] Found: ricecoder-storage v0.1.0
[INFO] Found: ricecoder-generation v0.1.71
[INFO] Resolving publish order...
[INFO] Publish order:
[INFO]   ricecoder-storage v0.1.0
[INFO]   ricecoder-generation v0.1.71
Proceed with publishing? (yes/no): yes
[INFO] ======================================
[INFO] Testing ricecoder-storage...
[SUCCESS] Successfully published ricecoder-storage
[INFO] Waiting 10 seconds for crates.io indexing...
[INFO] ======================================
[INFO] Testing ricecoder-generation...
[SUCCESS] Successfully published ricecoder-generation
[INFO] ======================================
[INFO] Publishing Summary
[INFO] ======================================
[SUCCESS] Published: 2
[SUCCESS]   ✓ ricecoder-storage
[SUCCESS]   ✓ ricecoder-generation
[SUCCESS] All crates published successfully!
```

## See Also

- [CARGO_PUBLISHING_GUIDE.md](../.ai/specs/ricecoder-advanced/docs/CARGO_PUBLISHING_GUIDE.md) - Complete publishing guide
- [PUBLISHING.md](../PUBLISHING.md) - Quick reference
- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
- [Crates.io](https://crates.io)
