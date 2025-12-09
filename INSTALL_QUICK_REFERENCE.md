# RiceCoder Installation Quick Reference

## One-Liner Installation

### Linux/macOS (via curl)
```bash
curl -fsSL https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install | bash
```

Or directly with install.sh:
```bash
curl -fsSL https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install.sh | bash
```

### Windows (PowerShell via curl)
```powershell
iex (New-Object Net.WebClient).DownloadString('https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install.ps1')
```

Or with newer PowerShell (7+):
```powershell
iex (Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install.ps1').Content
```

## Platform-Specific Quick Start

### Linux

```bash
# Prerequisites
sudo apt-get install -y build-essential pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone and install
git clone https://github.com/moabualruz/ricecoder.git
cd ricecoder
chmod +x scripts/install.sh
./scripts/install.sh

# Add to PATH
export PATH="$HOME/.local/bin:$PATH"
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
```

### macOS

```bash
# Prerequisites
xcode-select --install

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone and install
git clone https://github.com/moabualruz/ricecoder.git
cd ricecoder
chmod +x scripts/install.sh
./scripts/install.sh

# Add to PATH
export PATH="$HOME/.local/bin:$PATH"
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zprofile
```

### Windows (PowerShell)

```powershell
# Prerequisites
# Download and install Visual Studio Build Tools from:
# https://visualstudio.microsoft.com/downloads/

# Install Rust
iex (New-Object Net.WebClient).DownloadString('https://win.rustup.rs/')

# Clone and install
git clone https://github.com/moabualruz/ricecoder.git
cd ricecoder
.\scripts\install.ps1

# Add to PATH (restart PowerShell after)
[Environment]::SetEnvironmentVariable(
    "PATH",
    "$env:PATH;$env:LOCALAPPDATA\ricecoder\bin",
    "User"
)
```

### Windows (CMD)

```cmd
REM Prerequisites
REM Download and install Visual Studio Build Tools from:
REM https://visualstudio.microsoft.com/downloads/

REM Install Rust
REM Download from https://win.rustup.rs/ and run installer

REM Clone and install
git clone https://github.com/moabualruz/ricecoder.git
cd ricecoder
scripts\install.bat

REM Add to PATH (restart CMD after)
setx PATH "%PATH%;%LOCALAPPDATA%\ricecoder\bin"
```

## Installation Options

### Standard Installation (Release Build)

```bash
# Linux/macOS
./scripts/install.sh

# Windows (PowerShell)
.\scripts\install.ps1

# Windows (CMD)
scripts\install.bat
```

### Custom Installation Prefix

```bash
# Linux/macOS
./scripts/install.sh --prefix /usr/local

# Windows (PowerShell)
.\scripts\install.ps1 -Prefix "C:\Program Files\ricecoder"

# Windows (CMD)
scripts\install.bat --prefix "C:\Program Files\ricecoder"
```

### Debug Build

```bash
# Linux/macOS
./scripts/install.sh --debug

# Windows (PowerShell)
.\scripts\install.ps1 -Debug

# Windows (CMD)
scripts\install.bat --debug
```

### Verbose Output

```bash
# Linux/macOS
./scripts/install.sh --verbose

# Windows (PowerShell)
.\scripts\install.ps1 -Verbose

# Windows (CMD)
scripts\install.bat --verbose
```

## Verification

```bash
# Check version
ricecoder --version

# Show help
ricecoder --help

# Run tests
cargo test --release

# Run benchmarks
cargo bench
```

## Common Issues & Solutions

| Issue | Solution |
|-------|----------|
| **Rust not found** | Install from https://rustup.rs/ |
| **Build fails** | Update Rust: `rustup update` |
| **Command not found** | Add to PATH (see platform guides above) |
| **Permission denied** | Make executable: `chmod +x ~/.local/bin/ricecoder*` |
| **MSVC not found** | Install Visual Studio Build Tools |
| **Out of memory** | Reduce jobs: `cargo build --release -j 2` |

## Installation Locations

### Linux/macOS (Default)

- **Binaries**: `~/.local/bin/`
- **Config**: `~/.ricecoder/`
- **Docs**: `~/.local/share/doc/ricecoder/`

### Windows (Default)

- **Binaries**: `%LOCALAPPDATA%\ricecoder\bin\`
- **Config**: `%APPDATA%\ricecoder\`
- **Docs**: `%LOCALAPPDATA%\ricecoder\share\doc\ricecoder\`

## Uninstall

### Linux/macOS

```bash
rm -rf ~/.local/bin/ricecoder*
rm -rf ~/.ricecoder
rm -rf ~/.local/share/doc/ricecoder
```

### Windows (PowerShell)

```powershell
Remove-Item -Path "$env:LOCALAPPDATA\ricecoder" -Recurse -Force
Remove-Item -Path "$env:APPDATA\ricecoder" -Recurse -Force
```

### Windows (CMD)

```cmd
rmdir /s /q "%LOCALAPPDATA%\ricecoder"
rmdir /s /q "%APPDATA%\ricecoder"
```

## Next Steps

1. **Verify Installation**: `ricecoder --version`
2. **Read Documentation**: `ricecoder --help`
3. **Configure**: Create `~/.ricecoder/config.yaml`
4. **Get Started**: Follow the Quick Start guide

## Support

- **Documentation**: https://github.com/moabualruz/ricecoder/wiki
- **Issues**: https://github.com/moabualruz/ricecoder/issues
- **Discussions**: https://github.com/moabualruz/ricecoder/discussions

## See Also

- [INSTALLATION.md](./INSTALLATION.md) - Comprehensive installation guide
- [README.md](./README.md) - Project overview
- [CONTRIBUTING.md](./CONTRIBUTING.md) - Contributing guidelines
