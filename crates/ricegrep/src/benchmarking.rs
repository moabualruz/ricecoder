use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sysinfo::System;
use tokio::sync::mpsc;

use crate::lexical::{Bm25IndexHandle, LexicalSearcher};
use crate::performance::{
    BenchmarkError, BenchmarkHarness, BenchmarkMode, BenchmarkQuery, BenchmarkResult,
};
use crate::vector::alerting::{AlertManager, AlertSeverity, MetricKind};

const HISTORY_FILENAME: &str = "benchmark-history.json";
const BASELINE_FILENAME: &str = "benchmark-baseline.json";
const LOAD_TEST_FILENAME: &str = "load-test-history.json";

const SUITE_MODES: [BenchmarkMode; 4] = [
    BenchmarkMode::Bm25,
    BenchmarkMode::Ann,
    BenchmarkMode::Hybrid,
    BenchmarkMode::Fallback,
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRecord {
    pub timestamp: DateTime<Utc>,
    pub mode: BenchmarkMode,
    pub result: BenchmarkResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestRecord {
    pub timestamp: DateTime<Utc>,
    pub result: LoadTestResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestResult {
    pub worker_count: usize,
    pub duration_seconds: f64,
    pub total_queries: usize,
    pub actual_qps: f64,
    pub median_latency_ms: f64,
    pub max_latency_ms: f64,
    pub cpu_percent: f32,
    pub memory_usage_bytes: u64,
}

#[derive(Debug)]
pub struct BenchmarkStorage {
    root: PathBuf,
}

impl BenchmarkStorage {
    pub fn new(root: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    fn history_path(&self) -> PathBuf {
        self.root.join(HISTORY_FILENAME)
    }

    fn baseline_path(&self) -> PathBuf {
        self.root.join(BASELINE_FILENAME)
    }

    fn load_test_path(&self) -> PathBuf {
        self.root.join(LOAD_TEST_FILENAME)
    }

    fn read_records<T>(&self, path: &Path) -> io::Result<Vec<T>>
    where
        T: DeserializeOwned,
    {
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }

    fn write_records<T>(&self, path: &Path, records: &[T]) -> io::Result<()>
    where
        T: Serialize,
    {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, records)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }

    pub fn persist(&self, record: &BenchmarkRecord) -> io::Result<()> {
        let mut history = self.read_records(&self.history_path())?;
        history.push(record.clone());
        self.write_records(&self.history_path(), &history)
    }

    pub fn baseline_map(&self) -> io::Result<HashMap<BenchmarkMode, BenchmarkRecord>> {
        let records = self.read_records(&self.baseline_path())?;
        Ok(records
            .into_iter()
            .map(|record: BenchmarkRecord| (record.mode, record))
            .collect())
    }

    pub fn update_baseline(
        &self,
        results: &HashMap<BenchmarkMode, BenchmarkResult>,
    ) -> io::Result<()> {
        let now = Utc::now();
        let records: Vec<BenchmarkRecord> = results
            .iter()
            .map(|(mode, result)| BenchmarkRecord {
                timestamp: now,
                mode: *mode,
                result: result.clone(),
            })
            .collect();
        self.write_records(&self.baseline_path(), &records)
    }

    pub fn persist_load_test(&self, record: &LoadTestRecord) -> io::Result<()> {
        let mut history = self.read_records(&self.load_test_path())?;
        history.push(record.clone());
        self.write_records(&self.load_test_path(), &history)
    }
}

#[derive(Debug, Clone)]
pub struct RegressionAlert {
    pub name: String,
    pub severity: AlertSeverity,
    pub metric: MetricKind,
    pub value: f64,
    pub threshold: f64,
    pub message: String,
}

#[derive(Debug)]
pub struct RegressionDetector {
    hybrid_delta: f64,
    fallback_delta: f64,
    degrade_tolerance: f64,
}

impl RegressionDetector {
    pub fn new(hybrid_delta: f64, fallback_delta: f64, degrade_tolerance: f64) -> Self {
        Self {
            hybrid_delta,
            fallback_delta,
            degrade_tolerance,
        }
    }

    pub fn default() -> Self {
        Self::new(0.15, 0.05, 0.02)
    }

    pub fn detect(
        &self,
        latest: &HashMap<BenchmarkMode, BenchmarkResult>,
        baseline: &HashMap<BenchmarkMode, BenchmarkRecord>,
    ) -> Vec<RegressionAlert> {
        let mut alerts = Vec::new();
        if let (Some(hybrid), Some(bm25)) = (
            latest.get(&BenchmarkMode::Hybrid),
            latest.get(&BenchmarkMode::Bm25),
        ) {
            let delta = hybrid.mrr - bm25.mrr;
            if delta < self.hybrid_delta {
                alerts.push(RegressionAlert {
                    name: "HybridQualityDelta".to_string(),
                    severity: AlertSeverity::Critical,
                    metric: MetricKind::BenchmarkMrr,
                    value: delta,
                    threshold: self.hybrid_delta,
                    message: format!(
                        "hybrid MRR delta dropped to {:.3} (target {:.3})",
                        delta, self.hybrid_delta
                    ),
                });
            }
        }
        if let (Some(fallback), Some(bm25)) = (
            latest.get(&BenchmarkMode::Fallback),
            latest.get(&BenchmarkMode::Bm25),
        ) {
            let delta = fallback.mrr - bm25.mrr;
            if delta < self.fallback_delta {
                alerts.push(RegressionAlert {
                    name: "FallbackQualityDelta".to_string(),
                    severity: AlertSeverity::Warning,
                    metric: MetricKind::BenchmarkMrr,
                    value: delta,
                    threshold: self.fallback_delta,
                    message: format!(
                        "fallback MRR delta dropped to {:.3} (target {:.3})",
                        delta, self.fallback_delta
                    ),
                });
            }
        }
        for (mode, latest_result) in latest {
            if let Some(previous) = baseline.get(mode) {
                if latest_result.mrr + self.degrade_tolerance < previous.result.mrr {
                    alerts.push(RegressionAlert {
                        name: format!("{mode:?}Regression"),
                        severity: AlertSeverity::Warning,
                        metric: MetricKind::BenchmarkMrr,
                        value: latest_result.mrr,
                        threshold: previous.result.mrr + self.degrade_tolerance,
                        message: format!(
                            "{:?} MRR dropped to {:.3} vs baseline {:.3}",
                            mode, latest_result.mrr, previous.result.mrr
                        ),
                    });
                }
            }
        }
        alerts
    }
}

pub struct BenchmarkCoordinator {
    index_dir: PathBuf,
    queries: Vec<BenchmarkQuery>,
    storage: BenchmarkStorage,
    detector: RegressionDetector,
    alert_manager: Arc<AlertManager>,
}

impl BenchmarkCoordinator {
    pub fn new(
        index_dir: PathBuf,
        benchmark_root: PathBuf,
        queries: Vec<BenchmarkQuery>,
        alert_manager: Arc<AlertManager>,
    ) -> io::Result<Self> {
        Ok(Self {
            index_dir,
            queries,
            storage: BenchmarkStorage::new(benchmark_root)?,
            detector: RegressionDetector::default(),
            alert_manager,
        })
    }

    fn harness(&self) -> BenchmarkHarness {
        BenchmarkHarness::new(self.index_dir.clone(), self.queries.clone())
    }

    pub fn run_mode(&self, mode: BenchmarkMode) -> Result<BenchmarkResult, BenchmarkError> {
        let result = self.harness().run(mode)?;
        let record = BenchmarkRecord {
            timestamp: Utc::now(),
            mode,
            result: result.clone(),
        };
        self.storage.persist(&record).map_err(BenchmarkError::Io)?;
        Ok(result)
    }

    pub fn run_suite(&self) -> Result<HashMap<BenchmarkMode, BenchmarkResult>, BenchmarkError> {
        let baseline = self.storage.baseline_map().map_err(BenchmarkError::Io)?;
        let mut results = HashMap::new();
        for mode in SUITE_MODES {
            let result = self.run_mode(mode)?;
            results.insert(mode, result);
        }
        for alert in self.detector.detect(&results, &baseline) {
            self.alert_manager.fire_custom_alert(
                &alert.name,
                alert.severity,
                alert.metric,
                Some(alert.value),
                alert.threshold,
                Some(alert.message.clone()),
            );
        }
        self.storage
            .update_baseline(&results)
            .map_err(BenchmarkError::Io)?;
        Ok(results)
    }

    pub async fn run_load_test(
        &self,
        worker_count: usize,
        duration: Duration,
    ) -> Result<LoadTestResult, BenchmarkError> {
        let workers = worker_count.max(1);
        let burst_duration = duration.max(Duration::from_secs(5));
        let deadline = Instant::now() + burst_duration;
        let (tx, mut rx) = mpsc::unbounded_channel::<Duration>();
        let mut handles = Vec::with_capacity(workers);
        for _ in 0..workers {
            let index_dir = self.index_dir.clone();
            let queries = self.queries.clone();
            let sender = tx.clone();
            let worker_deadline = deadline;
            handles.push(tokio::task::spawn_blocking(
                move || -> Result<(), BenchmarkError> {
                    let handle = Bm25IndexHandle::open(&index_dir)?;
                    let searcher = LexicalSearcher::new(handle);
                    let mut query_iter = queries.iter().cycle();
                    while Instant::now() < worker_deadline {
                        let query = query_iter.next().unwrap();
                        let start = Instant::now();
                        searcher.search(&query.query, query.k)?;
                        let elapsed = start.elapsed();
                        if sender.send(elapsed).is_err() {
                            break;
                        }
                    }
                    Ok(())
                },
            ));
        }
        drop(tx);
        let mut durations = Vec::new();
        while let Some(elapsed) = rx.recv().await {
            durations.push(elapsed);
        }
        for handle in handles {
            handle.await.map_err(|err| {
                BenchmarkError::Io(io::Error::new(io::ErrorKind::Interrupted, err.to_string()))
            })??;
        }
        let total_queries = durations.len();
        let actual_duration = burst_duration.as_secs_f64();
        let actual_qps = if actual_duration > 0.0 {
            total_queries as f64 / actual_duration
        } else {
            total_queries as f64
        };
        let median_latency_ms = median_duration_ms(&durations);
        let max_latency_ms = durations
            .iter()
            .map(|dur| dur.as_secs_f64() * 1_000.0)
            .fold(0.0, f64::max);
        let mut system = System::new_all();
        system.refresh_all();
        let cpu_percent = system.global_cpu_info().cpu_usage();
        let memory_usage_bytes = system.used_memory() * 1024;
        let result = LoadTestResult {
            worker_count: workers,
            duration_seconds: actual_duration,
            total_queries,
            actual_qps,
            median_latency_ms,
            max_latency_ms,
            cpu_percent,
            memory_usage_bytes,
        };
        self.storage
            .persist_load_test(&LoadTestRecord {
                timestamp: Utc::now(),
                result: result.clone(),
            })
            .map_err(BenchmarkError::Io)?;
        Ok(result)
    }
}

fn median_duration_ms(samples: &[Duration]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let mut ms: Vec<f64> = samples
        .iter()
        .map(|duration| duration.as_secs_f64() * 1_000.0)
        .collect();
    ms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = ms.len() / 2;
    if ms.len() % 2 == 0 {
        (ms[mid - 1] + ms[mid]) / 2.0
    } else {
        ms[mid]
    }
}
