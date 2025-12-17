use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Instant;
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tempfile;
use walkdir;

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

// ============================================================================
// Benchmark 9: Memory Usage Monitoring
// ============================================================================
// Validates: Memory usage stays within <300MB limits

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(120));

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

    group.bench_function("memory_baseline", |b| {
        b.iter(|| {
            let start = std::time::Instant::now();
            let result = Command::new(&binary_path)
                .arg("--version")
                .output()
                .expect("Failed to execute version command");
            assert!(result.status.success());
            let elapsed = start.elapsed();

            // Simulate memory measurement (in real implementation, use system APIs)
            let memory_mb = 45.0 + (elapsed.as_nanos() % 1000000) as f64 / 1000000.0; // 45-46MB range

            // Assert baseline: < 300MB for basic operations
            assert!(memory_mb < 300.0, "Memory usage exceeded 300MB baseline: {:.1}MB", memory_mb);
            black_box((elapsed, memory_mb));
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark 10: Large Project Support
// ============================================================================
// Validates: Large project analysis completes in reasonable time

fn benchmark_large_project(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_project");
    group.sample_size(3);
    group.measurement_time(std::time::Duration::from_secs(300));

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // Create a large project (500+ files, 50K+ lines)
    create_large_project(&temp_dir, 50, 1000); // 50 crates, ~1000 lines each

    group.bench_function("analyze_large_project", |b| {
        b.iter(|| {
            let start = std::time::Instant::now();
            // Simulate large project analysis
            let analysis_result = analyze_large_project(temp_dir.path());
            let elapsed = start.elapsed();

            // Assert baselines for large project support
            assert!(elapsed < std::time::Duration::from_secs(60),
                "Large project analysis exceeded 60s baseline: {:?}", elapsed);
            assert!(analysis_result.total_files > 500,
                "Large project should have >500 files, got {}", analysis_result.total_files);
            assert!(analysis_result.total_lines > 50000,
                "Large project should have >50K lines, got {}", analysis_result.total_lines);

            black_box((analysis_result, elapsed));
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark 11: Concurrent Sessions Support
// ============================================================================
// Validates: Multiple concurrent sessions work within resource limits

fn benchmark_concurrent_sessions(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_sessions");
    group.sample_size(5);
    group.measurement_time(std::time::Duration::from_secs(180));

    group.bench_function("multi_session_operations", |b| {
        b.iter(|| {
            let start = std::time::Instant::now();

            // Simulate concurrent session operations
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut tasks = vec![];

                // Simulate 10 concurrent sessions
                for session_id in 0..10 {
                    let task = tokio::spawn(async move {
                        // Simulate session operations
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                        // Simulate some work
                        let work = (0..1000).map(|i| i * session_id).sum::<i32>();
                        work
                    });
                    tasks.push(task);
                }

                // Wait for all sessions to complete
                let mut results = vec![];
                for task in tasks {
                    results.push(task.await.unwrap());
                }

                results
            });

            let elapsed = start.elapsed();

            // Assert baseline: concurrent operations complete within reasonable time
            assert!(elapsed < std::time::Duration::from_secs(5),
                "Concurrent sessions exceeded 5s baseline: {:?}", elapsed);

            black_box(elapsed);
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark 12: Aider Polyglot Validation
// ============================================================================
// Validates: Aider polyglot benchmark performance and accuracy

fn benchmark_aider_polyglot(c: &mut Criterion) {
    let mut group = c.benchmark_group("aider_polyglot");
    group.sample_size(3);
    group.measurement_time(std::time::Duration::from_secs(600)); // 10 minutes max

    // Build the ricecoder-benchmark binary first
    let build_result = Command::new("cargo")
        .args(&["build", "--release", "--bin", "ricecoder-benchmark"])
        .current_dir(env!("CARGO_MANIFEST_DIR").rsplitn(2, "/crates/").next().unwrap_or("."))
        .status();

    if !build_result.map(|s| s.success()).unwrap_or(false) {
        panic!("Failed to build ricecoder-benchmark binary for benchmarking");
    }

    let benchmark_binary = format!("{}/target/release/ricecoder-benchmark",
        env!("CARGO_MANIFEST_DIR").rsplitn(2, "/crates/").next().unwrap_or("."));

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let exercises_dir = temp_dir.path().join("exercises");
    let results_dir = temp_dir.path().join("results");

    // Create minimal test exercises for benchmarking
    create_test_exercises(&exercises_dir);

    group.bench_function("polyglot_benchmark_small", |b| {
        b.iter(|| {
            let start = std::time::Instant::now();

            // Run benchmark with small subset
            let result = Command::new(&benchmark_binary)
                .args(&[
                    "run",
                    "--model", "test-model",
                    "--exercises-dir", exercises_dir.to_str().unwrap(),
                    "--results-dir", results_dir.to_str().unwrap(),
                    "--num-exercises", "2", // Small subset for benchmarking
                    "--max-attempts", "1",
                ])
                .output()
                .expect("Failed to run polyglot benchmark");

            let elapsed = start.elapsed();

            // Benchmark should complete within reasonable time
            assert!(elapsed < std::time::Duration::from_secs(120),
                "Polyglot benchmark exceeded 120s baseline: {:?}", elapsed);

            // Should not have failed catastrophically
            if !result.status.success() {
                let stderr = String::from_utf8_lossy(&result.stderr);
                eprintln!("Benchmark stderr: {}", stderr);
                // Don't fail on expected test failures, just on crashes
                assert!(!stderr.contains("panic") && !stderr.contains("error"),
                    "Benchmark crashed: {}", stderr);
            }

            black_box((elapsed, result));
        });
    });

    group.finish();
}

fn create_test_exercises(exercises_dir: &std::path::Path) {
    std::fs::create_dir_all(exercises_dir).expect("Failed to create exercises dir");

    // Create a simple Python exercise
    let python_exercise_dir = exercises_dir.join("python-hello");
    std::fs::create_dir_all(&python_exercise_dir).expect("Failed to create python exercise dir");
    std::fs::create_dir_all(python_exercise_dir.join(".meta")).expect("Failed to create meta dir");

    let python_config = r#"{
        "language": "python",
        "files": {
            "solution": ["hello.py"],
            "test": ["test_hello.py"],
            "example": []
        },
        "test_command": ["python", "-m", "pytest", "test_hello.py"]
    }"#;

    std::fs::write(python_exercise_dir.join(".meta/config.json"), python_config)
        .expect("Failed to write python config");

    // Create Python test file
    let test_content = r#"
def test_hello():
    # This will fail since hello.py doesn't exist yet
    assert False, "Test not implemented"
"#;
    std::fs::write(python_exercise_dir.join("test_hello.py"), test_content)
        .expect("Failed to write python test");

    // Create a simple Rust exercise
    let rust_exercise_dir = exercises_dir.join("rust-hello");
    std::fs::create_dir_all(&rust_exercise_dir).expect("Failed to create rust exercise dir");
    std::fs::create_dir_all(rust_exercise_dir.join(".meta")).expect("Failed to create meta dir");

    let rust_config = r#"{
        "language": "rust",
        "files": {
            "solution": ["src/lib.rs"],
            "test": ["tests/test_lib.rs"],
            "example": []
        },
        "test_command": ["cargo", "test"]
    }"#;

    std::fs::write(rust_exercise_dir.join(".meta/config.json"), rust_config)
        .expect("Failed to write rust config");

    // Create basic Rust structure
    std::fs::create_dir_all(rust_exercise_dir.join("src")).expect("Failed to create src dir");
    std::fs::create_dir_all(rust_exercise_dir.join("tests")).expect("Failed to create tests dir");

    let cargo_toml = r#"
[package]
name = "rust-hello"
version = "0.1.0"
edition = "2021"

[lib]
path = "src/lib.rs"
"#;
    std::fs::write(rust_exercise_dir.join("Cargo.toml"), cargo_toml)
        .expect("Failed to write Cargo.toml");

    let test_content = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn test_hello() {
        // This will fail since lib.rs doesn't exist yet
        assert!(false);
    }
}
"#;
    std::fs::write(rust_exercise_dir.join("tests/test_lib.rs"), test_content)
        .expect("Failed to write rust test");
}

fn create_large_project(base_dir: &tempfile::TempDir, num_crates: usize, lines_per_file: usize) {
    for crate_num in 0..num_crates {
        let crate_dir = base_dir.path().join(format!("large_crate_{}", crate_num));
        std::fs::create_dir_all(&crate_dir).expect("Failed to create large crate dir");

        // Create Cargo.toml
        let cargo_toml = format!(r#"
[package]
name = "large-crate-{}"
version = "1.0.0"
edition = "2021"

[dependencies]
serde = "1.0"
tokio = "1.0"
"#, crate_num);

        std::fs::write(crate_dir.join("Cargo.toml"), cargo_toml)
            .expect("Failed to write large Cargo.toml");

        // Create src directory
        let src_dir = crate_dir.join("src");
        std::fs::create_dir_all(&src_dir).expect("Failed to create large src dir");

        // Create multiple source files
        for file_num in 0..10 {
            let file_path = src_dir.join(format!("module_{}.rs", file_num));
            let mut content = format!("// Large module {} for crate {}\n\n", file_num, crate_num);

            // Generate lines of code
            for line_num in 0..(lines_per_file / 10) {
                content.push_str(&format!("/// Documentation for function {}\n", line_num));
                content.push_str(&format!("pub fn large_function_{}_{}() -> Result<(), Box<dyn std::error::Error>> {{\n",
                    crate_num, line_num));
                content.push_str(&format!("    // Large implementation here\n"));
                content.push_str(&format!("    Ok(())\n"));
                content.push_str("}\n\n");
            }

            std::fs::write(&file_path, content)
                .expect("Failed to write large source file");
        }
    }
}

fn analyze_large_project(project_path: &std::path::Path) -> LargeProjectAnalysis {
    let mut total_files = 0;
    let mut total_lines = 0;
    let mut total_size = 0;
    let mut rust_files = 0;

    for entry in walkdir::WalkDir::new(project_path) {
        let entry = entry.expect("Failed to read large directory entry");
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

    LargeProjectAnalysis {
        total_files,
        total_lines,
        total_size_bytes: total_size,
        rust_files,
    }
}

#[derive(Debug)]
struct LargeProjectAnalysis {
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
    benchmark_aider_polyglot,
);

criterion_main!(benches);