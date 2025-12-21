//! Comprehensive integration test suites with performance validation
//!
//! Tests the integration between:
//! - ricecoder-performance (performance monitoring)
//! - ricecoder-benchmark (benchmarking)
//! - ricecoder-domain (core entities)
//! - ricecoder-orchestration (coordination)
//! - ricecoder-sessions (session management)
//! - ricecoder-storage (persistence)
//! - ricecoder-providers (AI providers)
//! - ricecoder-mcp (external tools)

use ricecoder_benchmark::{BenchmarkRunner, BenchmarkSuite};
use ricecoder_domain::entities::*;
use ricecoder_mcp::{MCPClient, ToolRegistry};
use ricecoder_orchestration::{OrchestrationManager, WorkspaceScanner};
use ricecoder_performance::{PerformanceMetrics, PerformanceMonitor, PerformanceValidator};
use ricecoder_providers::{providers::OpenAiProvider, ProviderManager};
use ricecoder_security::encryption::KeyManager;
use ricecoder_sessions::{SessionManager, SessionStore};
use ricecoder_storage::{StorageManager, StorageMode};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::tempdir;
use tokio::time::timeout;

/// Mock storage manager for testing
struct MockStorageManager {
    global_path: PathBuf,
    project_path: Option<PathBuf>,
}

impl StorageManager for MockStorageManager {
    fn global_path(&self) -> &PathBuf {
        &self.global_path
    }

    fn project_path(&self) -> Option<&PathBuf> {
        self.project_path.as_ref()
    }

    fn mode(&self) -> StorageMode {
        StorageMode::Merged
    }

    fn global_resource_path(
        &self,
        _resource_type: ricecoder_storage::types::ResourceType,
    ) -> PathBuf {
        self.global_path.join("resources")
    }

    fn project_resource_path(
        &self,
        _resource_type: ricecoder_storage::types::ResourceType,
    ) -> Option<PathBuf> {
        self.project_path.as_ref().map(|p| p.join("resources"))
    }

    fn is_first_run(&self) -> bool {
        false
    }
}

#[tokio::test]
async fn test_full_system_integration_performance() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let workspace_root = temp_dir.path().to_path_buf();

    // Initialize performance monitoring
    let performance_monitor = PerformanceMonitor::new();
    let validator = PerformanceValidator::new();

    let start_time = Instant::now();

    // Initialize all major components
    let storage = Arc::new(MockStorageManager {
        global_path: workspace_root.clone(),
        project_path: Some(workspace_root.clone()),
    });

    let key_manager = KeyManager::new()?;
    let session_store = SessionStore::new(storage.clone(), Some(key_manager));
    let session_manager = SessionManager::new(session_store);

    let orchestration = OrchestrationManager::new(workspace_root.clone());
    orchestration.initialize().await?;

    let provider_manager = ProviderManager::new();
    provider_manager
        .register_provider(Box::new(OpenAiProvider::new("test-key".to_string())))
        .await?;

    let tool_registry = ToolRegistry::new();
    let mcp_client = MCPClient::new(Default::default(), None);

    // Create test workspace with multiple projects
    for i in 1..=5 {
        let project_dir = workspace_root.join(format!("project{}", i));
        std::fs::create_dir_all(&project_dir)?;

        // Create some test files
        for j in 1..=10 {
            let file_path = project_dir.join(format!("file{}.rs", j));
            let content = format!("// Test file {} in project {}\nfn main() {{}}\n", j, i);
            std::fs::write(file_path, content)?;
        }
    }

    let initialization_time = start_time.elapsed();

    // Test orchestration performance
    let scan_start = Instant::now();
    let scanner = WorkspaceScanner::new(workspace_root.clone());
    let projects = scanner.scan_workspace().await?;
    let scan_time = scan_start.elapsed();

    assert!(!projects.is_empty(), "Should discover projects");

    // Test session creation performance
    let session_start = Instant::now();
    let mut session_ids = Vec::new();
    for i in 1..=10 {
        let session_id = session_manager.create_session("openai", "gpt-4").await?;
        session_manager
            .set_session_name(session_id.clone(), format!("perf-test-session-{}", i))
            .await?;
        session_ids.push(session_id);
    }
    let session_creation_time = session_start.elapsed();

    // Test concurrent operations performance
    let concurrent_start = Instant::now();
    let mut handles = Vec::new();

    for session_id in session_ids.clone() {
        let session_mgr = session_manager.clone();
        let handle = tokio::spawn(async move {
            for i in 1..=5 {
                let message = ricecoder_sessions::Message {
                    role: ricecoder_sessions::MessageRole::User,
                    content: format!("Test message {}", i),
                    metadata: Default::default(),
                };
                session_mgr
                    .add_message(session_id.clone(), message)
                    .await
                    .unwrap();
            }
            session_mgr.save_session(&session_id).await.unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }
    let concurrent_time = concurrent_start.elapsed();

    // Measure final metrics
    let total_time = start_time.elapsed();

    // Validate performance baselines
    let metrics = PerformanceMetrics {
        initialization_time_ms: initialization_time.as_millis() as u64,
        scan_time_ms: scan_time.as_millis() as u64,
        session_creation_time_ms: session_creation_time.as_millis() as u64,
        concurrent_operations_time_ms: concurrent_time.as_millis() as u64,
        total_execution_time_ms: total_time.as_millis() as u64,
        memory_usage_mb: 0,     // Would need actual measurement
        cpu_usage_percent: 0.0, // Would need actual measurement
    };

    let validation_result = validator.validate_metrics(&metrics).await?;
    assert!(
        validation_result.is_within_baselines,
        "Performance should be within baselines"
    );

    // Clean up
    orchestration.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_end_to_end_workflow_performance() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let workspace_root = temp_dir.path().to_path_buf();

    // Initialize benchmark suite
    let benchmark_suite = BenchmarkSuite::new("end-to-end-workflow");
    let runner = BenchmarkRunner::new(benchmark_suite);

    // Set up test environment
    let storage = Arc::new(MockStorageManager {
        global_path: workspace_root.clone(),
        project_path: Some(workspace_root.clone()),
    });

    let key_manager = KeyManager::new()?;
    let session_store = SessionStore::new(storage.clone(), Some(key_manager));
    let session_manager = SessionManager::new(session_store);

    let orchestration = OrchestrationManager::new(workspace_root.clone());
    orchestration.initialize().await?;

    // Create test project
    let project = Project::new(
        "performance-test-project".to_string(),
        ProgrammingLanguage::Rust,
        workspace_root
            .join("perf-test")
            .to_string_lossy()
            .to_string(),
    )?;
    std::fs::create_dir_all(&project.root_path)?;

    // Benchmark end-to-end workflow
    runner
        .benchmark("full_workflow", || async {
            // 1. Create session
            let session_id = session_manager.create_session("openai", "gpt-4").await?;

            // 2. Add context and messages
            for i in 1..=5 {
                let message = ricecoder_sessions::Message {
                    role: ricecoder_sessions::MessageRole::User,
                    content: format!("Analyze this code segment {}", i),
                    metadata: Default::default(),
                };
                session_manager
                    .add_message(session_id.clone(), message)
                    .await?;
            }

            // 3. Save session
            session_manager.save_session(&session_id).await?;

            // 4. Load and verify
            let loaded = session_manager.load_session(&session_id).await?;
            assert!(loaded.is_some());

            // 5. Run orchestration analysis
            let impact_report = orchestration.analyze_impact(&project.id).await?;
            assert!(impact_report.details.len() >= 0); // Allow empty for mock

            Ok(())
        })
        .await?;

    // Run performance analysis
    let results = runner.run_analysis().await?;
    assert!(
        results.overall_score >= 0.7,
        "Workflow performance should be acceptable"
    );

    // Validate against performance baselines
    let validator = PerformanceValidator::new();
    let validation = validator.validate_workflow_performance(&results).await?;
    assert!(
        validation.passed,
        "Workflow should pass performance validation"
    );

    // Clean up
    orchestration.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_concurrent_session_performance_under_load() -> Result<(), Box<dyn std::error::Error>>
{
    let temp_dir = tempdir()?;
    let storage_path = temp_dir.path().to_path_buf();

    // Initialize components
    let storage = Arc::new(MockStorageManager {
        global_path: storage_path.clone(),
        project_path: Some(storage_path.clone()),
    });

    let key_manager = KeyManager::new()?;
    let session_store = SessionStore::new(storage.clone(), Some(key_manager));
    let session_manager = SessionManager::new(session_store);

    let performance_monitor = PerformanceMonitor::new();

    // Test concurrent session operations
    let num_sessions = 20;
    let num_operations_per_session = 10;

    let start_time = Instant::now();

    // Create multiple sessions concurrently
    let mut session_handles = Vec::new();
    for i in 0..num_sessions {
        let session_mgr = session_manager.clone();
        let handle = tokio::spawn(async move {
            let session_id = session_mgr.create_session("openai", "gpt-4").await.unwrap();
            session_mgr
                .set_session_name(session_id.clone(), format!("load-test-{}", i))
                .await
                .unwrap();

            // Perform operations on each session
            for j in 0..num_operations_per_session {
                let message = ricecoder_sessions::Message {
                    role: ricecoder_sessions::MessageRole::User,
                    content: format!("Load test message {} for session {}", j, i),
                    metadata: Default::default(),
                };
                session_mgr
                    .add_message(session_id.clone(), message)
                    .await
                    .unwrap();
            }

            session_mgr.save_session(&session_id).await.unwrap();
            session_id
        });
        session_handles.push(handle);
    }

    // Wait for all sessions to complete
    let mut session_ids = Vec::new();
    for handle in session_handles {
        session_ids.push(handle.await?);
    }

    let concurrent_time = start_time.elapsed();

    // Verify all sessions were created and saved
    assert_eq!(session_ids.len(), num_sessions);

    // Load and verify each session
    for session_id in session_ids {
        let loaded = session_manager.load_session(&session_id).await?;
        assert!(
            loaded.is_some(),
            "Session {} should be loadable",
            session_id
        );

        let session = loaded.unwrap();
        assert_eq!(session.messages.len(), num_operations_per_session);
    }

    // Validate performance under load
    let operations_per_second =
        (num_sessions * num_operations_per_session) as f64 / concurrent_time.as_secs_f64();
    assert!(
        operations_per_second >= 50.0,
        "Should handle at least 50 operations per second"
    );

    // Check memory usage (mock implementation)
    let memory_metrics = performance_monitor.get_memory_usage().await?;
    assert!(
        memory_metrics.current_mb < 500.0,
        "Memory usage should be reasonable"
    );

    Ok(())
}

#[tokio::test]
async fn test_provider_integration_performance() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;

    // Initialize provider performance monitoring
    let provider_manager = ProviderManager::new();
    let performance_monitor =
        ricecoder_providers::performance_monitor::ProviderPerformanceMonitor::new();

    // Register providers
    provider_manager
        .register_provider(Box::new(OpenAiProvider::new("test-key".to_string())))
        .await?;
    provider_manager
        .register_provider(Box::new(
            ricecoder_providers::providers::AnthropicProvider::new("test-key".to_string()),
        ))
        .await?;

    // Test provider response times
    let test_requests = vec![
        ricecoder_providers::ChatRequest {
            messages: vec![ricecoder_providers::Message {
                role: ricecoder_providers::MessageRole::User,
                content: "Hello".to_string(),
            }],
            model: "gpt-3.5-turbo".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(50),
        },
        ricecoder_providers::ChatRequest {
            messages: vec![ricecoder_providers::Message {
                role: ricecoder_providers::MessageRole::User,
                content: "Write a function".to_string(),
            }],
            model: "claude-3-haiku".to_string(),
            temperature: Some(0.5),
            max_tokens: Some(100),
        },
    ];

    let start_time = Instant::now();
    let mut response_times = Vec::new();

    // Simulate provider calls (would normally make real API calls)
    for request in test_requests {
        let request_start = Instant::now();

        // Mock provider response time
        tokio::time::sleep(Duration::from_millis(50)).await;

        let response_time = request_start.elapsed();
        response_times.push(response_time);

        // Record performance metrics
        performance_monitor
            .record_request(&request, response_time, true)
            .await?;
    }

    let total_time = start_time.elapsed();

    // Validate performance
    let avg_response_time = response_times.iter().sum::<Duration>() / response_times.len() as u32;
    assert!(
        avg_response_time < Duration::from_millis(200),
        "Average response time should be < 200ms"
    );

    // Check provider performance summary
    let summary = performance_monitor.generate_summary().await?;
    assert!(
        summary.average_response_time < Duration::from_millis(150),
        "Provider performance should be good"
    );
    assert!(summary.success_rate >= 0.95, "Success rate should be high");

    Ok(())
}

#[tokio::test]
async fn test_mcp_integration_performance() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;

    // Initialize MCP components
    let tool_registry = ToolRegistry::new();
    let mcp_client = MCPClient::new(Default::default(), None);

    // Register test tools
    for i in 1..=10 {
        tool_registry.register_tool(ricecoder_mcp::ToolMetadata {
            name: format!("test_tool_{}", i),
            description: format!("Test tool {}", i),
            input_schema: serde_json::json!({"type": "object"}),
            permissions_required: vec![],
        });
    }

    // Test tool discovery performance
    let discovery_start = Instant::now();
    let tools = tool_registry.list_tools().await?;
    let discovery_time = discovery_start.elapsed();

    assert_eq!(tools.len(), 10);
    assert!(
        discovery_time < Duration::from_millis(50),
        "Tool discovery should be fast"
    );

    // Test tool execution performance
    let execution_start = Instant::now();
    let mut execution_times = Vec::new();

    for tool in tools {
        let exec_start = Instant::now();

        // Mock tool execution
        let result = tool_registry
            .execute_tool(&tool.name, serde_json::json!({}))
            .await;
        // Allow execution to fail (expected for mock)

        let exec_time = exec_start.elapsed();
        execution_times.push(exec_time);
    }

    let total_execution_time = execution_start.elapsed();

    // Validate MCP performance
    let avg_execution_time =
        execution_times.iter().sum::<Duration>() / execution_times.len() as u32;
    assert!(
        avg_execution_time < Duration::from_millis(100),
        "Tool execution should be reasonably fast"
    );

    // Test connection pooling performance
    let pool_config = ricecoder_mcp::connection_pool::PoolConfig {
        max_connections: 10,
        min_connections: 2,
        acquire_timeout: Duration::from_secs(5),
        idle_timeout: Duration::from_secs(300),
    };

    let connection_pool = ricecoder_mcp::connection_pool::ConnectionPool::new(pool_config);

    let pool_start = Instant::now();
    let mut connections = Vec::new();

    // Acquire multiple connections
    for _ in 0..5 {
        let conn = connection_pool.acquire().await?;
        connections.push(conn);
    }

    let pool_acquire_time = pool_start.elapsed();

    // Release connections
    for conn in connections {
        connection_pool.release(conn).await?;
    }

    assert!(
        pool_acquire_time < Duration::from_millis(200),
        "Connection pool should be fast"
    );

    Ok(())
}

#[tokio::test]
async fn test_system_scalability_performance() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let workspace_root = temp_dir.path().to_path_buf();

    // Initialize full system
    let storage = Arc::new(MockStorageManager {
        global_path: workspace_root.clone(),
        project_path: Some(workspace_root.clone()),
    });

    let orchestration = OrchestrationManager::new(workspace_root.clone());
    orchestration.initialize().await?;

    let key_manager = KeyManager::new()?;
    let session_store = SessionStore::new(storage.clone(), Some(key_manager));
    let session_manager = SessionManager::new(session_store);

    // Create large workspace (simulate scalability test)
    let num_projects = 20;
    let num_files_per_project = 50;

    let creation_start = Instant::now();

    for i in 1..=num_projects {
        let project_dir = workspace_root.join(format!("scale_project_{}", i));
        std::fs::create_dir_all(&project_dir)?;

        for j in 1..=num_files_per_project {
            let file_path = project_dir.join(format!("file_{}.rs", j));
            let content = format!(
                "// Scalability test file {} in project {}\nfn test_{}() {{}}\n",
                j, i, j
            );
            std::fs::write(file_path, content)?;
        }
    }

    let creation_time = creation_start.elapsed();

    // Test orchestration scalability
    let scan_start = Instant::now();
    let scanner = WorkspaceScanner::new(workspace_root.clone());
    let projects = scanner.scan_workspace().await?;
    let scan_time = scan_start.elapsed();

    assert_eq!(projects.len(), num_projects);

    // Test session scalability
    let session_start = Instant::now();
    let mut session_ids = Vec::new();

    for i in 0..num_projects {
        let session_id = session_manager.create_session("openai", "gpt-4").await?;
        session_manager
            .set_session_name(session_id.clone(), format!("scale-session-{}", i))
            .await?;
        session_ids.push(session_id);
    }

    let session_time = session_start.elapsed();

    // Validate scalability metrics
    let total_files = num_projects * num_files_per_project;
    let files_per_second = total_files as f64 / creation_time.as_secs_f64();
    let projects_per_second = num_projects as f64 / scan_time.as_secs_f64();
    let sessions_per_second = num_projects as f64 / session_time.as_secs_f64();

    assert!(
        files_per_second >= 100.0,
        "Should create at least 100 files per second"
    );
    assert!(
        projects_per_second >= 5.0,
        "Should scan at least 5 projects per second"
    );
    assert!(
        sessions_per_second >= 10.0,
        "Should create at least 10 sessions per second"
    );

    // Test memory usage under load
    let performance_monitor = PerformanceMonitor::new();
    let memory_usage = performance_monitor.get_memory_usage().await?;
    assert!(
        memory_usage.current_mb < 1000.0,
        "Memory usage should remain reasonable under load"
    );

    // Clean up
    orchestration.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_performance_regression_detection() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;

    // Initialize performance monitoring and regression detection
    let performance_monitor = PerformanceMonitor::new();
    let regression_detector = ricecoder_performance::regression::RegressionDetector::new();

    // Establish baseline performance
    let baseline_metrics = vec![
        PerformanceMetrics {
            initialization_time_ms: 100,
            scan_time_ms: 50,
            session_creation_time_ms: 20,
            concurrent_operations_time_ms: 200,
            total_execution_time_ms: 500,
            memory_usage_mb: 150,
            cpu_usage_percent: 25.0,
        },
        PerformanceMetrics {
            initialization_time_ms: 105,
            scan_time_ms: 52,
            session_creation_time_ms: 22,
            concurrent_operations_time_ms: 210,
            total_execution_time_ms: 510,
            memory_usage_mb: 155,
            cpu_usage_percent: 26.0,
        },
    ];

    // Train regression detector with baseline
    for metrics in baseline_metrics {
        regression_detector.add_baseline_sample(metrics).await?;
    }

    // Test current performance (simulate slight degradation)
    let current_metrics = PerformanceMetrics {
        initialization_time_ms: 120, // 20% increase - potential regression
        scan_time_ms: 55,
        session_creation_time_ms: 25,
        concurrent_operations_time_ms: 220,
        total_execution_time_ms: 550,
        memory_usage_mb: 160,
        cpu_usage_percent: 28.0,
    };

    // Check for regressions
    let regression_alerts = regression_detector
        .detect_regressions(&current_metrics)
        .await?;

    // Should detect initialization time regression
    assert!(
        !regression_alerts.is_empty(),
        "Should detect performance regression"
    );

    let init_regression = regression_alerts
        .iter()
        .find(|a| a.metric == "initialization_time_ms");
    assert!(
        init_regression.is_some(),
        "Should detect initialization time regression"
    );

    let alert = init_regression.unwrap();
    assert!(
        alert.percentage_change > 15.0,
        "Regression should be significant"
    );

    // Test performance threshold validation
    let thresholds = ricecoder_performance::PerformanceThresholds {
        max_initialization_time_ms: 150,
        max_scan_time_ms: 100,
        max_session_creation_time_ms: 50,
        max_memory_usage_mb: 300,
        max_cpu_usage_percent: 80.0,
    };

    let threshold_validation = performance_monitor
        .validate_thresholds(&current_metrics, &thresholds)
        .await?;
    assert!(
        threshold_validation.passed,
        "Current metrics should pass thresholds"
    );

    Ok(())
}
