use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ricecoder_mcp::ToolMarshaler;
use serde_json::json;

fn benchmark_marshal_input(c: &mut Criterion) {
    let marshaler = ToolMarshaler::new();
    let input = black_box(json!({"param1": "value1", "param2": 42}));

    c.bench_function("marshal_input", |b| {
        b.iter(|| marshaler.marshal_input(&input))
    });
}

fn benchmark_unmarshal_output(c: &mut Criterion) {
    let marshaler = ToolMarshaler::new();
    let output = black_box(json!({"result": "success"}));

    c.bench_function("unmarshal_output", |b| {
        b.iter(|| marshaler.unmarshal_output(&output))
    });
}

criterion_group!(benches, benchmark_marshal_input, benchmark_unmarshal_output);
criterion_main!(benches);
