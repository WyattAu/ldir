use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use ldir_core::compiler::compile_sir;
use ldir_md::parse_markdown;

fn generate_markdown_document(size: usize) -> String {
    let mut doc = String::new();
    doc.push_str("# Benchmark Document\n\n");

    for i in 0..size {
        doc.push_str(&format!("## Section {}\n\n", i));
        doc.push_str("This is paragraph text for benchmarking purposes. ");
        doc.push_str("It contains multiple sentences with varying lengths. ");
        doc.push_str("The quick brown fox jumps over the lazy dog.\n\n");

        if i % 3 == 0 {
            doc.push_str("- List item one\n");
            doc.push_str("- List item two\n");
            doc.push_str("- List item three\n\n");
        }

        if i % 5 == 0 {
            doc.push_str("| Column A | Column B | Column C |\n");
            doc.push_str("|----------|----------|----------|\n");
            for j in 0..5 {
                doc.push_str(&format!("| Cell {} | Cell {} | Cell {} |\n", j, j, j));
            }
            doc.push_str("\n");
        }

        if i % 7 == 0 {
            doc.push_str("> This is a blockquote for benchmarking.\n\n");
        }

        if i % 11 == 0 {
            doc.push_str("```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\n\n");
        }
    }

    doc.push_str("# Final Section\n\nEnd of document.\n");
    doc
}

fn bench_markdown_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("markdown_parsing");

    for size in [10, 50, 100, 500] {
        let doc = generate_markdown_document(size);
        group.throughput(Throughput::Bytes(doc.len() as u64));
        group.bench_with_input(BenchmarkId::new("parse", size), &doc, |b, doc| {
            b.iter(|| parse_markdown(doc));
        });
    }
    group.finish();
}

fn bench_latex_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("latex_parsing");

    let simple = r#"\documentclass{article}
\begin{document}
\section{Introduction}
This is a simple document with \textbf{bold} and \textit{italic} text.
\begin{equation}
E = mc^2
\end{equation}
\end{document}"#;

    group.throughput(Throughput::Bytes(simple.len() as u64));
    group.bench_function("simple_document", |b| {
        b.iter(|| ldir_tex::parse_tex(simple));
    });
    group.finish();
}

fn bench_sir_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("sir_compilation");

    for size in [10, 50, 100] {
        let doc = generate_markdown_document(size);
        group.throughput(Throughput::Bytes(doc.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("compile_md_to_sir", size),
            &doc,
            |b, doc| {
                b.iter(|| {
                    let module = parse_markdown(doc);
                    let _ = compile_sir(&module);
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_markdown_parsing,
    bench_latex_parsing,
    bench_sir_compilation
);
criterion_main!(benches);
