use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ricecoder_permissions::{
    permission::PermissionChecker, AuditAction, AuditLogger, AuditResult, GlobMatcher,
    PermissionConfig, PermissionLevel, PermissionManager, ToolPermission,
};
use std::sync::Arc;

// ============================================================================
// Benchmark 1: Permission Check Performance
// ============================================================================
// Validates: Permission check completes in < 100ms

fn benchmark_permission_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("permission_check");
    group.sample_size(100);

    // Setup: Create permission config with various numbers of permissions
    for num_permissions in [10, 50, 100, 500].iter() {
        let mut config = PermissionConfig::new();
        for i in 0..*num_permissions {
            config.add_permission(ToolPermission::new(
                format!("tool_{}", i),
                if i % 3 == 0 {
                    PermissionLevel::Allow
                } else if i % 3 == 1 {
                    PermissionLevel::Ask
                } else {
                    PermissionLevel::Deny
                },
            ));
        }

        let manager = PermissionManager::new(Arc::new(config));

        group.bench_with_input(
            BenchmarkId::from_parameter(num_permissions),
            num_permissions,
            |b, _| {
                b.iter(|| {
                    // Benchmark: Check permission for a tool
                    let _ = manager.check_permission(black_box("tool_42"), black_box(None));
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark 2: Glob Pattern Matching Performance
// ============================================================================
// Validates: Pattern matching is efficient

fn benchmark_glob_pattern_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("glob_pattern_matching");
    group.sample_size(100);

    let matcher = GlobMatcher::new();

    // Benchmark different pattern types
    let patterns = vec![
        ("exact_match", "my_tool_name"),
        ("wildcard_prefix", "my_*"),
        ("wildcard_suffix", "*_tool"),
        ("wildcard_both", "*tool*"),
        ("universal_wildcard", "*"),
    ];

    for (pattern_type, pattern) in patterns {
        group.bench_with_input(
            BenchmarkId::from_parameter(pattern_type),
            &pattern,
            |b, &pattern| {
                b.iter(|| {
                    // Benchmark: Match pattern against tool name
                    let _ = matcher.match_pattern(black_box(pattern), black_box("my_tool_name"));
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark 3: Audit Log Query Performance
// ============================================================================
// Validates: Audit log query completes in < 1 second

fn benchmark_audit_log_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("audit_log_query");
    group.sample_size(50);

    // Setup: Create logger with various numbers of entries
    for num_entries in [100, 1000, 10000].iter() {
        let logger = AuditLogger::new();

        // Populate with entries
        for i in 0..*num_entries {
            let tool = format!("tool_{}", i % 50);
            let agent = if i % 2 == 0 {
                Some(format!("agent_{}", i % 10))
            } else {
                None
            };

            if i % 3 == 0 {
                logger.log_execution(tool, agent, None).unwrap();
            } else if i % 3 == 1 {
                logger.log_denial(tool, agent, None).unwrap();
            } else {
                logger.log_prompt(tool, agent, None).unwrap();
            }
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(num_entries),
            num_entries,
            |b, _| {
                b.iter(|| {
                    // Benchmark: Query all entries
                    let _ = logger.entries();
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark 4: Audit Log Filtering Performance
// ============================================================================
// Validates: Filtering large audit logs is efficient

fn benchmark_audit_log_filtering(c: &mut Criterion) {
    let mut group = c.benchmark_group("audit_log_filtering");
    group.sample_size(50);

    let logger = AuditLogger::new();

    // Populate with 10000 entries
    for i in 0..10000 {
        let tool = format!("tool_{}", i % 50);
        let agent = if i % 2 == 0 {
            Some(format!("agent_{}", i % 10))
        } else {
            None
        };

        if i % 3 == 0 {
            logger.log_execution(tool, agent, None).unwrap();
        } else if i % 3 == 1 {
            logger.log_denial(tool, agent, None).unwrap();
        } else {
            logger.log_prompt(tool, agent, None).unwrap();
        }
    }

    group.bench_function("filter_by_tool", |b| {
        b.iter(|| {
            let entries = logger.entries().unwrap();
            let filtered: Vec<_> = entries.iter().filter(|e| e.tool == "tool_42").collect();
            black_box(filtered);
        });
    });

    group.bench_function("filter_by_action", |b| {
        b.iter(|| {
            let entries = logger.entries().unwrap();
            let filtered: Vec<_> = entries
                .iter()
                .filter(|e| e.action == AuditAction::Denied)
                .collect();
            black_box(filtered);
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark 5: Permission Manager Operations
// ============================================================================
// Validates: Permission manager operations are efficient

fn benchmark_permission_manager_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("permission_manager_operations");
    group.sample_size(100);

    let mut config = PermissionConfig::new();
    for i in 0..100 {
        config.add_permission(ToolPermission::new(
            format!("tool_{}", i),
            PermissionLevel::Allow,
        ));
    }

    let manager = PermissionManager::new(Arc::new(config));

    group.bench_function("check_permission", |b| {
        b.iter(|| {
            let _ = manager.check_permission(black_box("tool_42"), black_box(None));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_permission_check,
    benchmark_glob_pattern_matching,
    benchmark_audit_log_query,
    benchmark_audit_log_filtering,
    benchmark_permission_manager_operations,
);

criterion_main!(benches);
