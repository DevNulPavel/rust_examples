use criterion::{black_box, criterion_group, criterion_main, Criterion};
use smallstr::SmallString;
use std::fmt::Write;

fn stack_64() {
    let mut buf = SmallString::<[u8; 64]>::new();

    assert_eq!(buf.len(), 0);

    write!(&mut buf, "Birds: {}", black_box("asdasd")).unwrap();

    assert_eq!(buf.len(), 13);
}

fn stack_256() {
    let mut buf = SmallString::<[u8; 256]>::new();

    assert_eq!(buf.len(), 0);

    write!(&mut buf, "Birds: {}", black_box("asdasd")).unwrap();

    assert_eq!(buf.len(), 13);
}

fn stack_512() {
    let mut buf = SmallString::<[u8; 512]>::new();

    assert_eq!(buf.len(), 0);

    write!(&mut buf, "Birds: {}", black_box("asdasd")).unwrap();

    assert_eq!(buf.len(), 13);
}

fn stack_4096() {
    let mut buf = SmallString::<[u8; 4096]>::new();

    assert_eq!(buf.len(), 0);

    write!(&mut buf, "Birds: {}", black_box("asdasd")).unwrap();

    assert_eq!(buf.len(), 13);
}

fn stack_16384() {
    let mut buf = SmallString::<[u8; 16384]>::new();

    assert_eq!(buf.len(), 0);

    write!(&mut buf, "Birds: {}", black_box("asdasd")).unwrap();

    assert_eq!(buf.len(), 13);
}

fn alloc(len: usize) {
    let mut buf = String::with_capacity(len);

    assert_eq!(buf.len(), 0);

    write!(&mut buf, "Birds: {}", black_box("asdasd")).unwrap();

    assert_eq!(buf.len(), 13);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("alloc 64", |b| b.iter(|| alloc(black_box(64))));
    c.bench_function("stack 64", |b| b.iter(stack_64));

    c.bench_function("alloc 256", |b| b.iter(|| alloc(black_box(256))));
    c.bench_function("stack 256", |b| b.iter(stack_256));

    c.bench_function("alloc 512", |b| b.iter(|| alloc(black_box(512))));
    c.bench_function("stack 512", |b| b.iter(stack_512));

    c.bench_function("alloc 4096", |b| b.iter(|| alloc(black_box(4096))));
    c.bench_function("stack 4096", |b| b.iter(stack_4096));

    c.bench_function("alloc 16384", |b| b.iter(|| alloc(black_box(16384))));
    c.bench_function("stack 16384", |b| b.iter(stack_16384));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
