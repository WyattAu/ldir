use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use ldir_core::compiler::compile_sir;
use ldir_ir::sir::{ROOT_SENTINEL, SIRDocument, SIRInstruction, SIROpcode};

fn make_doc(n_instructions: usize) -> SIRDocument {
    let mut doc = SIRDocument::new();
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        0,
        ROOT_SENTINEL,
        0,
    ));
    for i in 1..n_instructions {
        doc.push(SIRInstruction::new(SIROpcode::SetContent, i as u32, 0, 0));
    }
    doc
}

fn bench_compile_sir(c: &mut Criterion) {
    let mut group = c.benchmark_group("compile_sir");
    for size in [10, 100, 1000] {
        group.bench_with_input(BenchmarkId::new("instructions", size), &size, |b, &size| {
            let doc = make_doc(size);
            b.iter(|| black_box(compile_sir(&doc).unwrap()));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_compile_sir);
criterion_main!(benches);
