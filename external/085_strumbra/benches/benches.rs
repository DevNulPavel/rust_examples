use strumbra::{ArcString, BoxString};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::Rng as _;

const INPUT_LENGTHS: [usize; 6] = [4, 8, 12, 16, 32, 64];

fn cmp_random(c: &mut Criterion) {
    let mut group = c.benchmark_group("cmp-random");
    for len in INPUT_LENGTHS {
        group.bench_with_input(BenchmarkId::new("String", len), &len, |b, &len| {
            b.iter_batched_ref(
                || (random_string(len), random_string(len)),
                |(a, b)| a.cmp(&b),
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("ArcString", len), &len, |b, &len| {
            b.iter_batched_ref(
                || {
                    (
                        ArcString::try_from(random_string(len).as_str()).unwrap(),
                        ArcString::try_from(random_string(len).as_str()).unwrap(),
                    )
                },
                |(a, b)| a.cmp(&b),
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("BoxString", len), &len, |b, &len| {
            b.iter_batched_ref(
                || {
                    (
                        BoxString::try_from(random_string(len)).unwrap(),
                        BoxString::try_from(random_string(len)).unwrap(),
                    )
                },
                |(a, b)| a.cmp(&b),
                criterion::BatchSize::SmallInput,
            )
        });
    }
}

fn cmp_same(c: &mut Criterion) {
    let mut group = c.benchmark_group("cmp-same");
    for len in INPUT_LENGTHS {
        group.bench_with_input(BenchmarkId::new("String", len), &len, |b, &len| {
            b.iter_batched_ref(
                || {
                    let s = random_string(len);
                    (s.clone(), s)
                },
                |(a, b)| a.cmp(&b),
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("ArcString", len), &len, |b, &len| {
            b.iter_batched_ref(
                || {
                    let s = random_string(len);
                    (
                        ArcString::try_from(s.as_str()).unwrap(),
                        ArcString::try_from(s.as_str()).unwrap(),
                    )
                },
                |(a, b)| a.cmp(&b),
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("BoxString", len), &len, |b, &len| {
            b.iter_batched_ref(
                || {
                    let s = random_string(len);
                    (
                        BoxString::try_from(s.as_str()).unwrap(),
                        BoxString::try_from(s.as_str()).unwrap(),
                    )
                },
                |(a, b)| a.cmp(&b),
                criterion::BatchSize::SmallInput,
            )
        });
    }
}

fn eq_random(c: &mut Criterion) {
    let mut group = c.benchmark_group("eq-random");
    for len in INPUT_LENGTHS {
        group.bench_with_input(BenchmarkId::new("String", len), &len, |b, &len| {
            b.iter_batched_ref(
                || (random_string(len), random_string(len)),
                |(a, b)| a.eq(&b),
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("ArcString", len), &len, |b, &len| {
            b.iter_batched_ref(
                || {
                    (
                        ArcString::try_from(random_string(len).as_str()).unwrap(),
                        ArcString::try_from(random_string(len).as_str()).unwrap(),
                    )
                },
                |(a, b)| a.eq(&b),
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("BoxString", len), &len, |b, &len| {
            b.iter_batched_ref(
                || {
                    (
                        BoxString::try_from(random_string(len)).unwrap(),
                        BoxString::try_from(random_string(len)).unwrap(),
                    )
                },
                |(a, b)| a.eq(&b),
                criterion::BatchSize::SmallInput,
            )
        });
    }
}

fn eq_same(c: &mut Criterion) {
    let mut group = c.benchmark_group("eq-same");
    for len in INPUT_LENGTHS {
        group.bench_with_input(BenchmarkId::new("String", len), &len, |b, &len| {
            b.iter_batched_ref(
                || {
                    let s = random_string(len);
                    (s.clone(), s)
                },
                |(a, b)| a.eq(&b),
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("ArcString", len), &len, |b, &len| {
            b.iter_batched_ref(
                || {
                    let s = random_string(len);
                    (
                        ArcString::try_from(s.as_str()).unwrap(),
                        ArcString::try_from(s.as_str()).unwrap(),
                    )
                },
                |(a, b)| a.eq(&b),
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("BoxString", len), &len, |b, &len| {
            b.iter_batched_ref(
                || {
                    let s = random_string(len);
                    (
                        BoxString::try_from(s.as_str()).unwrap(),
                        BoxString::try_from(s.as_str()).unwrap(),
                    )
                },
                |(a, b)| a.eq(&b),
                criterion::BatchSize::SmallInput,
            )
        });
    }
}

fn cmp_random_mixed_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("cmp-random-mixed-types");
    for len in INPUT_LENGTHS {
        group.bench_with_input(BenchmarkId::new("String", len), &len, |b, &len| {
            b.iter_batched_ref(
                || (random_string(len), random_string(len)),
                |(a, b)| a.cmp(&b),
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("ArcString", len), &len, |b, &len| {
            b.iter_batched_ref(
                || {
                    (
                        ArcString::try_from(random_string(len).as_str()).unwrap(),
                        random_string(len),
                    )
                },
                |(a, b)| PartialOrd::partial_cmp(a, b),
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("BoxString", len), &len, |b, &len| {
            b.iter_batched_ref(
                || {
                    (
                        BoxString::try_from(random_string(len)).unwrap(),
                        random_string(len),
                    )
                },
                |(a, b)| PartialOrd::partial_cmp(a, b),
                criterion::BatchSize::SmallInput,
            )
        });
    }
}

fn cmp_same_mixed_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("cmp-same-mixed-types");
    for len in INPUT_LENGTHS {
        group.bench_with_input(BenchmarkId::new("String", len), &len, |b, &len| {
            b.iter_batched_ref(
                || {
                    let s = random_string(len);
                    (s.clone(), s)
                },
                |(a, b)| a.cmp(&b),
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("ArcString", len), &len, |b, &len| {
            b.iter_batched_ref(
                || {
                    let s = random_string(len);
                    (ArcString::try_from(s.as_str()).unwrap(), s)
                },
                |(a, b)| PartialOrd::partial_cmp(a, b),
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("BoxString", len), &len, |b, &len| {
            b.iter_batched_ref(
                || {
                    let s = random_string(len);
                    (BoxString::try_from(s.as_str()).unwrap(), s)
                },
                |(a, b)| PartialOrd::partial_cmp(a, b),
                criterion::BatchSize::SmallInput,
            )
        });
    }
}

fn eq_random_mixed_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("eq-random-mixed-types");
    for len in INPUT_LENGTHS {
        group.bench_with_input(BenchmarkId::new("String", len), &len, |b, &len| {
            b.iter_batched_ref(
                || (random_string(len), random_string(len)),
                |(a, b)| a.eq(&b),
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("ArcString", len), &len, |b, &len| {
            b.iter_batched_ref(
                || {
                    (
                        ArcString::try_from(random_string(len).as_str()).unwrap(),
                        random_string(len),
                    )
                },
                |(a, b)| PartialEq::eq(a, b),
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("BoxString", len), &len, |b, &len| {
            b.iter_batched_ref(
                || {
                    (
                        BoxString::try_from(random_string(len)).unwrap(),
                        random_string(len),
                    )
                },
                |(a, b)| PartialEq::eq(a, b),
                criterion::BatchSize::SmallInput,
            )
        });
    }
}

fn eq_same_mixed_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("eq-same-mixed-types");
    for len in INPUT_LENGTHS {
        group.bench_with_input(BenchmarkId::new("String", len), &len, |b, &len| {
            b.iter_batched_ref(
                || {
                    let s = random_string(len);
                    (s.clone(), s)
                },
                |(a, b)| a.eq(&b),
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("ArcString", len), &len, |b, &len| {
            b.iter_batched_ref(
                || {
                    let s = random_string(len);
                    (ArcString::try_from(s.as_str()).unwrap(), s)
                },
                |(a, b)| PartialEq::eq(a, b),
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("BoxString", len), &len, |b, &len| {
            b.iter_batched_ref(
                || {
                    let s = random_string(len);
                    (BoxString::try_from(s.as_str()).unwrap(), s)
                },
                |(a, b)| PartialEq::eq(a, b),
                criterion::BatchSize::SmallInput,
            )
        });
    }
}

fn construct_empty(c: &mut Criterion) {
    let mut group = c.benchmark_group("construct-empty");
    group.bench_function(BenchmarkId::new("ArcString", "copy"), |b| {
        b.iter_batched(
            String::new,
            |s| ArcString::try_from(s.as_str()),
            criterion::BatchSize::SmallInput,
        )
    });
    group.bench_function(BenchmarkId::new("ArcString", "move"), |b| {
        b.iter_batched(
            String::new,
            ArcString::try_from,
            criterion::BatchSize::SmallInput,
        )
    });
    group.bench_function(BenchmarkId::new("BoxString", "copy"), |b| {
        b.iter_batched(
            String::new,
            |s| BoxString::try_from(s.as_str()),
            criterion::BatchSize::SmallInput,
        )
    });
    group.bench_function(BenchmarkId::new("BoxString", "move"), |b| {
        b.iter_batched(
            String::new,
            BoxString::try_from,
            criterion::BatchSize::SmallInput,
        )
    });
}

fn construct_non_empty(c: &mut Criterion) {
    let mut group = c.benchmark_group("construct-non-empty");
    for len in INPUT_LENGTHS {
        group.bench_with_input(BenchmarkId::new("ArcStringCopy", len), &len, |b, &len| {
            b.iter_batched(
                || random_string(len),
                |s| ArcString::try_from(s.as_str()),
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("ArcStringMove", len), &len, |b, &len| {
            b.iter_batched(
                || random_string(len),
                ArcString::try_from,
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("BoxStringCopy", len), &len, |b, &len| {
            b.iter_batched(
                || random_string(len),
                |s| BoxString::try_from(s.as_str()),
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("BoxStringMove", len), &len, |b, &len| {
            b.iter_batched(
                || random_string(len),
                BoxString::try_from,
                criterion::BatchSize::SmallInput,
            )
        });
    }
}

fn random_string(len: usize) -> String {
    let bytes = rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(len)
        .collect::<Vec<_>>();

    String::from_utf8(bytes).unwrap()
}

criterion_group!(
    benches,
    cmp_random,
    cmp_same,
    eq_random,
    eq_same,
    cmp_random_mixed_types,
    cmp_same_mixed_types,
    eq_random_mixed_types,
    eq_same_mixed_types,
    construct_empty,
    construct_non_empty,
);
criterion_main!(benches);
