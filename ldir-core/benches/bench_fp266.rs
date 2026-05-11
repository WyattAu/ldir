use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ldir_core::fp266::Fp266;

fn bench_fp266_add(c: &mut Criterion) {
    c.bench_function("fp266_add", |b| {
        b.iter(|| {
            let mut acc = Fp266::ZERO;
            for i in 0..1000 {
                acc = acc + Fp266::from_int(i);
            }
            black_box(acc)
        })
    });
}

fn bench_fp266_mul(c: &mut Criterion) {
    c.bench_function("fp266_mul", |b| {
        b.iter(|| {
            let mut acc = Fp266::from_int(1);
            for i in 1..1000 {
                acc = acc.mul(Fp266::from_int(i));
            }
            black_box(acc)
        })
    });
}

fn bench_fp266_sqrt(c: &mut Criterion) {
    c.bench_function("fp266_sqrt", |b| {
        b.iter(|| {
            for i in 1..1000 {
                black_box(Fp266::from_int(i).sqrt());
            }
        })
    });
}

fn bench_fp266_from_f64(c: &mut Criterion) {
    c.bench_function("fp266_from_f64", |b| {
        b.iter(|| {
            for i in 0..1000 {
                black_box(Fp266::from_f64(i as f64 * 0.123456));
            }
        })
    });
}

fn bench_fp266_to_f64(c: &mut Criterion) {
    c.bench_function("fp266_to_f64", |b| {
        b.iter(|| {
            for i in 0..1000 {
                black_box(Fp266::from_int(i).to_f64());
            }
        })
    });
}

fn bench_fp266_div(c: &mut Criterion) {
    c.bench_function("fp266_div", |b| {
        b.iter(|| {
            let mut acc = Fp266::from_int(1);
            for i in 1..1000 {
                acc = acc.div(Fp266::from_int(i));
            }
            black_box(acc)
        })
    });
}

criterion_group!(
    benches,
    bench_fp266_add,
    bench_fp266_mul,
    bench_fp266_sqrt,
    bench_fp266_from_f64,
    bench_fp266_to_f64,
    bench_fp266_div
);
criterion_main!(benches);
