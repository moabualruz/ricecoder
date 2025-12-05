# LSP Integration Troubleshooting Guide

This guide provides solutions for common issues with RiceCoder's LSP integration.

## Quick Diagnostics

Before troubleshooting, gather diagnostic information:

```bash
# Check RiceCoder version
ricecoder --version

# Check LSP server version
ricecoder lsp --version

# Test LSP server startup
ricecoder lsp start

# Check logs with debug level
RICECODER_LSP_LOG_LEVEL=debug ricecoder lsp start
```

## Common Issues and Solutions

### Server Issues

#### Issue: LSP server fails to start

**Error Messages**:
- "Command not found: ricecoder"
- "LSP server exited with code 1"
- "Failed to initialize server"

**Diagnosis**:
```bash
# Check if ricecoder is installed
which ricecoder

# Check if ricecoder is in PATH
echo $PATH

# Try running ricecoder directly
ricecoder --help
```

**Solutions**:

1. **Install RiceCoder**:
   ```bash
   # Using cargo
   cargo install ricecoder
   
   # Or build from source
   git clone https://github.com/moabualruz/ricecoder.git
   cd ricecoder
   cargo install --path .
   ```

2. **Add to PATH**:
   ```bash
   # Add to ~/.bashrc or ~/.zshrc
   export PATH="$HOME/.cargo/bin:$PATH"
   ```

3. **Check permissions**:
   ```bash
   # Ensure ricecoder binary is executable
   chmod +x ~/.cargo/bin/ricecoder
   ```

#### Issue: Server crashes immediately

**Error Messages**:
- "LSP server exited unexpectedly"
- "Segmentation fault"
- "Out of memory"

**Diagnosis**:
```bash
# Run with debug logging
RICECODER_LSP_LOG_LEVEL=debug ricecoder lsp start 2>&1 | head -50

# Check system resources
free -h
df -h
```

**Solutions**:

1. **Check system resources**:
   ```bash
   # Ensure sufficient memory
   free -h
   
   # Ensure sufficient disk space
   df -h
   ```

2. **Reduce cache size**:
   ```bash
   export RICECODER_LSP_CACHE_SIZE=50
   ricecoder lsp start
   ```

3. **Increase timeout**:
   ```bash
   export RICECODER_LSP_TIMEOUT_MS=15000
   ricecoder lsp start
   ```

4. **Check for corrupted cache**:
   ```bash
   # Clear cache
   rm -rf ~/.ricecoder/cache
   ricecoder lsp start
   ```

#### Issue: Server hangs or becomes unresponsive

**Symptoms**:
- IDE shows "LSP server not responding"
- Requests timeout
- Server uses 100% CPU

**Diagnosis**:
```bash
# Check if server is running
ps aux | grep ricecoder

# Check CPU usage
top -p $(pgrep ricecoder)

# Check memory usage
ps aux | grep ricecoder | awk '{print $6}'
```

**Solutions**:

1. **Increase timeout**:
   ```bash
   export RICECODER_LSP_TIMEOUT_MS=30000
   ricecoder lsp start
   ```

2. **Reduce analysis scope**:
   - Close large files
   - Disable diagnostics for large files
   - Split large files into smaller ones

3. **Restart server**:
   - Close IDE
   - Kill any running ricecoder processes: `pkill ricecoder`
   - Restart IDE

### Analysis Issues

#### Issue: Diagnostics are not showing

**Symptoms**:
- No errors or warnings appear
- Diagnostics panel is empty
- Code issues are not highlighted

**Diagnosis**:
```bash
# Check if diagnostics are enabled
grep -r "diagnostics" ~/.ricecoder/

# Check file language detection
RICECODER_LSP_LOG_LEVEL=debug ricecoder lsp start 2>&1 | grep -i language

# Test with a simple file
echo 'let x = 1;' > test.rs
```

**Solutions**:

1. **Check file extension**:
   - Rust: `.rs`
   - TypeScript: `.ts`
   - Python: `.py`

2. **Enable diagnostics in configuration**:
   ```yaml
   # ~/.ricecoder/lsp.yaml
   lsp:
     diagnostics:
       enabled: true
   ```

3. **Check language-specific settings**:
   ```yaml
   languages:
     rust:
       diagnostics: true
   ```

4. **Verify file is recognized**:
   ```bash
   # Check logs for language detection
   RICECODER_LSP_LOG_LEVEL=debug ricecoder lsp start 2>&1 | grep "Detected language"
   ```

#### Issue: Hover information is not showing

**Symptoms**:
- Hovering shows no information
- Hover popup is empty
- Type information is missing

**Diagnosis**:
```bash
# Check if hover is enabled
grep -r "hover" ~/.ricecoder/

# Check logs for hover errors
RICECODER_LSP_LOG_LEVEL=debug ricecoder lsp start 2>&1 | grep -i hover

# Test with a simple symbol
echo 'fn test() {}' > test.rs
```

**Solutions**:

1. **Enable hover in configuration**:
   ```yaml
   # ~/.ricecoder/lsp.yaml
   lsp:
     hover_provider: true
   ```

2. **Check symbol recognition**:
   - Hover over function names
   - Hover over type names
   - Hover over variable names

3. **Verify semantic analysis**:
   ```bash
   # Check logs for analysis errors
   RICECODER_LSP_LOG_LEVEL=debug ricecoder lsp start 2>&1 | grep -i "analysis\|semantic"
   ```

#### Issue: Code actions are not available

**Symptoms**:
- Quick fix menu is empty
- No suggestions appear
- Code action command fails

**Diagnosis**:
```bash
# Check if code actions are enabled
grep -r "code_action" ~/.ricecoder/

# Check logs for code action errors
RICECODER_LSP_LOG_LEVEL=debug ricecoder lsp start 2>&1 | grep -i "code_action"

# Verify diagnostics are showing
# (code actions require diagnostics)
```

**Solutions**:

1. **Enable code actions in configuration**:
   ```yaml
   # ~/.ricecoder/lsp.yaml
   lsp:
     code_action_provider: true
   ```

2. **Verify diagnostics are showing**:
   - Code actions only appear for identified issues
   - Check that diagnostics are enabled and showing

3. **Check language support**:
   - Rust: Full support
   - TypeScript: Full support
   - Python: Full support

### Performance Issues

#### Issue: Analysis is slow

**Symptoms**:
- Diagnostics take a long time to appear
- IDE feels sluggish
- Hover information is delayed

**Diagnosis**:
```bash
# Check file size
wc -l large_file.rs

# Check cache hit rate
RICECODER_LSP_LOG_LEVEL=debug ricecoder lsp start 2>&1 | grep -i "cache"

# Monitor performance
time ricecoder lsp start
```

**Solutions**:

1. **Increase cache size**:
   ```bash
   export RICECODER_LSP_CACHE_SIZE=500
   ricecoder lsp start
   ```

2. **Increase timeout**:
   ```bash
   export RICECODER_LSP_TIMEOUT_MS=15000
   ricecoder lsp start
   ```

3. **Reduce file size**:
   - Split large files into smaller modules
   - Close files you're not working on

4. **Disable unused features**:
   ```yaml
   languages:
     rust:
       diagnostics: true
       code_actions: false  # Disable if not needed
   ```

#### Issue: Memory usage is high

**Symptoms**:
- LSP server uses lots of memory
- System becomes slow
- Out of memory errors

**Diagnosis**:
```bash
# Check memory usage
ps aux | grep ricecoder | awk '{print $6}'

# Monitor memory over time
watch -n 1 'ps aux | grep ricecoder | awk "{print \$6}"'

# Check for memory leaks
valgrind ricecoder lsp start
```

**Solutions**:

1. **Reduce cache size**:
   ```bash
   export RICECODER_LSP_CACHE_SIZE=50
   ricecoder lsp start
   ```

2. **Close large files**:
   - Large files (>1MB) use significant memory
   - Close files you're not actively editing

3. **Restart server periodically**:
   - Close IDE
   - Restart IDE to clear memory

4. **Check for large projects**:
   - Very large projects may require more memory
   - Consider working on smaller subsets

### Language-Specific Issues

#### Rust Issues

**Issue: Rust diagnostics are not accurate**

**Solutions**:
1. Ensure Rust toolchain is installed: `rustc --version`
2. Check that Cargo.toml is valid
3. Run `cargo check` to verify project compiles

**Issue: Rust symbols are not recognized**

**Solutions**:
1. Ensure file has `.rs` extension
2. Check that module structure is correct
3. Verify imports are correct

#### TypeScript Issues

**Issue: TypeScript diagnostics are not showing**

**Solutions**:
1. Ensure file has `.ts` extension
2. Check that tsconfig.json is valid
3. Verify TypeScript is installed: `tsc --version`

**Issue: TypeScript imports are not resolved**

**Solutions**:
1. Check that import paths are correct
2. Verify files exist at import paths
3. Check tsconfig.json paths configuration

#### Python Issues

**Issue: Python diagnostics are not showing**

**Solutions**:
1. Ensure file has `.py` extension
2. Check that Python is installed: `python --version`
3. Verify file syntax is valid

**Issue: Python imports are not resolved**

**Solutions**:
1. Check that import paths are correct
2. Verify files exist at import paths
3. Check PYTHONPATH environment variable

### IDE-Specific Issues

#### VS Code Issues

**Issue: Extension doesn't connect to LSP server**

**Solutions**:
1. Check extension settings: `ricecoder.lsp.enabled`
2. Verify command path: `ricecoder.lsp.command`
3. Check extension logs: View → Output → RiceCoder

**Issue: Diagnostics don't appear in VS Code**

**Solutions**:
1. Check Problems panel: View → Problems
2. Verify file language is detected: Bottom right corner
3. Check extension settings for language

#### Neovim Issues

**Issue: LSP client doesn't connect**

**Solutions**:
1. Check lsp-config: `:LspInfo`
2. Verify command path in config
3. Check logs: `:LspLog`

**Issue: Diagnostics don't appear in Neovim**

**Solutions**:
1. Check diagnostic configuration
2. Verify signs are enabled: `vim.diagnostic.config()`
3. Check virtual text settings

#### Emacs Issues

**Issue: LSP mode doesn't start**

**Solutions**:
1. Check lsp-mode configuration
2. Verify command path in lsp-register-client
3. Check lsp-mode logs: `lsp-log`

**Issue: Diagnostics don't appear in Emacs**

**Solutions**:
1. Check lsp-ui configuration
2. Verify flycheck is installed
3. Check diagnostic display settings

## Debugging

### Enable Debug Logging

```bash
# Set debug level
export RICECODER_LSP_LOG_LEVEL=debug

# Start server with debug output
ricecoder lsp start 2>&1 | tee lsp-debug.log

# Analyze logs
grep -i error lsp-debug.log
grep -i warning lsp-debug.log
```

### Collect Diagnostic Information

```bash
# Create diagnostic bundle
mkdir ricecoder-diagnostics
cd ricecoder-diagnostics

# Collect system info
uname -a > system.txt
free -h >> system.txt
df -h >> system.txt

# Collect RiceCoder info
ricecoder --version > ricecoder.txt
which ricecoder >> ricecoder.txt

# Collect logs
RICECODER_LSP_LOG_LEVEL=debug ricecoder lsp start 2>&1 | head -1000 > lsp.log

# Collect configuration
cp ~/.ricecoder/lsp.yaml . 2>/dev/null || echo "No config file"

# Create archive
tar -czf ricecoder-diagnostics.tar.gz *
```

### Test Individual Components

```bash
# Test semantic analysis
echo 'fn test() {}' > test.rs
ricecoder analyze test.rs

# Test diagnostics
ricecoder diagnose test.rs

# Test code actions
ricecoder actions test.rs
```

## Getting Help

If you can't resolve the issue:

1. **Collect diagnostic information** (see above)
2. **Check the logs** for error messages
3. **Search GitHub issues** for similar problems
4. **Open a new issue** with:
   - Diagnostic bundle
   - Reproduction steps
   - Expected vs actual behavior
   - IDE and version information

## Related Documentation

- **LSP Integration Guide**: `LSP_INTEGRATION_GUIDE.md`
- **API Documentation**: `README.md`
- **Requirements**: `.kiro/specs/ricecoder-lsp/requirements.md`
- **Design**: `.kiro/specs/ricecoder-lsp/design.md`

## License

Part of the RiceCoder project. See LICENSE for details.
