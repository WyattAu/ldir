use criterion::{criterion_group, criterion_main, Criterion, black_box, BatchSize, BenchmarkId};
use ldir_core::compiler::compile_sir;
use ldir_ir::sir::{ROOT_SENTINEL, SIRDocument, SIRInstruction, SIROpcode};

fn make_paragraph_doc(n_paragraphs: usize) -> SIRDocument {
    let mut doc = SIRDocument::new();
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        0,
        ROOT_SENTINEL,
        0,
    ));
    let mut id = 1u32;
    for _ in 0..n_paragraphs {
        let para_id = id;
        id += 1;
        let content_id = id;
        id += 1;
        doc.push(SIRInstruction::new(SIROpcode::PushBlock, para_id, 0, 0));
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, content_id, para_id, 0),
            b"The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs.",
        );
    }
    doc
}

fn bench_arena_paragraph_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena_paragraph");
    for n in [1, 5, 10, 50] {
        group.bench_with_input(BenchmarkId::new("paragraphs", n), &n, |b, &n| {
            let doc = make_paragraph_doc(n);
            b.iter_batched(
                || doc.clone(),
                |d| black_box(compile_sir(&d).unwrap()),
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, bench_arena_paragraph_compilation);
criterion_main!(benches);
