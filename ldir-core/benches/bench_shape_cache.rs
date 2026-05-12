//! Benchmark: HarfBuzz shape cache hit/miss rates.
//!
//! Measures the overhead of the LRU cache for shaped text runs.
//! Two scenarios:
//!   - Cache miss path: unique strings force shaping on every lookup.
//!   - Cache hit path: repeated strings return cached ShapedRun.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use ldir_core::fp266::Fp266;
use ldir_core::shaping::cache::ShapeCache;
use ldir_core::shaping::fast_path::shape_ascii;

/// Generate `n` unique text strings of realistic paragraph length.
fn unique_texts(n: usize) -> Vec<String> {
    (0..n)
        .map(|i| {
            format!(
                "Paragraph {i}: The quick brown fox jumps over the lazy dog. \
                 Sed ut perspiciatis unde omnis iste natus error sit voluptatem \
                 accusantium doloremque laudantium, totam rem aperiam."
            )
        })
        .collect()
}

fn bench_cache_miss(c: &mut Criterion) {
    let mut group = c.benchmark_group("shape_cache_miss");

    for &n in &[100usize, 500, 1000] {
        let texts = unique_texts(n);
        group.bench_function(format!("unique_{n}"), |b| {
            b.iter_batched(
                || ShapeCache::new(n * 2),
                |mut cache| {
                    for t in &texts {
                        black_box(
                            cache.get_or_shape(t.as_str(), 0, Fp266::from_int(12), |text, fid, fs| {
                                shape_ascii(text, fs, fid)
                            }),
                        );
                    }
                    cache
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_cache_hit(c: &mut Criterion) {
    let mut group = c.benchmark_group("shape_cache_hit");

    for &pool_size in &[50usize, 200, 500] {
        let texts = unique_texts(pool_size);
        let lookups = 1000;

        group.bench_function(format!("pool_{pool_size}_lookups_{lookups}"), |b| {
            b.iter_batched(
                || {
                    let mut cache = ShapeCache::new(pool_size * 2);
                    // Warm up: populate cache (all misses)
                    for t in &texts {
                        cache.get_or_shape(t.as_str(), 0, Fp266::from_int(12), |text, fid, fs| {
                            shape_ascii(text, fs, fid)
                        });
                    }
                    cache.reset_stats();
                    cache
                },
                |mut cache| {
                    for i in 0..lookups {
                        let t = &texts[i % pool_size];
                        black_box(
                            cache.get_or_shape(t.as_str(), 0, Fp266::from_int(12), |_, _, _| {
                                unreachable!("should always hit")
                            }),
                        );
                    }
                    cache
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_cache_lru_eviction(c: &mut Criterion) {
    let mut group = c.benchmark_group("shape_cache_lru_eviction");

    // Cache smaller than working set: forces eviction churn
    for &(pool_size, cache_cap) in &[(200usize, 50), (500, 100), (1000, 200)] {
        let texts = unique_texts(pool_size);
        group.bench_function(format!("pool_{pool_size}_cap_{cache_cap}"), |b| {
            b.iter_batched(
                || ShapeCache::new(cache_cap),
                |mut cache| {
                    // Iterate through pool twice: first pass fills+evicts, second pass hits
                    for t in texts.iter().chain(texts.iter()) {
                        black_box(
                            cache.get_or_shape(t.as_str(), 0, Fp266::from_int(12), |text, fid, fs| {
                                shape_ascii(text, fs, fid)
                            }),
                        );
                    }
                    cache
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(benches, bench_cache_miss, bench_cache_hit, bench_cache_lru_eviction);
criterion_main!(benches);
