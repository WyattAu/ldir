use criterion::{Criterion, criterion_group, criterion_main};
use ldir_core::layout::hyphenate::hyphenate_word;

fn bench_hyphenation(c: &mut Criterion) {
    let mut group = c.benchmark_group("hyphenation");

    let words = [
        "documentation",
        "implementation",
        "configuration",
        "specification",
        "architecture",
        "performance",
        "interpolation",
        "demonstration",
        "characteristic",
        "responsibility",
        "internationalization",
        "compartmentalization",
    ];

    for word in &words {
        group.bench_function(*word, |b| {
            b.iter(|| hyphenate_word(word));
        });
    }
    group.finish();
}

fn bench_string_interning(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_interning");

    let strings: Vec<String> = (0..1000).map(|i| format!("string_{}", i)).collect();

    group.bench_function("1000_strings", |b| {
        b.iter(|| {
            let mut interner = ldir_core::interner::StringInterner::new();
            for s in &strings {
                interner.intern(s);
            }
        });
    });
    group.finish();
}

criterion_group!(benches, bench_hyphenation, bench_string_interning);
criterion_main!(benches);
