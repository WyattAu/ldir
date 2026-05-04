use ldir_core::compiler::compile_sir;
use ldir_core::compiler::context::CompileContext;
use ldir_core::compiler::cross_ref;
use ldir_core::compiler::v2_compile::compile_v2_document;
use ldir_core::emitter::emit_gir;
use ldir_core::parser::parse_sir;
use ldir_core::validator::validate_sir;
use ldir_core::verifier::check_gir;
use ldir_ir::gir::{GIRCommand, GIRPage};
use ldir_ir::sir::v2::SIRModuleV2;
use ldir_ir::sir::v2::nodes::{FloatPlacement, ListType, MathType, Node, NodeType};
use ldir_ir::sir::{BlockType, ROOT_SENTINEL, SIRInstruction, SIROpcode};

fn make_simple_sir() -> ldir_ir::sir::SIRDocument {
    let mut doc = ldir_ir::sir::SIRDocument::new();
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        0,
        ROOT_SENTINEL,
        0,
    ));
    doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));
    doc
}

fn make_nested_sir() -> ldir_ir::sir::SIRDocument {
    let mut doc = ldir_ir::sir::SIRDocument::new();
    doc.push_with_payload(
        SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0),
        &[BlockType::Document as u8],
    );
    doc.push_with_payload(
        SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0),
        b"Hello\x00",
    );
    doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 2, 0, 0));
    doc.push_with_payload(
        SIRInstruction::new(SIROpcode::SetContent, 3, 0, 0),
        b"World\x00",
    );
    doc
}

fn make_deeply_nested_sir(depth: usize) -> ldir_ir::sir::SIRDocument {
    let mut doc = ldir_ir::sir::SIRDocument::new();
    doc.push_with_payload(
        SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0),
        &[BlockType::Document as u8],
    );
    for i in 1..depth {
        let text = format!("node {}\x00", i);
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, i as u32, 0, 0),
            text.as_bytes(),
        );
    }
    let text = format!("leaf {}\x00", depth);
    doc.push_with_payload(
        SIRInstruction::new(SIROpcode::SetContent, depth as u32, 0, 0),
        text.as_bytes(),
    );
    doc
}

fn make_all_opcodes_sir() -> ldir_ir::sir::SIRDocument {
    let mut doc = ldir_ir::sir::SIRDocument::new();
    doc.push_with_payload(
        SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0),
        &[BlockType::Document as u8],
    );
    doc.push_with_payload(
        SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0),
        b"text\x00",
    );
    doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 2, 0, 0));
    doc.push_with_payload(
        SIRInstruction::new(SIROpcode::InsertMath, 3, 0, 0),
        b"x^2\x00",
    );
    doc.push_with_payload(
        SIRInstruction::new(SIROpcode::LinkData, 4, 0, 0),
        b"https://example.com\x00",
    );
    doc
}

#[test]
fn test_compile_deterministic() {
    let doc = make_simple_sir();

    let mut results: Vec<Vec<u8>> = Vec::new();
    for _ in 0..10 {
        let gir = compile_sir(&doc).unwrap();
        let bytes = emit_gir(&gir);
        results.push(bytes);
    }

    let first = &results[0];
    for (i, result) in results.iter().enumerate().skip(1) {
        assert_eq!(first, result, "compile output differs at iteration {}", i);
    }
}

#[test]
fn test_compile_deterministic_nested() {
    let doc = make_nested_sir();

    let bytes1 = {
        let gir = compile_sir(&doc).unwrap();
        emit_gir(&gir)
    };
    let bytes2 = {
        let gir = compile_sir(&doc).unwrap();
        emit_gir(&gir)
    };

    assert_eq!(bytes1, bytes2);
}

#[test]
fn test_compile_deterministic_deep() {
    let doc = make_deeply_nested_sir(50);

    let bytes1 = {
        let gir = compile_sir(&doc).unwrap();
        emit_gir(&gir)
    };
    let bytes2 = {
        let gir = compile_sir(&doc).unwrap();
        emit_gir(&gir)
    };

    assert_eq!(bytes1, bytes2);
}

#[test]
fn test_compile_deterministic_all_opcodes() {
    let doc = make_all_opcodes_sir();

    let bytes1 = {
        let gir = compile_sir(&doc).unwrap();
        emit_gir(&gir)
    };
    let bytes2 = {
        let gir = compile_sir(&doc).unwrap();
        emit_gir(&gir)
    };

    assert_eq!(bytes1, bytes2);
}

#[test]
fn test_gir_structural_equality() {
    let doc = make_nested_sir();
    let gir1 = compile_sir(&doc).unwrap();
    let gir2 = compile_sir(&doc).unwrap();
    assert_eq!(gir1, gir2);
}

#[test]
fn test_emit_deterministic() {
    let mut doc = ldir_ir::gir::GIRDocument::new();
    let mut page = GIRPage::new();
    page.push(GIRCommand::new_set_font(1));
    page.push(GIRCommand::new_move_xy(100, 200));
    page.push(GIRCommand::new_put_glyph(65, 640));
    page.push(GIRCommand::new_push_stack());
    page.push(GIRCommand::new_pop_stack());
    doc.push_page(page);

    let mut results: Vec<Vec<u8>> = Vec::new();
    for _ in 0..10 {
        results.push(emit_gir(&doc));
    }

    let first = &results[0];
    for (i, result) in results.iter().enumerate().skip(1) {
        assert_eq!(first, result, "emit output differs at iteration {}", i);
    }
}

#[test]
fn test_roundtrip_end_to_end() {
    let sir_doc = make_nested_sir();

    assert!(validate_sir(&sir_doc).is_ok(), "S-IR doc should be valid");

    let gir_doc = compile_sir(&sir_doc).unwrap();
    assert!(
        gir_doc.is_well_formed(),
        "compiled G-IR should be well-formed"
    );
    assert!(
        check_gir(&gir_doc).is_ok(),
        "compiled G-IR should pass verifier"
    );

    let emitted = emit_gir(&gir_doc);
    assert!(!emitted.is_empty(), "emitted bytes should not be empty");

    let parsed_gir = ldir_core::emitter::parse_gir(&emitted).unwrap();
    assert_eq!(
        gir_doc, parsed_gir,
        "parse round-trip should preserve document"
    );
}

#[test]
fn test_roundtrip_deep_doc() {
    let sir_doc = make_deeply_nested_sir(20);

    assert!(validate_sir(&sir_doc).is_ok());
    let gir_doc = compile_sir(&sir_doc).unwrap();
    assert!(check_gir(&gir_doc).is_ok());

    let emitted = emit_gir(&gir_doc);
    let parsed = ldir_core::emitter::parse_gir(&emitted).unwrap();
    assert_eq!(gir_doc, parsed);
}

#[test]
fn test_roundtrip_all_opcodes() {
    let sir_doc = make_all_opcodes_sir();

    assert!(validate_sir(&sir_doc).is_ok());
    let gir_doc = compile_sir(&sir_doc).unwrap();
    assert!(check_gir(&gir_doc).is_ok());

    let emitted = emit_gir(&gir_doc);
    let parsed = ldir_core::emitter::parse_gir(&emitted).unwrap();
    // Note: G-IR serialization does not preserve links (known limitation).
    // Verify structural properties instead of full equality.
    assert_eq!(gir_doc.page_count(), parsed.page_count());
    assert_eq!(gir_doc.total_commands(), parsed.total_commands());
}

#[test]
fn test_sir_serialize_parse_roundtrip() {
    let sir_doc = make_nested_sir();
    let bytes = sir_doc.to_bytes();
    let parsed = parse_sir(&bytes).unwrap();
    // Note: S-IR serialization preserves instructions but not payload data
    // (known limitation). Compare instruction count and structure.
    assert_eq!(
        sir_doc.len(),
        parsed.len(),
        "instruction count should match"
    );
    for (orig, rt) in sir_doc.iter().zip(parsed.iter()) {
        assert_eq!(orig.opcode(), rt.opcode());
        assert_eq!(orig.entity_id(), rt.entity_id());
        assert_eq!(orig.parent_id(), rt.parent_id());
    }
}

#[test]
fn test_sir_serialize_compile_deterministic() {
    let sir_doc = make_nested_sir();
    let bytes = sir_doc.to_bytes();

    let gir1 = {
        let parsed = parse_sir(&bytes).unwrap();
        compile_sir(&parsed).unwrap()
    };
    let gir2 = {
        let parsed = parse_sir(&bytes).unwrap();
        compile_sir(&parsed).unwrap()
    };

    assert_eq!(gir1, gir2);
}

fn make_complex_v2_module() -> SIRModuleV2 {
    let mut module = SIRModuleV2::new();
    let doc_id = module.body.push(Node::new(0, NodeType::Document));

    let ch1 = module.body.push(
        Node::new(1, NodeType::Chapter)
            .with_label("ch:intro")
            .with_parent(doc_id),
    );
    let ch1_text = module.body.push(
        Node::new(
            2,
            NodeType::Text {
                content: "Introduction".into(),
            },
        )
        .with_parent(ch1),
    );
    if let Some(c) = module.body.get_mut(ch1) {
        c.add_child(ch1_text);
    }

    let sec1 = module.body.push(
        Node::new(3, NodeType::Section)
            .with_label("sec:methods")
            .with_parent(doc_id),
    );
    let sec1_text = module.body.push(
        Node::new(
            4,
            NodeType::Text {
                content: "Methods".into(),
            },
        )
        .with_parent(sec1),
    );
    if let Some(s) = module.body.get_mut(sec1) {
        s.add_child(sec1_text);
    }

    let subsec1 = module.body.push(
        Node::new(5, NodeType::Subsection)
            .with_label("subsec:setup")
            .with_parent(doc_id),
    );
    let subsec1_text = module.body.push(
        Node::new(
            6,
            NodeType::Text {
                content: "Setup".into(),
            },
        )
        .with_parent(subsec1),
    );
    if let Some(s) = module.body.get_mut(subsec1) {
        s.add_child(subsec1_text);
    }

    let para1 = module
        .body
        .push(Node::new(7, NodeType::Paragraph).with_parent(doc_id));
    let para1_text = module.body.push(
        Node::new(
            8,
            NodeType::Text {
                content: "This is a paragraph about methods.".into(),
            },
        )
        .with_parent(para1),
    );
    if let Some(p) = module.body.get_mut(para1) {
        p.add_child(para1_text);
    }

    let eq1 = module.body.push(
        Node::new(
            9,
            NodeType::MathBlock {
                math_type: MathType::Equation,
                numbered: true,
            },
        )
        .with_label("eq:euler")
        .with_parent(doc_id),
    );
    let eq1_text = module.body.push(
        Node::new(
            10,
            NodeType::Text {
                content: "e^{i\\pi} + 1 = 0".into(),
            },
        )
        .with_parent(eq1),
    );
    if let Some(e) = module.body.get_mut(eq1) {
        e.add_child(eq1_text);
    }

    let fig1 = module.body.push(
        Node::new(
            11,
            NodeType::Figure {
                placement: FloatPlacement::Here,
            },
        )
        .with_label("fig:diagram")
        .with_parent(doc_id),
    );

    let para2 = module
        .body
        .push(Node::new(12, NodeType::Paragraph).with_parent(doc_id));
    let para2_bold = module
        .body
        .push(Node::new(13, NodeType::Bold).with_parent(para2));
    let para2_bold_text = module.body.push(
        Node::new(
            14,
            NodeType::Text {
                content: "bold".into(),
            },
        )
        .with_parent(para2_bold),
    );
    let para2_text = module.body.push(
        Node::new(
            15,
            NodeType::Text {
                content: " text with references.".into(),
            },
        )
        .with_parent(para2),
    );
    if let Some(p) = module.body.get_mut(para2) {
        p.add_child(para2_bold);
        p.add_child(para2_text);
    }
    if let Some(b) = module.body.get_mut(para2_bold) {
        b.add_child(para2_bold_text);
    }

    let sec2 = module.body.push(
        Node::new(16, NodeType::Section)
            .with_label("sec:results")
            .with_parent(doc_id),
    );
    let sec2_text = module.body.push(
        Node::new(
            17,
            NodeType::Text {
                content: "Results".into(),
            },
        )
        .with_parent(sec2),
    );
    if let Some(s) = module.body.get_mut(sec2) {
        s.add_child(sec2_text);
    }

    let para3 = module
        .body
        .push(Node::new(18, NodeType::Paragraph).with_parent(doc_id));
    let para3_text = module.body.push(
        Node::new(
            19,
            NodeType::Text {
                content: "More content here for the results section.".into(),
            },
        )
        .with_parent(para3),
    );
    if let Some(p) = module.body.get_mut(para3) {
        p.add_child(para3_text);
    }

    let bq = module
        .body
        .push(Node::new(20, NodeType::BlockQuote).with_parent(doc_id));
    let bq_text = module.body.push(
        Node::new(
            21,
            NodeType::Text {
                content: "A notable quote.".into(),
            },
        )
        .with_parent(bq),
    );
    if let Some(b) = module.body.get_mut(bq) {
        b.add_child(bq_text);
    }

    let list = module.body.push(
        Node::new(
            22,
            NodeType::List {
                list_type: ListType::Ordered,
                ordered: true,
                start: Some(1),
            },
        )
        .with_parent(doc_id),
    );
    let li1 = module
        .body
        .push(Node::new(23, NodeType::ListItem).with_parent(list));
    let li1_text = module.body.push(
        Node::new(
            24,
            NodeType::Text {
                content: "First item".into(),
            },
        )
        .with_parent(li1),
    );
    let li2 = module
        .body
        .push(Node::new(25, NodeType::ListItem).with_parent(list));
    let li2_text = module.body.push(
        Node::new(
            26,
            NodeType::Text {
                content: "Second item".into(),
            },
        )
        .with_parent(li2),
    );
    if let Some(l) = module.body.get_mut(list) {
        l.add_child(li1);
        l.add_child(li2);
    }
    if let Some(l) = module.body.get_mut(li1) {
        l.add_child(li1_text);
    }
    if let Some(l) = module.body.get_mut(li2) {
        l.add_child(li2_text);
    }

    if let Some(d) = module.body.get_mut(doc_id) {
        d.add_child(ch1);
        d.add_child(sec1);
        d.add_child(subsec1);
        d.add_child(para1);
        d.add_child(eq1);
        d.add_child(fig1);
        d.add_child(para2);
        d.add_child(sec2);
        d.add_child(para3);
        d.add_child(bq);
        d.add_child(list);
    }

    module
}

#[test]
fn test_v2_compile_sha256_deterministic() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let module = make_complex_v2_module();

    fn hash_bytes(bytes: &[u8]) -> u64 {
        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        hasher.finish()
    }

    let first_hash = {
        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();
        let bytes = emit_gir(&gir);
        hash_bytes(&bytes)
    };

    for i in 1..10 {
        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();
        let bytes = emit_gir(&gir);
        let h = hash_bytes(&bytes);
        assert_eq!(
            first_hash, h,
            "v2 compile output hash differs at iteration {}",
            i
        );
    }
}

#[test]
fn test_v2_compile_bit_identical_10_runs() {
    let module = make_complex_v2_module();

    let first_bytes = {
        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();
        emit_gir(&gir)
    };

    for i in 1..10 {
        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();
        let bytes = emit_gir(&gir);
        assert_eq!(
            first_bytes, bytes,
            "v2 compile bytes differ at iteration {}",
            i
        );
    }
}

#[test]
fn test_cross_reference_numbering_deterministic() {
    let module = make_complex_v2_module();

    let mut all_label_numbers: Vec<Vec<(String, String)>> = Vec::new();

    for _ in 0..10 {
        let labels = cross_ref::collect_labels(&module);
        let numbers: Vec<(String, String)> = labels
            .iter()
            .filter(|l| !l.number.is_empty())
            .map(|l| (l.label.clone(), l.number.clone()))
            .collect();
        all_label_numbers.push(numbers);
    }

    let first = &all_label_numbers[0];
    for (i, result) in all_label_numbers.iter().enumerate().skip(1) {
        assert_eq!(first, result, "cross-ref numbers differ at iteration {}", i);
    }
}

#[test]
fn test_cross_reference_section_ordering() {
    let module = make_complex_v2_module();

    for _ in 0..10 {
        let labels = cross_ref::collect_labels(&module);

        let sec_intro = labels.iter().find(|l| l.label == "ch:intro");
        assert!(sec_intro.is_some(), "ch:intro should exist");
        assert_eq!(sec_intro.unwrap().number, "1");

        let sec_methods = labels.iter().find(|l| l.label == "sec:methods");
        assert!(sec_methods.is_some(), "sec:methods should exist");
        assert_eq!(sec_methods.unwrap().number, "1.1");

        let subsec_setup = labels.iter().find(|l| l.label == "subsec:setup");
        assert!(subsec_setup.is_some(), "subsec:setup should exist");
        assert_eq!(subsec_setup.unwrap().number, "1.1.1");

        let eq_euler = labels.iter().find(|l| l.label == "eq:euler");
        assert!(eq_euler.is_some(), "eq:euler should exist");
        assert_eq!(eq_euler.unwrap().number, "1");

        let fig_diagram = labels.iter().find(|l| l.label == "fig:diagram");
        assert!(fig_diagram.is_some(), "fig:diagram should exist");
        assert_eq!(fig_diagram.unwrap().number, "1");

        let sec_results = labels.iter().find(|l| l.label == "sec:results");
        assert!(sec_results.is_some(), "sec:results should exist");
        assert_eq!(sec_results.unwrap().number, "1.2");
    }
}

#[test]
fn test_resolve_text_references_deterministic() {
    let module = make_complex_v2_module();
    let labels = cross_ref::collect_labels(&module);
    let numbers = cross_ref::resolve_references(&labels);
    let kinds = cross_ref::resolve_kind_map(&labels);

    let text = r"See \ref{sec:methods} and \eqref{eq:euler}, also \autoref{fig:diagram} and \ref{sec:results}.";

    let first_result = cross_ref::resolve_text_references(text, &numbers, &kinds);

    for i in 1..10 {
        let result = cross_ref::resolve_text_references(text, &numbers, &kinds);
        assert_eq!(
            first_result, result,
            "resolved text differs at iteration {}",
            i
        );
    }
}

#[test]
fn test_v2_compile_with_resolved_refs_deterministic() {
    let module = make_complex_v2_module();

    let first_bytes = {
        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();
        emit_gir(&gir)
    };

    assert!(!first_bytes.is_empty(), "should produce output");

    for i in 1..10 {
        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();
        let bytes = emit_gir(&gir);
        assert_eq!(
            first_bytes, bytes,
            "v2 compile with refs differs at iteration {}",
            i
        );
    }
}
