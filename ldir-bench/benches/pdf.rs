use criterion::{Criterion, Throughput, criterion_group, criterion_main};

fn bench_pdf_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("pdf_write");

    let doc = r#"# PDF Write Benchmark

## Section

This document tests PDF writing performance.

Paragraph with text.

| A | B |
|---|---|
| 1 | 2 |

- Item
"#;

    group.throughput(Throughput::Bytes(doc.len() as u64));
    group.bench_function("small_document", |b| {
        b.iter(|| {
            let module = ldir_md::parse_markdown(doc);
            let gir = ldir_core::compiler::compile_sir(&module).unwrap();
            let _ = ldir_pdf::converter::gir_to_pdf(&gir);
        });
    });
    group.finish();
}

fn bench_pdf_write_with_font(c: &mut Criterion) {
    let mut group = c.benchmark_group("pdf_write_with_font");

    let doc = r#"# PDF Write Benchmark

## Section

This document tests PDF writing performance with font embedding.

Paragraph with text.
"#;

    let font_data = ldir_test_helpers::test_font_data();

    group.throughput(Throughput::Bytes(doc.len() as u64));
    group.bench_function("small_document", |b| {
        b.iter(|| {
            let module = ldir_md::parse_markdown(doc);
            let gir = ldir_core::compiler::compile_sir(&module).unwrap();
            let _ = ldir_pdf::converter::gir_to_pdf_with_font(&gir, Some(&font_data));
        });
    });
    group.finish();
}

criterion_group!(benches, bench_pdf_write, bench_pdf_write_with_font);
criterion_main!(benches);
