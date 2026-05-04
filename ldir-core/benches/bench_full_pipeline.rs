use criterion::{Criterion, black_box, criterion_group, criterion_main};

use ldir_core::compile_sir_to_lir;
use ldir_core::compiler::context::CompileContext;
use ldir_core::compiler::v2_compile::compile_v2_document;
use ldir_core::interner::StringInterner;
use ldir_ir::sir::v2::SIRModuleV2;
use ldir_ir::sir::v2::nodes::{ColSpec, ColumnAlign, ListType, Node, NodeType};

fn make_small_doc() -> SIRModuleV2 {
    let mut module = SIRModuleV2::new();

    let doc_id = module.body.push(Node::new(0, NodeType::Document));
    let para_id = module
        .body
        .push(Node::new(1, NodeType::Paragraph).with_parent(doc_id));
    let text_id = module.body.push(
        Node::new(
            2,
            NodeType::Text {
                content: "The quick brown fox jumps over the lazy dog. \
            This sentence contains every letter of the English alphabet and serves as a \
            convenient test string for font rendering and layout engines. It has been used \
            in typography since the late nineteenth century and remains popular today. \
            Lorem ipsum dolor sit amet, consectetur adipiscing elit."
                    .to_string(),
            },
        )
        .with_parent(para_id),
    );

    if let Some(p) = module.body.get_mut(para_id) {
        p.add_child(text_id);
    }
    if let Some(d) = module.body.get_mut(doc_id) {
        d.add_child(para_id);
    }

    module
}

fn make_medium_doc() -> SIRModuleV2 {
    let mut module = SIRModuleV2::new();
    let mut next_id: u32 = 0;

    let doc_id = next_id;
    module.body.push(Node::new(next_id, NodeType::Document));
    next_id += 1;

    for sec in 0..5u32 {
        let heading_id = next_id;
        module
            .body
            .push(Node::new(next_id, NodeType::Section).with_parent(doc_id));
        next_id += 1;
        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(heading_id);
        }

        let heading_text_id = next_id;
        module.body.push(
            Node::new(
                next_id,
                NodeType::Text {
                    content: format!("Section {}: An overview of topics", sec + 1),
                },
            )
            .with_parent(heading_id),
        );
        next_id += 1;
        if let Some(h) = module.body.get_mut(heading_id) {
            h.add_child(heading_text_id);
        }

        for para in 0..2u32 {
            let para_id = next_id;
            module
                .body
                .push(Node::new(next_id, NodeType::Paragraph).with_parent(doc_id));
            next_id += 1;
            if let Some(d) = module.body.get_mut(doc_id) {
                d.add_child(para_id);
            }

            let text_id = next_id;
            module.body.push(
                Node::new(
                    next_id,
                    NodeType::Text {
                        content: format!(
                            "Paragraph {} in section {}. This is a body of text that contains \
                        enough words to span multiple lines when typeset. The compiler \
                        must handle line breaking, justification, and page overflow \
                        correctly for this content. Sed ut perspiciatis unde omnis iste \
                        natus error sit voluptatem accusantium doloremque laudantium.",
                            para + 1,
                            sec + 1,
                        ),
                    },
                )
                .with_parent(para_id),
            );
            next_id += 1;
            if let Some(p) = module.body.get_mut(para_id) {
                p.add_child(text_id);
            }
        }

        let list_id = next_id;
        module.body.push(
            Node::new(
                next_id,
                NodeType::List {
                    list_type: ListType::Unordered,
                    ordered: false,
                    start: None,
                },
            )
            .with_parent(doc_id),
        );
        next_id += 1;
        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(list_id);
        }

        for item in 0..3u32 {
            let item_id = next_id;
            module
                .body
                .push(Node::new(next_id, NodeType::ListItem).with_parent(list_id));
            next_id += 1;
            if let Some(l) = module.body.get_mut(list_id) {
                l.add_child(item_id);
            }

            let item_para_id = next_id;
            module
                .body
                .push(Node::new(next_id, NodeType::Paragraph).with_parent(item_id));
            next_id += 1;
            if let Some(i) = module.body.get_mut(item_id) {
                i.add_child(item_para_id);
            }

            let item_text_id = next_id;
            module.body.push(
                Node::new(
                    next_id,
                    NodeType::Text {
                        content: format!(
                            "List item {}: a brief description of this point.",
                            item + 1
                        ),
                    },
                )
                .with_parent(item_para_id),
            );
            next_id += 1;
            if let Some(ip) = module.body.get_mut(item_para_id) {
                ip.add_child(item_text_id);
            }
        }

        let table_id = next_id;
        module.body.push(
            Node::new(
                next_id,
                NodeType::Table {
                    col_specs: vec![
                        ColSpec {
                            align: ColumnAlign::Left,
                            width: None,
                        },
                        ColSpec {
                            align: ColumnAlign::Left,
                            width: None,
                        },
                        ColSpec {
                            align: ColumnAlign::Right,
                            width: None,
                        },
                    ],
                    num_cols: 3,
                    caption: None,
                    column_widths: vec![],
                    header_row: false,
                },
            )
            .with_parent(doc_id),
        );
        next_id += 1;
        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(table_id);
        }

        let header_row_id = next_id;
        module
            .body
            .push(Node::new(next_id, NodeType::TableRow { is_header: true }).with_parent(table_id));
        next_id += 1;
        if let Some(t) = module.body.get_mut(table_id) {
            t.add_child(header_row_id);
        }

        for (col, header) in ["Name", "Description", "Value"].iter().enumerate() {
            let cell_id = next_id;
            module.body.push(
                Node::new(
                    next_id,
                    NodeType::TableCell {
                        colspan: 1,
                        rowspan: 1,
                    },
                )
                .with_parent(header_row_id),
            );
            next_id += 1;
            if let Some(hr) = module.body.get_mut(header_row_id) {
                hr.add_child(cell_id);
            }

            let cell_text_id = next_id;
            module.body.push(
                Node::new(
                    next_id,
                    NodeType::Text {
                        content: header.to_string(),
                    },
                )
                .with_parent(cell_id),
            );
            next_id += 1;
            if let Some(c) = module.body.get_mut(cell_id) {
                c.add_child(cell_text_id);
            }
            let _ = col;
        }

        for row in 0..2u32 {
            let row_id = next_id;
            module.body.push(
                Node::new(next_id, NodeType::TableRow { is_header: false }).with_parent(table_id),
            );
            next_id += 1;
            if let Some(t) = module.body.get_mut(table_id) {
                t.add_child(row_id);
            }

            for (col, val) in [
                format!("Item {}", row + 1),
                "A description".to_string(),
                format!("{}", (row + 1) * 100),
            ]
            .iter()
            .enumerate()
            {
                let cell_id = next_id;
                module.body.push(
                    Node::new(
                        next_id,
                        NodeType::TableCell {
                            colspan: 1,
                            rowspan: 1,
                        },
                    )
                    .with_parent(row_id),
                );
                next_id += 1;
                if let Some(r) = module.body.get_mut(row_id) {
                    r.add_child(cell_id);
                }

                let cell_text_id = next_id;
                module.body.push(
                    Node::new(
                        next_id,
                        NodeType::Text {
                            content: val.clone(),
                        },
                    )
                    .with_parent(cell_id),
                );
                next_id += 1;
                if let Some(c) = module.body.get_mut(cell_id) {
                    c.add_child(cell_text_id);
                }
                let _ = col;
            }
        }
    }

    module
}

fn make_large_doc() -> SIRModuleV2 {
    let mut module = SIRModuleV2::new();
    let mut next_id: u32 = 0;

    let doc_id = next_id;
    module.body.push(Node::new(next_id, NodeType::Document));
    next_id += 1;

    for i in 0..100u32 {
        if i % 10 == 0 {
            let heading_id = next_id;
            module
                .body
                .push(Node::new(next_id, NodeType::Section).with_parent(doc_id));
            next_id += 1;
            if let Some(d) = module.body.get_mut(doc_id) {
                d.add_child(heading_id);
            }

            let heading_text_id = next_id;
            module.body.push(
                Node::new(
                    next_id,
                    NodeType::Text {
                        content: format!("Section {}", i / 10 + 1),
                    },
                )
                .with_parent(heading_id),
            );
            next_id += 1;
            if let Some(h) = module.body.get_mut(heading_id) {
                h.add_child(heading_text_id);
            }
        }

        let para_id = next_id;
        module
            .body
            .push(Node::new(next_id, NodeType::Paragraph).with_parent(doc_id));
        next_id += 1;
        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(para_id);
        }

        let text_id = next_id;
        module.body.push(
            Node::new(
                next_id,
                NodeType::Text {
                    content: format!(
                        "Paragraph {}. This is a substantial block of text designed to fill \
                    space on the page. The quick brown fox jumps over the lazy dog. \
                    Sed ut perspiciatis unde omnis iste natus error sit voluptatem \
                    accusantium doloremque laudantium, totam rem aperiam eaque ipsa \
                    quae ab illo inventore veritatis et quasi architecto beatae vitae \
                    dicta sunt explicabo. Nemo enim ipsam voluptatem quia voluptas \
                    sit aspernatur aut odit aut fugit.",
                        i + 1,
                    ),
                },
            )
            .with_parent(para_id),
        );
        next_id += 1;
        if let Some(p) = module.body.get_mut(para_id) {
            p.add_child(text_id);
        }
    }

    module
}

fn bench_v2_compile_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("v2_compile");

    let small = make_small_doc();
    group.bench_function("small_1page", |b| {
        b.iter(|| {
            let mut ctx = CompileContext::default();
            black_box(compile_v2_document(&small, &mut ctx).unwrap())
        });
    });

    let medium = make_medium_doc();
    group.bench_function("medium_10pages", |b| {
        b.iter(|| {
            let mut ctx = CompileContext::default();
            black_box(compile_v2_document(&medium, &mut ctx).unwrap())
        });
    });

    let large = make_large_doc();
    group.bench_function("large_100pages", |b| {
        b.iter(|| {
            let mut ctx = CompileContext::default();
            black_box(compile_v2_document(&large, &mut ctx).unwrap())
        });
    });

    group.finish();
}

fn bench_lir_pipeline(c: &mut Criterion) {
    let module = make_medium_doc();

    c.bench_function("lir_compile_medium", |b| {
        b.iter(|| {
            let ctx = CompileContext::default();
            black_box(compile_sir_to_lir(&module, &ctx).unwrap())
        });
    });
}

fn bench_pdf_generation(c: &mut Criterion) {
    let module = make_small_doc();

    let gir_doc = {
        let mut ctx = CompileContext::default();
        compile_v2_document(&module, &mut ctx).unwrap()
    };

    c.bench_function("pdf_generation_small", |b| {
        b.iter(|| black_box(ldir_pdf::converter::gir_to_pdf(&gir_doc)));
    });
}

fn bench_interner(c: &mut Criterion) {
    let repeated = [
        "Chapter",
        "Section",
        "Figure",
        "Table",
        "The quick brown fox jumps over the lazy dog.",
    ];

    c.bench_function("interner_1000_strings_20pct_dupes", |b| {
        b.iter(|| {
            let mut interner = StringInterner::new();
            for i in 0..1000 {
                let owned;
                let s = if i % 5 == 0 {
                    repeated[i as usize % repeated.len()]
                } else {
                    owned = format!("unique_text_{}", i);
                    &owned as &str
                };
                black_box(interner.intern(s));
            }
            black_box(interner.len())
        });
    });

    c.bench_function("interner_10000_high_duplication", |b| {
        b.iter(|| {
            let mut interner = StringInterner::new();
            for i in 0..10000 {
                let s = repeated[i as usize % repeated.len()];
                black_box(interner.intern(s));
            }
            assert_eq!(interner.len(), repeated.len());
            black_box(interner.bytes_saved())
        });
    });
}

fn bench_string_concat(c: &mut Criterion) {
    let parts: Vec<String> = (0..100)
        .map(|i| {
            format!(
                "The quick brown fox jumps over the lazy dog. Sentence {}.",
                i
            )
        })
        .collect();

    c.bench_function("string_concat_with_capacity", |b| {
        b.iter(|| {
            let total: usize = parts.iter().map(|s| s.len()).sum();
            let mut result = String::with_capacity(total);
            for s in &parts {
                result.push_str(s);
            }
            black_box(result)
        });
    });

    c.bench_function("string_concat_without_capacity", |b| {
        b.iter(|| {
            let mut result = String::new();
            for s in &parts {
                result.push_str(s);
            }
            black_box(result)
        });
    });
}

criterion_group!(
    benches,
    bench_v2_compile_sizes,
    bench_lir_pipeline,
    bench_pdf_generation,
    bench_interner,
    bench_string_concat,
);
criterion_main!(benches);
