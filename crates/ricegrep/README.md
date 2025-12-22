# RiceGrep

RiceGrep delivers the familiar ripgrep CLI ergonomics and performance while layering in AI-assisted search, structured indexing, and a first-class observability/operations surface that powers production-grade search services.

## Overview

- **Ripgrep-compatible core**: The legacy CLI mode mirrors ripgrep patterns, globbing, and exit codes so existing scripts and habits work without change.
- **Modern subcommands**: The `ricegrep search`, `watch`, `install`, `mcp`, `index`, and `process` subcommands unlock structured search, background indexing, and MCP integration.
- **AI-aware ranking**: Heuristic reranking, natural language understanding, and answer generation sit atop deterministic fallback logic so queries stay consistent even when AI helpers are unavailable.

## Key Capabilities

| Area | Highlights | Implementation Reference |
|------|------------|--------------------------|
| Search | Regex, literal, file filtering, context, watch, replace, dry-run, and quiet traps with steeped progress indicators | `projects/ricecoder/crates/ricegrep/src/search.rs`, `args.rs`, `replace.rs` |
| Observability & Monitoring | `/metrics`, `/metrics/history`, `/alerts`, `/health`, and Grafana-ready dashboards built on `VectorTelemetry`, `VectorMetrics`, and `AlertManager` | `projects/ricecoder/crates/ricegrep/src/vector/observability.rs`, `vector/alerting.rs`, `api/http.rs` |
| Administration | `/admin/command` executes reindex, optimize, clear-cache, and configuration overrides through `AdminToolset` | `projects/ricecoder/crates/ricegrep/src/admin.rs`, `api/http.rs` |
| Benchmarking & Regressions | `/benchmarks/run`, `run_performance_benchmarks.*`, and the `BenchmarkCoordinator`/`BenchmarkHarness` suite persist results and fire regression alerts via `RegressionDetector` | `projects/ricecoder/crates/ricegrep/src/benchmarking.rs`, `projects/ricecoder/crates/ricegrep/src/performance.rs` |

## Getting Started

1. Clone the RiceCoder workspace and build the `ricegrep` binary:
   ```bash
   git clone https://github.com/moabualruz/ricecoder.git
   cd ricecoder
   cargo build --bin ricegrep
   ```
2. Run core commands:
   ```bash
   target/release/ricegrep search "fn main" src/
   target/release/ricegrep search --ai-enhanced "list index commands"
   target/release/ricegrep watch --quiet src/
   ```
3. Optional: install CRC-coded assistant integrations (`ricegrep install claude-code`, etc.) to leverage MCP tooling.

## Observability & Alerting (Milestone v1.2)

- `/metrics` refreshes `SystemResourceSampler`, records CPU/memory/disk/network usage, updates `VectorMetrics`, and serializes the Prometheus family set for scraping. History entries are persisted via `MetricsStorage` before the `/metrics/history` endpoint returns aggregated buckets that honor the 90-day retention policy.
- `/alerts` polls `AlertManager::check_alerts`, which drives `AlertRule` evaluations built on `VectorTelemetrySnapshot` and `MetricKind`. `/alerts/{name}/ack` and `/alerts/{name}/resolve` surface acknowledgement metadata, actor notes, and triggers for `Notifier` and `RemediationHandler` chains.
- Grafana dashboards (system overview, component deep dive, business metrics) live under `projects/ricecoder/crates/ricegrep/dashboards/` and visualize the `ricegrep_vector_*` metrics emitted by the new observability stack.

## Administration & Reindexing

- `/admin/command` dispatches `AdminAction` requests through `AdminToolset`, allowing operators to reindex repositories, optimize BM25 handles, clear caches, and inject runtime config overrides. All actions emit `AdminCommandResponse` objects that include the summary (and `LexicalIngestStats` when available) for telemetry.
- The administrative helpers wire `LexicalIngestPipeline` into `VectorTelemetry`, ensuring ingestion statistics appear in both the atomic telemetry snapshot and Prometheus exposures.

## Benchmarking, Regression Detection, and Load Testing

- `BenchmarkHarness` computes recall (Recall@K, Precision@K, MRR, NDCG) for BM25, ANN, hybrid, and fallback modes. `BenchmarkCoordinator` persists each run, updates baseline records, and fires alerts through `RegressionDetector` when hybrid/fallback deltas slip past the defined thresholds.
- `/benchmarks/run` accepts a suite run request or individual mode invocation. The `run_performance_benchmarks.*` scripts share the same entry points for CI gating.
- Load testing uses `BenchmarkCoordinator::run_load_test`, writes `LoadTestRecord` artifacts, and exposes worker-level CPU/memory/latency telemetry for dashboards.

## Documentation & Contribution

- Ricegrep inherits the RiceCoder [license](LICENSE.md) and [contributing guide](CONTRIBUTING.md). Please follow the policies described there when adding features, tests, or docs.
- Detailed architecture, requirement, and design references live in `.kiro/specs/ricecode-ricegrep-ultrafast-hybrid-search/` and highlight how the v1.0–v1.2 milestones trace to the current code.
- Further user and developer docs live in `projects/ricecoder.wiki/RiceGrep.md` and the new `RiceGrep-Operations.md` documentation (see the wiki folder for the latest content).

## Running Tests & Benchmarks

```bash
cargo test -p ricegrep
cargo test -p ricegrep vector::observability vector::alerting
cargo bench --bin ricegrep_bench
./run_performance_benchmarks.sh
```

## Support & Feedback

- Report issues via the RiceCoder issue tracker.
- Join the community discussions (Discord/GitHub Discussions) for operations, monitoring, and benchmarking questions.

---

Ricegrep is part of the RiceCoder Ultrafast Hybrid Search stack. Follow the project-level roadmap and SSD docs to understand upcoming features and traceability mappings.
