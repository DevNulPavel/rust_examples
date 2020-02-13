mod fibonacci;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

// use fibonacci::fibonacci_recursive as test_f;
use fibonacci::fibonacci_non_recursive as test_f;


fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| {
        b.iter(|| {
            test_f(black_box(20))
        });
    });

    c.bench_function("fib 10", |b| {
        b.iter(|| {
            test_f(black_box(10))
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

