use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use ldir_core::validator::validate_sir;
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

fn bench_validate_sir(c: &mut Criterion) {
    let mut group = c.benchmark_group("validate_sir");
    for size in [10, 100, 1000] {
        group.bench_with_input(BenchmarkId::new("instructions", size), &size, |b, &size| {
            let doc = make_doc(size);
            b.iter(|| black_box(validate_sir(&doc)));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_validate_sir);
criterion_main!(benches);
