# Performance Profiling and Optimization Guide

**Status**: Phase 4 - Performance Optimization (Task 24.1)

**Date**: December 5, 2025

**Purpose**: Guide for profiling and optimizing hot paths in ricecoder

---

## Overview

This guide documents the performance profiling infrastructure and optimization strategies for ricecoder. The goal is to ensure all operations meet the performance targets defined in NFR-1:

- CLI startup: < 2 seconds
- Code generation: < 30 seconds
- Template rendering: < 1 second
- File operations: < 5 seconds
- Large project support: 1000+ files

---

## Performance Targets (NFR-1)

| Operation | Target | Current | Status |
|-----------|--------|---------|--------|
| CLI startup | < 2s | TBD | 📋 To Profile |
| Config loading | < 500ms | TBD | 📋 To Profile |
| Provider init | < 1s | TBD | 📋 To Profile |
| Spec parsing | < 1s | TBD | 📋 To Profile |
| File operations | < 5s | TBD | 📋 To Profile |
| Code generation | < 30s | TBD | 📋 To Profile |
| Template rendering | < 1s | TBD | 📋 To Profile |

---

## Profiling Tools

### 1. Criterion Benchmarks

Criterion is used for micro-benchmarking and performance regression detection.

**Location**: `projects/ricecoder/benches/`

**Benchmarks**:
- `cli_startup_benchmarks.rs` - CLI startup and initialization
- `crates/ricecoder-permissions/benches/performance_benchmarks.rs` - Permission system

**Running Benchmarks**:

```bash
# Run all benchmarks
cargo bench --all

# Run specific benchmark
cargo bench --bench cli_startup_benchmarks

# Run with baseline comparison
cargo bench --bench cli_startup_benchmarks -- --baseline main

# Generate HTML reports
cargo bench --bench cli_startup_benchmarks -- --verbose
```

**Output**: Criterion generates HTML reports in `target/criterion/` with detailed statistics and graphs.

### 2. Flamegraph Profiling

Flamegraph shows where CPU time is spent in the call stack.

**Installation**:

```bash
# Install flamegraph
cargo install flamegraph

# Install perf (Linux only)
sudo apt-get install linux-tools-generic
```

**Running Flamegraph**:

```bash
# Profile CLI startup
cargo flamegraph --bin ricecoder-cli -- --help

# Profile specific command
cargo flamegraph --bin ricecoder-cli -- gen spec.yaml

# Profile with custom options
cargo flamegraph --bin ricecoder-cli --freq 99 -- chat "hello"
```

**Output**: Generates `flamegraph.svg` showing call stack with time spent in each function.

**Interpreting Results**:
- Width = time spent in function
- Height = call stack depth
- Wider boxes = more time spent
- Look for unexpectedly wide boxes as optimization targets

### 3. Valgrind Memory Profiling

Valgrind detects memory leaks and excessive allocations.

**Installation**:

```bash
# Linux
sudo apt-get install valgrind

# macOS
brew install valgrind
```

**Running Valgrind**:

```bash
# Memory profiling
valgrind --tool=massif --massif-out-file=massif.out ./target/debug/ricecoder-cli --help

# Generate report
ms_print massif.out

# Leak detection
valgrind --leak-check=full --show-leak-kinds=all ./target/debug/ricecoder-cli --help
```

**Output**: Shows memory usage over time and identifies memory leaks.

### 4. Perf (Linux)

Perf is the Linux performance profiler.

**Installation**:

```bash
sudo apt-get install linux-tools-generic
```

**Running Perf**:

```bash
# Record performance data
perf record -g ./target/debug/ricecoder-cli --help

# Generate report
perf report

# Flamegraph from perf
perf script | stackcollapse-perf.pl | flamegraph.pl > perf_flamegraph.svg
```

### 5. Cargo Flamegraph with Perf

Combines cargo and perf for easy profiling.

**Running**:

```bash
# Profile with flamegraph
cargo flamegraph --bin ricecoder-cli -- --help

# Profile release build
cargo flamegraph --release --bin ricecoder-cli -- --help
```

---

## Hot Paths to Profile

### 1. CLI Startup Path

**Flow**:
1. Binary starts
2. Parse CLI arguments (clap)
3. Initialize logging
4. Load configuration
5. Initialize provider
6. Execute command

**Profiling**:

```bash
# Profile help command (minimal work)
cargo flamegraph --bin ricecoder-cli -- --help

# Profile with timing
time cargo run --release -- --help
```

**Optimization Opportunities**:
- Lazy load providers (only initialize when needed)
- Cache parsed configuration
- Defer non-essential initialization

### 2. Configuration Loading Path

**Flow**:
1. Load global config from `~/.ricecoder/config.yaml`
2. Load project config from `.agent/config.yaml`
3. Merge configurations
4. Validate merged config

**Profiling**:

```bash
# Profile config loading
cargo flamegraph --bin ricecoder-cli -- config list
```

**Optimization Opportunities**:
- Cache parsed configs with TTL
- Lazy load config sections
- Parallel config loading

### 3. Provider Initialization Path

**Flow**:
1. Load provider config
2. Initialize HTTP client
3. Validate credentials
4. Test connection

**Profiling**:

```bash
# Profile provider initialization
cargo flamegraph --bin ricecoder-cli -- chat "test"
```

**Optimization Opportunities**:
- Lazy initialize HTTP clients
- Cache provider instances
- Defer credential validation

### 4. Spec Parsing Path

**Flow**:
1. Read spec file
2. Parse YAML/Markdown
3. Validate spec structure
4. Build spec context

**Profiling**:

```bash
# Profile spec parsing
cargo flamegraph --bin ricecoder-cli -- gen large_spec.yaml
```

**Optimization Opportunities**:
- Cache parsed specs
- Lazy parse spec sections
- Parallel parsing for large specs

### 5. File Operations Path

**Flow**:
1. Read file
2. Create backup
3. Write file
4. Update git

**Profiling**:

```bash
# Profile file operations
cargo flamegraph --bin ricecoder-cli -- gen spec.yaml
```

**Optimization Opportunities**:
- Async file I/O
- Batch git operations
- Streaming for large files

---

## Optimization Strategies

### 1. Lazy Initialization

Defer expensive initialization until needed.

**Example**: Providers

```rust
// Before: Initialize all providers on startup
let providers = vec![
    OpenAiProvider::new()?,
    AnthropicProvider::new()?,
    OllamaProvider::new()?,
];

// After: Initialize only when needed
let provider = match provider_name {
    "openai" => OpenAiProvider::new()?,
    "anthropic" => AnthropicProvider::new()?,
    "ollama" => OllamaProvider::new()?,
    _ => return Err("Unknown provider"),
};
```

### 2. Caching

Cache expensive computations.

**Example**: Configuration

```rust
// Before: Parse config every time
fn get_config() -> Result<Config> {
    let yaml = std::fs::read_to_string("config.yaml")?;
    serde_yaml::from_str(&yaml)
}

// After: Cache parsed config
lazy_static::lazy_static! {
    static ref CONFIG_CACHE: Mutex<Option<Config>> = Mutex::new(None);
}

fn get_config() -> Result<Config> {
    let mut cache = CONFIG_CACHE.lock().unwrap();
    if let Some(config) = cache.as_ref() {
        return Ok(config.clone());
    }
    
    let yaml = std::fs::read_to_string("config.yaml")?;
    let config = serde_yaml::from_str(&yaml)?;
    *cache = Some(config.clone());
    Ok(config)
}
```

### 3. Async I/O

Use async operations for I/O-bound tasks.

**Example**: File reading

```rust
// Before: Blocking I/O
fn read_file(path: &str) -> Result<String> {
    std::fs::read_to_string(path)
}

// After: Async I/O
async fn read_file(path: &str) -> Result<String> {
    tokio::fs::read_to_string(path).await
}
```

### 4. Streaming

Stream large data instead of loading into memory.

**Example**: Large file processing

```rust
// Before: Load entire file into memory
fn process_file(path: &str) -> Result<()> {
    let content = std::fs::read_to_string(path)?;
    for line in content.lines() {
        process_line(line)?;
    }
    Ok(())
}

// After: Stream file line by line
fn process_file(path: &str) -> Result<()> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        process_line(&line?)?;
    }
    Ok(())
}
```

### 5. Parallel Processing

Use parallelism for CPU-bound tasks.

**Example**: Spec parsing

```rust
// Before: Sequential parsing
fn parse_specs(specs: Vec<&str>) -> Result<Vec<Spec>> {
    specs.iter().map(|s| parse_spec(s)).collect()
}

// After: Parallel parsing
fn parse_specs(specs: Vec<&str>) -> Result<Vec<Spec>> {
    use rayon::prelude::*;
    specs.par_iter().map(|s| parse_spec(s)).collect()
}
```

---

## Profiling Workflow

### Step 1: Establish Baseline

```bash
# Run benchmarks to establish baseline
cargo bench --all -- --baseline main

# Record baseline results
cp -r target/criterion target/criterion-baseline
```

### Step 2: Profile Hot Path

```bash
# Generate flamegraph for hot path
cargo flamegraph --release --bin ricecoder-cli -- <command>

# Analyze flamegraph.svg
# Look for unexpectedly wide boxes
```

### Step 3: Identify Bottleneck

- Look for functions taking > 10% of time
- Check for unnecessary allocations
- Look for blocking I/O operations
- Check for redundant computations

### Step 4: Implement Optimization

- Apply optimization strategy
- Ensure correctness with tests
- Verify no regressions

### Step 5: Measure Improvement

```bash
# Run benchmarks again
cargo bench --all -- --baseline main

# Compare results
# Should see improvement in target metric
```

### Step 6: Document Results

- Record before/after metrics
- Document optimization applied
- Update performance targets if needed

---

## Performance Monitoring

### Continuous Benchmarking

Run benchmarks in CI/CD to detect regressions:

```yaml
# .github/workflows/benchmark.yml
name: Benchmark
on: [push, pull_request]
jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo bench --all
```

### Performance Regression Detection

Compare benchmarks across commits:

```bash
# Compare with previous commit
cargo bench --all -- --baseline main

# Compare with specific commit
git checkout <commit>
cargo bench --all -- --baseline old
git checkout -
cargo bench --all -- --baseline new
```

---

## Common Performance Issues

### 1. Excessive Allocations

**Symptom**: High memory usage, slow performance

**Solution**: Use references, avoid cloning, use `&str` instead of `String`

### 2. Blocking I/O

**Symptom**: CLI hangs, slow response times

**Solution**: Use async I/O with tokio

### 3. Redundant Computations

**Symptom**: Same computation repeated multiple times

**Solution**: Cache results, use memoization

### 4. Large Data Structures

**Symptom**: High memory usage, slow serialization

**Solution**: Stream data, use lazy evaluation

### 5. Inefficient Algorithms

**Symptom**: Slow performance with large inputs

**Solution**: Use better algorithms, add indexing

---

## Performance Checklist

Before releasing:

- [ ] All benchmarks pass
- [ ] No performance regressions
- [ ] CLI startup < 2 seconds
- [ ] Config loading < 500ms
- [ ] Provider init < 1 second
- [ ] Spec parsing < 1 second
- [ ] File operations < 5 seconds
- [ ] Memory usage reasonable (< 100MB for typical operations)
- [ ] No memory leaks detected
- [ ] Flamegraph shows no unexpected hot spots

---

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Flamegraph Guide](https://www.brendangregg.com/flamegraphs.html)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Valgrind Manual](https://valgrind.org/docs/manual/)

---

## Next Steps

1. Run baseline benchmarks (Task 24.1)
2. Profile hot paths with flamegraph (Task 24.1)
3. Implement caching strategies (Task 24.2)
4. Optimize memory usage (Task 24.3)
5. Verify improvements with benchmarks

---

*Last updated: December 5, 2025*
