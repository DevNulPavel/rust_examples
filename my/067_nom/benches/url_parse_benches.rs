use criterion::{black_box, criterion_group, criterion_main, Criterion};
use test67_nom::url_parse::{scheme::Scheme, authority::Authority};

fn scheme_benchmark(c: &mut Criterion) {
    c.bench_function("HTTPS Scheme parse", |b| {
        b.iter(|| Scheme::try_parse(black_box("https://www.rust-lang.org/en-US/")).unwrap())
    });

    c.bench_function("HTTP Scheme parse", |b| {
        b.iter(|| Scheme::try_parse(black_box("http://www.rust-lang.org/en-US/")).unwrap())
    });
}

fn authority_benchmark(c: &mut Criterion) {
    c.bench_function("Full auth parse", |b| {
        b.iter(|| Authority::try_parse(black_box("user:pass@www.google.com")).unwrap() )
    });

    c.bench_function("Partial auth parse", |b| {
        b.iter(|| Authority::try_parse(black_box("user:@www.google.com")).unwrap() )
    });
}

criterion_group!(benches, scheme_benchmark, authority_benchmark);
criterion_main!(benches);
