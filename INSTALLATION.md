# RiceCoder Installation Guide

This guide covers building and installing RiceCoder from source on Linux, macOS, and Windows.

## Table of Contents

- [Quick Start](#quick-start)
- [Prerequisites](#prerequisites)
- [Installation by Platform](#installation-by-platform)
- [Configuration](#configuration)
- [Verification](#verification)
- [Troubleshooting](#troubleshooting)
- [Uninstallation](#uninstallation)

## Quick Start

### Linux/macOS (Remote Installation via curl)

```bash
curl -fsSL https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install | bash
```

### Linux/macOS (Local Installation)

```bash
cd ricecoder
chmod +x scripts/install.sh
./scripts/install.sh
```

### Windows (PowerShell Remote Installation)

```powershell
iex (New-Object Net.WebClient).DownloadString('https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install.ps1')
```

### Windows (PowerShell Local Installation)

```powershell
cd ricecoder
.\scripts\install.ps1
```

### Windows (CMD Local Installation)

```cmd
cd ricecoder
scripts\install.bat
```

## Remote Installation (via curl)

### Linux/macOS

Download and run the installation script directly:

```bash
curl -fsSL https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install | bash
```

This automatically detects your OS and runs the appropriate installation script.

**With options:**
```bash
curl -fsSL https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install | bash -s -- --prefix /usr/local --debug
```

### Windows (PowerShell)

Download and run the installation script directly:

```powershell
iex (New-Object Net.WebClient).DownloadString('https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install.ps1')
```

Or with newer PowerShell (7+):

```powershell
iex (Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install.ps1').Content
```

**With options:**
```powershell
$params = @{
    Prefix = "C:\Program Files\ricecoder"
    BuildMode = "debug"
}
iex (New-Object Net.WebClient).DownloadString('https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install.ps1') @params
```

---

## Prerequisites

### All Platforms

- **Rust 1.70+**: Install from [https://rustup.rs/](https://rustup.rs/)
- **Git**: Install from [https://git-scm.com/](https://git-scm.com/)
- **Cargo**: Included with Rust

### Linux

- **Build Tools**: `gcc`, `make`, `pkg-config`
- **Development Headers**: `libssl-dev`, `libffi-dev`

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev libffi-dev
```

**Fedora/RHEL:**
```bash
sudo dnf install -y gcc make pkg-config openssl-devel libffi-devel
```

**Arch:**
```bash
sudo pacman -S base-devel pkg-config openssl
```

### macOS

- **Xcode Command Line Tools**: `xcode-select --install`
- **Homebrew** (optional): Install from [https://brew.sh/](https://brew.sh/)

```bash
xcode-select --install
```

### Windows

- **Visual Studio Build Tools** or **MSVC Compiler**
- **Windows 10/11**

Download from: [https://visualstudio.microsoft.com/downloads/](https://visualstudio.microsoft.com/downloads/)

Select "Desktop development with C++" workload.

## Installation by Platform

### Linux

#### Using the Installation Script

```bash
cd ricecoder
chmod +x scripts/install.sh
./scripts/install.sh [OPTIONS]
```

**Options:**
- `--prefix PATH`: Installation prefix (default: `~/.local`)
- `--release`: Build in release mode (default)
- `--debug`: Build in debug mode
- `--no-strip`: Don't strip binaries
- `--verbose`: Show verbose output
- `--help`: Show help message

**Examples:**

```bash
# Install to ~/.local (default)
./scripts/install.sh

# Install to /usr/local
./scripts/install.sh --prefix /usr/local

# Debug build
./scripts/install.sh --debug

# Verbose output
./scripts/install.sh --verbose
```

#### Manual Installation

```bash
# Build
cd ricecoder
cargo build --release

# Install binaries
mkdir -p ~/.local/bin
cp target/release/ricecoder* ~/.local/bin/

# Add to PATH
export PATH="$HOME/.local/bin:$PATH"

# Make permanent (add to ~/.bashrc or ~/.zshrc)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
```

### macOS

#### Using the Installation Script

```bash
cd ricecoder
chmod +x scripts/install.sh
./scripts/install.sh [OPTIONS]
```

**Examples:**

```bash
# Install to ~/.local (default)
./scripts/install.sh

# Install to /usr/local
./scripts/install.sh --prefix /usr/local

# Debug build
./scripts/install.sh --debug
```

#### Using Homebrew (if available)

```bash
brew install ricecoder
```

#### Manual Installation

```bash
# Build
cd ricecoder
cargo build --release

# Install binaries
mkdir -p ~/.local/bin
cp target/release/ricecoder* ~/.local/bin/

# Add to PATH
export PATH="$HOME/.local/bin:$PATH"

# Make permanent (add to ~/.zprofile)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zprofile
```

### Windows (PowerShell)

#### Using the Installation Script

```powershell
cd ricecoder
.\scripts\install.ps1 [OPTIONS]
```

**Options:**
- `-Prefix PATH`: Installation prefix (default: `$env:LOCALAPPDATA\ricecoder`)
- `-Release`: Build in release mode (default)
- `-Debug`: Build in debug mode
- `-Verbose`: Show verbose output
- `-Help`: Show help message

**Examples:**

```powershell
# Install to AppData (default)
.\scripts\install.ps1

# Install to Program Files
.\scripts\install.ps1 -Prefix "C:\Program Files\ricecoder"

# Debug build
.\scripts\install.ps1 -Debug

# Verbose output
.\scripts\install.ps1 -Verbose
```

#### Using the Installation Script (CMD)

```cmd
cd ricecoder
scripts\install.bat [OPTIONS]
```

**Options:**
- `--prefix PATH`: Installation prefix (default: `%LOCALAPPDATA%\ricecoder`)
- `--release`: Build in release mode (default)
- `--debug`: Build in debug mode
- `--verbose`: Show verbose output
- `--help`: Show help message

**Examples:**

```cmd
REM Install to AppData (default)
scripts\install.bat

REM Install to Program Files
scripts\install.bat --prefix "C:\Program Files\ricecoder"

REM Debug build
scripts\install.bat --debug
```

#### Manual Installation

```powershell
# Build
cd ricecoder
cargo build --release

# Create installation directory
$InstallDir = "$env:LOCALAPPDATA\ricecoder\bin"
New-Item -ItemType Directory -Path $InstallDir -Force

# Copy binaries
Copy-Item -Path "target\release\ricecoder*.exe" -Destination $InstallDir

# Add to PATH (permanent)
[Environment]::SetEnvironmentVariable(
    "PATH",
    "$env:PATH;$InstallDir",
    "User"
)
```

## Configuration

### Configuration Files

RiceCoder looks for configuration in this order:

1. **Project-level**: `.ricecoder/config.yaml` in project root
2. **User-level**: `~/.ricecoder/config.yaml` (Linux/macOS) or `%APPDATA%\ricecoder\config.yaml` (Windows)
3. **System-level**: `/etc/ricecoder/config.yaml` (Linux) or `C:\ProgramData\ricecoder\config.yaml` (Windows)
4. **Built-in defaults**: Compiled defaults

### Example Configuration

```yaml
# ~/.ricecoder/config.yaml

# Theme settings
theme:
  name: "dark"
  colors:
    primary: "#007acc"
    secondary: "#6a9955"

# Keybindings
keybindings:
  quit: "q"
  help: "?"
  execute: "Enter"

# Performance
performance:
  max_workers: 4
  timeout_ms: 30000

# Logging
logging:
  level: "info"
  format: "json"
```

### Environment Variables

Override configuration with environment variables:

```bash
# Linux/macOS
export RICECODER_THEME=dark
export RICECODER_LOG_LEVEL=debug

# Windows (PowerShell)
$env:RICECODER_THEME = "dark"
$env:RICECODER_LOG_LEVEL = "debug"

# Windows (CMD)
set RICECODER_THEME=dark
set RICECODER_LOG_LEVEL=debug
```

## Verification

### Check Installation

```bash
# Linux/macOS
ricecoder --version
ricecoder --help

# Windows
ricecoder.exe --version
ricecoder.exe --help
```

### Run Tests

```bash
cd ricecoder
cargo test --release
```

### Run Benchmarks

```bash
cd ricecoder
cargo bench
```

## Troubleshooting

### Build Failures

#### "Rust not found"

Install Rust from [https://rustup.rs/](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### "MSVC not found" (Windows)

Install Visual Studio Build Tools:

1. Download from [https://visualstudio.microsoft.com/downloads/](https://visualstudio.microsoft.com/downloads/)
2. Select "Desktop development with C++"
3. Install

#### "Linker error" (Linux)

Install development headers:

```bash
# Ubuntu/Debian
sudo apt-get install -y build-essential pkg-config libssl-dev

# Fedora/RHEL
sudo dnf install -y gcc make pkg-config openssl-devel

# Arch
sudo pacman -S base-devel pkg-config openssl
```

#### "Out of memory" during build

Reduce parallel jobs:

```bash
cargo build --release -j 2
```

### Runtime Issues

#### "Command not found"

Add installation directory to PATH:

**Linux/macOS:**
```bash
export PATH="$HOME/.local/bin:$PATH"
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
```

**Windows (PowerShell):**
```powershell
[Environment]::SetEnvironmentVariable(
    "PATH",
    "$env:PATH;$env:LOCALAPPDATA\ricecoder\bin",
    "User"
)
```

#### "Permission denied" (Linux/macOS)

Make binary executable:

```bash
chmod +x ~/.local/bin/ricecoder*
```

#### "Configuration not found"

Create configuration directory:

**Linux/macOS:**
```bash
mkdir -p ~/.ricecoder
cp config/default.yaml ~/.ricecoder/config.yaml
```

**Windows:**
```powershell
New-Item -ItemType Directory -Path "$env:APPDATA\ricecoder" -Force
Copy-Item -Path "config\default.yaml" -Destination "$env:APPDATA\ricecoder\config.yaml"
```

### Performance Issues

#### Slow startup

- Check system resources (CPU, memory)
- Reduce number of workers: `RICECODER_MAX_WORKERS=2`
- Use release build: `cargo build --release`

#### High memory usage

- Reduce worker count: `RICECODER_MAX_WORKERS=2`
- Increase timeout: `RICECODER_TIMEOUT_MS=60000`
- Check for memory leaks: `valgrind ricecoder`

## Uninstallation

### Linux/macOS

```bash
# Remove binaries
rm -f ~/.local/bin/ricecoder*

# Remove configuration
rm -rf ~/.ricecoder

# Remove documentation
rm -rf ~/.local/share/doc/ricecoder
```

### Windows (PowerShell)

```powershell
# Remove binaries
Remove-Item -Path "$env:LOCALAPPDATA\ricecoder\bin" -Recurse -Force

# Remove configuration
Remove-Item -Path "$env:APPDATA\ricecoder" -Recurse -Force

# Remove documentation
Remove-Item -Path "$env:LOCALAPPDATA\ricecoder\share" -Recurse -Force
```

### Windows (CMD)

```cmd
REM Remove binaries
rmdir /s /q "%LOCALAPPDATA%\ricecoder\bin"

REM Remove configuration
rmdir /s /q "%APPDATA%\ricecoder"

REM Remove documentation
rmdir /s /q "%LOCALAPPDATA%\ricecoder\share"
```

## Advanced Installation

### Custom Build Flags

```bash
# Build with specific features
cargo build --release --features "feature1,feature2"

# Build with optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Build with debug symbols
cargo build --release --debug-assertions
```

### Cross-Compilation

Build for different target:

```bash
# List available targets
rustup target list

# Install target
rustup target add x86_64-unknown-linux-gnu

# Build for target
cargo build --release --target x86_64-unknown-linux-gnu
```

### Docker Installation

```dockerfile
FROM rust:latest

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=0 /app/target/release/ricecoder* /usr/local/bin/

ENTRYPOINT ["ricecoder"]
```

Build and run:

```bash
docker build -t ricecoder .
docker run -it ricecoder --help
```

## Getting Help

- **Documentation**: [https://github.com/moabualruz/ricecoder/wiki](https://github.com/moabualruz/ricecoder/wiki)
- **Issues**: [https://github.com/moabualruz/ricecoder/issues](https://github.com/moabualruz/ricecoder/issues)
- **Discussions**: [https://github.com/moabualruz/ricecoder/discussions](https://github.com/moabualruz/ricecoder/discussions)

## See Also

- [README.md](./README.md) - Project overview
- [CONTRIBUTING.md](./CONTRIBUTING.md) - Contributing guidelines
- [SECURITY.md](./SECURITY.md) - Security policy
- [LICENSE.md](./LICENSE.md) - License information
