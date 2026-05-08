use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use ldir_core::compiler::compile_sir;
use ldir_core::validator::validate_sir;
use ldir_ir::sir::{BlockType, ROOT_SENTINEL, SIRDocument, SIRInstruction, SIROpcode};

fn bench_md_parse_small(c: &mut Criterion) {
    let input = "# Hello\n\nWorld paragraph";
    c.bench_function("md_parse_small", |b| {
        b.iter(|| black_box(ldir_md::parse_markdown(input)));
    });
}

fn bench_md_parse_medium(c: &mut Criterion) {
    let input = r#"# Document Title

This is the first paragraph of a medium-sized markdown document.
It contains **bold text** and *italic text* for testing inline styles.

## Section One

Here is a paragraph with `inline code` and [a link](https://example.com).

### Subsection

Another paragraph here with more text content for layout testing.

## Section Two

- First item in a list
- Second item with **bold**
- Third item with `code`

> A blockquote with some text inside it.

## Section Three

| Header 1 | Header 2 |
| --- | --- |
| Cell 1 | Cell 2 |
| Cell 3 | Cell 4 |

More paragraphs follow with additional text content.

```
code block example
with multiple lines
```

## Section Four

Final section with concluding paragraphs and text.

1. Ordered item one
2. Ordered item two
3. Ordered item three

---

End of document."#;
    c.bench_function("md_parse_medium", |b| {
        b.iter(|| black_box(ldir_md::parse_markdown(input)));
    });
}

fn bench_tex_parse_small(c: &mut Criterion) {
    let input = "\\section{Intro}Hello";
    c.bench_function("tex_parse_small", |b| {
        b.iter(|| black_box(ldir_tex::parse_tex(input)));
    });
}

fn bench_tex_parse_medium(c: &mut Criterion) {
    let input = r#"\documentclass{article}
\usepackage{amsmath}
\begin{document}
\section{Introduction}
This is the first paragraph of a medium TeX document.
It contains \textbf{bold text} and \textit{italic text}.
\subsection{Subsection One}
Here is a paragraph with more content for layout testing.
\subsection{Subsection Two}
\begin{itemize}
\item First item in a list
\item Second item with \textbf{bold}
\item Third item
\end{itemize}
\section{Methods}
Another section with additional paragraphs and text content.
\begin{enumerate}
\item First ordered item
\item Second ordered item
\end{enumerate}
\section{Results}
Final section with concluding text and more paragraphs.
\end{document}"#;
    c.bench_function("tex_parse_medium", |b| {
        b.iter(|| black_box(ldir_tex::parse_tex(input)));
    });
}

fn make_sir_doc(n_instructions: usize) -> SIRDocument {
    let mut doc = SIRDocument::new();
    let root_payload = doc.payload_mut().append(&[BlockType::Document as u8]);
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        0,
        ROOT_SENTINEL,
        root_payload,
    ));
    let para_payload = doc.payload_mut().append(&[BlockType::Paragraph as u8]);
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        1,
        0,
        para_payload,
    ));
    let mut id = 2u32;
    for _ in 1..n_instructions {
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, id, 1, 0),
            b"The quick brown fox jumps over the lazy dog.",
        );
        id += 1;
    }
    doc
}

fn bench_compile_sir(c: &mut Criterion) {
    let doc = make_sir_doc(20);
    c.bench_function("compile_sir_20_instructions", |b| {
        b.iter(|| {
            let result = compile_sir(&doc);
            if let Ok(gir) = result {
                black_box(gir);
            }
        });
    });
}

fn bench_pdf_generate(c: &mut Criterion) {
    let sir_doc = make_sir_doc(10);
    let gir_doc = match compile_sir(&sir_doc) {
        Ok(gir) => gir,
        Err(_) => return,
    };
    c.bench_function("pdf_generate_small", |b| {
        b.iter(|| black_box(ldir_pdf::converter::gir_to_pdf(&gir_doc)));
    });
}

fn bench_validator(c: &mut Criterion) {
    let mut group = c.benchmark_group("validator");
    for size in [10, 50, 100] {
        group.bench_with_input(BenchmarkId::new("instructions", size), &size, |b, &size| {
            let doc = make_sir_doc(size);
            b.iter(|| black_box(validate_sir(&doc)));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_md_parse_small,
    bench_md_parse_medium,
    bench_tex_parse_small,
    bench_tex_parse_medium,
    bench_compile_sir,
    bench_pdf_generate,
    bench_validator,
);
criterion_main!(benches);
