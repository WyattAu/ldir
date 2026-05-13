//! Benchmark: Incremental recompilation (single-word change in large document).
//!
//! Measures:
//!   - Zero-change path: dirty set empty, returns Arc::clone instantly.
//!   - Single-node dirty: marks one paragraph dirty, triggers full L-IR recompile.
//!   - Update diff: replaces one text node, auto-detects dirty nodes.

use std::sync::Arc;

use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};

use ldir_core::compiler::context::CompileContext;
use ldir_core::layout::incremental::IncrementalLayout;
use ldir_ir::sir::v2::SIRModuleV2;
use ldir_ir::sir::v2::nodes::{Node, NodeType};

/// Build a V2 document with `n` paragraphs of realistic text.
fn make_n_paragraph_doc(n: u32) -> SIRModuleV2 {
    let mut module = SIRModuleV2::new();
    let mut next_id: u32 = 0;

    let doc_id = next_id;
    module.body.push(Node::new(next_id, NodeType::Document));
    next_id += 1;

    for i in 0..n {
        let para_id = next_id;
        module
            .body
            .push(Node::new(next_id, NodeType::Paragraph).with_parent(doc_id));
        next_id += 1;

        let text_id = next_id;
        module.body.push(
            Node::new(
                next_id,
                NodeType::Text {
                    content: format!(
                        "Paragraph {p}: The quick brown fox jumps over the lazy dog. \
                         Sed ut perspiciatis unde omnis iste natus error sit voluptatem \
                         accusantium doloremque laudantium, totam rem aperiam eaque ipsa \
                         quae ab illo inventore veritatis et quasi architecto beatae vitae \
                         dicta sunt explicabo. Nemo enim ipsam voluptatem quia voluptas \
                         sit aspernatur aut odit aut fugit.",
                        p = i + 1,
                    ),
                },
            )
            .with_parent(para_id),
        );
        next_id += 1;

        if let Some(p) = module.body.get_mut(para_id) {
            p.add_child(text_id);
        }
        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(para_id);
        }
    }

    module
}

/// Build a modified copy with paragraph `idx` text changed (simulates single-word edit).
fn make_edited_doc(n: u32, edit_idx: u32) -> SIRModuleV2 {
    let mut module = SIRModuleV2::new();
    let mut next_id: u32 = 0;

    let doc_id = next_id;
    module.body.push(Node::new(next_id, NodeType::Document));
    next_id += 1;

    for i in 0..n {
        let para_id = next_id;
        module
            .body
            .push(Node::new(next_id, NodeType::Paragraph).with_parent(doc_id));
        next_id += 1;

        let content = if i == edit_idx {
            format!(
                "Paragraph {p}: MODIFIED TEXT HERE. The quick brown fox jumps over the lazy dog. \
                 This paragraph was changed to simulate a single-word edit in a large document.",
                p = i + 1,
            )
        } else {
            format!(
                "Paragraph {p}: The quick brown fox jumps over the lazy dog. \
                 Sed ut perspiciatis unde omnis iste natus error sit voluptatem \
                 accusantium doloremque laudantium, totam rem aperiam eaque ipsa \
                 quae ab illo inventore veritatis et quasi architecto beatae vitae \
                 dicta sunt explicabo. Nemo enim ipsam voluptatem quia voluptas \
                 sit aspernatur aut odit aut fugit.",
                p = i + 1,
            )
        };

        let text_id = next_id;
        module
            .body
            .push(Node::new(next_id, NodeType::Text { content }).with_parent(para_id));
        next_id += 1;

        if let Some(p) = module.body.get_mut(para_id) {
            p.add_child(text_id);
        }
        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(para_id);
        }
    }

    module
}

fn bench_incremental_no_change(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_no_change");

    for &n in &[100u32, 500, 1000] {
        let module = Arc::new(make_n_paragraph_doc(n));
        let ctx = CompileContext::default();

        // Initial compile to get baseline L-IR
        let initial_lir =
            Arc::new(ldir_core::compile_sir_to_lir(&module, &ctx).expect("initial compile failed"));

        group.bench_function(format!("{n}_paras"), |b| {
            let layout = IncrementalLayout::new(Arc::clone(&module));
            b.iter(|| {
                black_box(
                    layout
                        .recompile_lir(&initial_lir, &ctx)
                        .expect("recompile failed"),
                );
            });
        });
    }

    group.finish();
}

fn bench_incremental_single_edit(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_single_edit");

    for &n in &[100u32, 500, 1000] {
        let module = Arc::new(make_n_paragraph_doc(n));
        let ctx = CompileContext::default();

        let initial_lir =
            Arc::new(ldir_core::compile_sir_to_lir(&module, &ctx).expect("initial compile failed"));

        // Edit the middle paragraph
        let edit_idx = n / 2;
        let edited = Arc::new(make_edited_doc(n, edit_idx));

        group.bench_function(format!("{n}_paras"), |b| {
            b.iter_batched(
                || {
                    let mut layout = IncrementalLayout::new(Arc::clone(&module));
                    layout.update_sir(Arc::clone(&edited));
                    layout
                },
                |layout| {
                    black_box(
                        layout
                            .recompile_lir(&initial_lir, &ctx)
                            .expect("recompile failed"),
                    );
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_incremental_no_change,
    bench_incremental_single_edit,
);
criterion_main!(benches);
