use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Instant;
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tempfile;

// ============================================================================
// Benchmark 6: Response Time Monitoring
// ============================================================================
// Validates: Typical operations complete in < 500ms (end-to-end including MCP)

fn benchmark_response_times(c: &mut Criterion) {
    let mut group = c.benchmark_group("response_times");
    group.sample_size(50);
    group.measurement_time(std::time::Duration::from_secs(60));

    // Build the ricecoder binary first
    let build_result = Command::new("cargo")
        .args(&["build", "--release", "--bin", "ricecoder"])
        .current_dir(env!("CARGO_MANIFEST_DIR").rsplitn(2, "/crates/").next().unwrap_or("."))
        .status();

    if !build_result.map(|s| s.success()).unwrap_or(false) {
        panic!("Failed to build ricecoder binary for benchmarking");
    }

    let binary_path = format!("{}/target/release/ricecoder",
        env!("CARGO_MANIFEST_DIR").rsplitn(2, "/crates/").next().unwrap_or("."));

    group.bench_function("config_command", |b| {
        b.iter(|| {
            let start = Instant::now();
            let result = Command::new(&binary_path)
                .args(&["config", "--list"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .unwrap_or_else(|_| panic!("Failed to execute config command"));
            assert!(result.success());
            let elapsed = start.elapsed();
            // Assert baseline: < 500ms for responses
            assert!(elapsed < std::time::Duration::from_millis(500),
                "Response time exceeded 500ms baseline: {:?}", elapsed);
            black_box(elapsed);
        });
    });

    group.bench_function("help_command", |b| {
        b.iter(|| {
            let start = Instant::now();
            let result = Command::new(&binary_path)
                .arg("--help")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .unwrap_or_else(|_| panic!("Failed to execute help command"));
            assert!(result.success());
            let elapsed = start.elapsed();
            // Assert baseline: < 500ms for responses
            assert!(elapsed < std::time::Duration::from_millis(500),
                "Response time exceeded 500ms baseline: {:?}", elapsed);
            black_box(elapsed);
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark 10: Enterprise Workload Performance
// ============================================================================
// Validates: Large-scale operations complete within reasonable time limits

fn benchmark_enterprise_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("enterprise_workload");
    group.sample_size(5);
    group.measurement_time(std::time::Duration::from_secs(180));

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // Create enterprise-scale project (500+ files, 50K+ lines)
    create_enterprise_project(&temp_dir, 50, 1000); // 50 crates, ~1000 lines each

    group.bench_function("analyze_enterprise_project", |b| {
        b.iter(|| {
            let start = Instant::now();
            // Simulate enterprise project analysis
            let analysis_result = analyze_enterprise_project(temp_dir.path());
            let elapsed = start.elapsed();

            // Assert baselines for enterprise workload
            assert!(elapsed < std::time::Duration::from_secs(30),
                "Enterprise project analysis exceeded 30s baseline: {:?}", elapsed);
            assert!(analysis_result.total_files > 500,
                "Enterprise project should have >500 files, got {}", analysis_result.total_files);
            assert!(analysis_result.total_lines > 50000,
                "Enterprise project should have >50K lines, got {}", analysis_result.total_lines);

            black_box((analysis_result, elapsed));
        });
    });

    group.bench_function("resource_consumption_tracking", |b| {
        b.iter(|| {
            let start = Instant::now();
            // Simulate resource-intensive operations
            let mut resources = Vec::new();

            // Simulate concurrent file processing
            for i in 0..100 {
                let file_path = temp_dir.path().join(format!("concurrent_file_{}.rs", i));
                let content = format!("// Concurrent file {}\nfn func_{}() {{}}\n", i, i);
                std::fs::write(&file_path, content).expect("Failed to write concurrent file");
                resources.push(content);
            }

            // Simulate processing all files
            let processed: Vec<_> = resources.iter().map(|content| {
                content.lines().count()
            }).collect();

            let elapsed = start.elapsed();
            // Assert resource consumption baseline
            assert!(elapsed < std::time::Duration::from_secs(10),
                "Resource consumption tracking exceeded 10s baseline: {:?}", elapsed);
            assert!(processed.len() == 100,
                "Should process 100 files, got {}", processed.len());

            black_box((processed, elapsed));
        });
    });

    group.finish();
}

fn create_enterprise_project(base_dir: &tempfile::TempDir, num_crates: usize, lines_per_file: usize) {
    for crate_num in 0..num_crates {
        let crate_dir = base_dir.path().join(format!("enterprise_crate_{}", crate_num));
        std::fs::create_dir_all(&crate_dir).expect("Failed to create enterprise crate dir");

        // Create Cargo.toml
        let cargo_toml = format!(r#"
[package]
name = "enterprise-crate-{}"
version = "1.0.0"
edition = "2021"

[dependencies]
serde = "1.0"
tokio = "1.0"
axum = "0.6"
sqlx = "0.7"
redis = "0.23"
"#, crate_num);

        std::fs::write(crate_dir.join("Cargo.toml"), cargo_toml)
            .expect("Failed to write enterprise Cargo.toml");

        // Create src directory
        let src_dir = crate_dir.join("src");
        std::fs::create_dir_all(&src_dir).expect("Failed to create enterprise src dir");

        // Create multiple source files
        for file_num in 0..10 {
            let file_path = src_dir.join(format!("module_{}.rs", file_num));
            let mut content = format!("// Enterprise module {} for crate {}\n\n", file_num, crate_num);

            // Generate lines of code
            for line_num in 0..(lines_per_file / 10) {
                content.push_str(&format!("/// Documentation for function {}\n", line_num));
                content.push_str(&format!("pub fn enterprise_function_{}_{}() -> Result<(), Box<dyn std::error::Error>> {{\n",
                    crate_num, line_num));
                content.push_str(&format!("    // Enterprise logic here\n"));
                content.push_str(&format!("    Ok(())\n"));
                content.push_str("}\n\n");
            }

            std::fs::write(&file_path, content)
                .expect("Failed to write enterprise source file");
        }
    }
}

fn analyze_enterprise_project(project_path: &std::path::Path) -> EnterpriseAnalysis {
    let mut total_files = 0;
    let mut total_lines = 0;
    let mut total_size = 0;
    let mut rust_files = 0;

    for entry in walkdir::WalkDir::new(project_path) {
        let entry = entry.expect("Failed to read enterprise directory entry");
        if entry.file_type().is_file() {
            total_files += 1;
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                total_lines += content.lines().count();
                total_size += content.len();

                if entry.path().extension().map_or(false, |ext| ext == "rs") {
                    rust_files += 1;
                }
            }
        }
    }

    EnterpriseAnalysis {
        total_files,
        total_lines,
        total_size_bytes: total_size,
        rust_files,
    }
}

#[derive(Debug)]
struct EnterpriseAnalysis {
    total_files: usize,
    total_lines: usize,
    total_size_bytes: usize,
    rust_files: usize,
}

criterion_group!(
    benches,
    benchmark_response_times,
    benchmark_memory_usage,
    benchmark_large_project,
    benchmark_concurrent_sessions,
    benchmark_enterprise_workload,
);

criterion_main!(benches);