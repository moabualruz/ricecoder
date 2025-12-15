//! Performance benchmarks for the DI container

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ricecoder_di::{DIContainer, DIResult, ServiceLifetime};
use std::sync::Arc;

/// Simple test service for benchmarking
#[derive(Debug, Clone)]
struct TestService {
    id: u32,
    data: Vec<u8>,
}

impl TestService {
    fn new(id: u32) -> Self {
        Self {
            id,
            data: vec![0; 1024], // 1KB of data
        }
    }
}

fn benchmark_service_registration(c: &mut Criterion) {
    c.bench_function("register_singleton_service", |b| {
        b.iter(|| {
            let container = DIContainer::new();
            let result = container.register(|_| {
                let service = Arc::new(TestService::new(black_box(42)));
                Ok(service)
            });
            black_box(result)
        })
    });

    c.bench_function("register_transient_service", |b| {
        b.iter(|| {
            let container = DIContainer::new();
            let result = container.register_transient(|_| {
                let service = Arc::new(TestService::new(black_box(42)));
                Ok(service)
            });
            black_box(result)
        })
    });

    c.bench_function("register_scoped_service", |b| {
        b.iter(|| {
            let container = DIContainer::new();
            let result = container.register_scoped(|_| {
                let service = Arc::new(TestService::new(black_box(42)));
                Ok(service)
            });
            black_box(result)
        })
    });
}

fn benchmark_service_resolution(c: &mut Criterion) {
    // Setup container with registered services
    let container = DIContainer::new();
    container.register(|_| {
        let service = Arc::new(TestService::new(42));
        Ok(service)
    }).unwrap();

    container.register_transient(|_| {
        let service = Arc::new(TestService::new(42));
        Ok(service)
    }).unwrap();

    c.bench_function("resolve_singleton_service", |b| {
        b.iter(|| {
            let result: DIResult<Arc<TestService>> = container.resolve();
            black_box(result)
        })
    });

    c.bench_function("resolve_transient_service", |b| {
        b.iter(|| {
            let result: DIResult<Arc<TestService>> = container.resolve();
            black_box(result)
        })
    });

    c.bench_function("resolve_scoped_service", |b| {
        let scope = ricecoder_di::ServiceScope::new();
        b.iter(|| {
            let result: DIResult<Arc<TestService>> = container.resolve_with_scope(Some(&scope));
            black_box(result)
        })
    });
}

fn benchmark_container_operations(c: &mut Criterion) {
    c.bench_function("create_container", |b| {
        b.iter(|| {
            let container = DIContainer::new();
            black_box(container)
        })
    });

    c.bench_function("check_service_registration", |b| {
        let container = DIContainer::new();
        container.register(|_| {
            let service = Arc::new(TestService::new(42));
            Ok(service)
        }).unwrap();

        b.iter(|| {
            let result = container.is_registered::<TestService>();
            black_box(result)
        })
    });

    c.bench_function("get_service_count", |b| {
        b.iter(|| {
            let container = DIContainer::new();
            // Register a few services
            container.register(|_| {
                let service = Arc::new(TestService::new(1));
                Ok(service)
            }).unwrap();
            container.register(|_| {
                let service = Arc::new(TestService::new(2));
                Ok(service)
            }).unwrap();
            container.register(|_| {
                let service = Arc::new(TestService::new(3));
                Ok(service)
            }).unwrap();

            let count = container.service_count();
            black_box(count)
        })
    });
}

fn benchmark_memory_usage(c: &mut Criterion) {
    c.bench_function("memory_overhead_per_service", |b| {
        b.iter(|| {
            let container = DIContainer::new();

            // Register multiple services to measure memory scaling
            container.register(|_| {
                let service = Arc::new(TestService::new(1));
                Ok(service)
            }).unwrap();
            container.register(|_| {
                let service = Arc::new(TestService::new(2));
                Ok(service)
            }).unwrap();
            container.register(|_| {
                let service = Arc::new(TestService::new(3));
                Ok(service)
            }).unwrap();

            // Force resolution to create instances
            for _ in 0..3 {
                let _: DIResult<Arc<TestService>> = container.resolve();
            }

            black_box(container.service_count())
        })
    });
}

criterion_group!(
    benches,
    benchmark_service_registration,
    benchmark_service_resolution,
    benchmark_container_operations,
    benchmark_memory_usage
);
criterion_main!(benches);