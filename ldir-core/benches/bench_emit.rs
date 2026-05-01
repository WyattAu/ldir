use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ldir_core::compiler::compile_sir;
use ldir_core::emitter::{emit_gir, parse_gir};
use ldir_ir::sir::{ROOT_SENTINEL, SIRDocument, SIRInstruction, SIROpcode};

fn make_gir_doc() -> ldir_ir::gir::GIRDocument {
    let mut doc = SIRDocument::new();
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        0,
        ROOT_SENTINEL,
        0,
    ));
    for i in 1..500u32 {
        doc.push(SIRInstruction::new(SIROpcode::SetContent, i, 0, 0));
    }
    compile_sir(&doc).unwrap()
}

fn bench_emit_gir(c: &mut Criterion) {
    let doc = make_gir_doc();
    c.bench_function("emit_gir", |b| {
        b.iter(|| black_box(emit_gir(&doc)));
    });
}

fn bench_parse_gir(c: &mut Criterion) {
    let doc = make_gir_doc();
    let bytes = emit_gir(&doc);
    c.bench_function("parse_gir", |b| {
        b.iter(|| black_box(parse_gir(&bytes).unwrap()));
    });
}

criterion_group!(benches, bench_emit_gir, bench_parse_gir);
criterion_main!(benches);
