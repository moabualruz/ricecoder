# ricecoder-performance

Performance monitoring and regression detection for RiceCoder.

## DDD Layer

**Layer**: Infrastructure (Monitoring)

### Responsibilities

- Performance metric collection
- Regression detection and alerting
- Baseline management
- Performance report generation
- System resource monitoring

### SOLID Analysis

| Principle | Score | Notes |
|-----------|-------|-------|
| SRP | ✅ | Clear separation of metrics, detection, reporting |
| OCP | ✅ | Extensible via new metrics and reporters |
| LSP | ✅ | Consistent metric interfaces |
| ISP | ✅ | Segregated concerns (collection, analysis, reporting) |
| DIP | ✅ | Depends on abstractions for metrics storage |

**Score**: 5/5

### Integration Points

| Component | Direction | Purpose |
|-----------|-----------|---------|
| ricecoder-monitoring | Complements | Performance metrics feed into monitoring |
| ricecoder-benchmark | Related | Benchmark results for regression detection |
| ricecoder-cli | Used by | Performance validation commands |

## Features

- **Metrics**: Startup time, memory usage, response latency
- **Baselines**: JSON-based baseline storage
- **Detection**: Configurable regression thresholds
- **Reports**: Markdown and JSON report formats

## CLI Usage

```bash
# Check for regressions
ricecoder-performance check-regression \
  --binary ./target/release/ricecoder \
  --baseline performance-baselines.json

# Update baselines
ricecoder-performance update-baseline \
  --binary ./target/release/ricecoder \
  --output performance-baselines.json
```

## Performance Targets

| Metric | Target | Threshold |
|--------|--------|-----------|
| Startup Time | < 3s | +20% regression |
| Response Time | < 500ms | +15% regression |
| Memory Usage | < 300MB | +25% regression |

## License

MIT
