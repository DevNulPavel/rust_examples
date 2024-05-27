mod icmp_function;

use criterion::{criterion_group, criterion_main, Criterion}; // black_box

use icmp_function::test_icmp;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Bench icmp function", |b| {
        b.iter(|| {
            test_icmp();
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

