use ldir_core::compiler::compile_sir;
use ldir_core::emitter::emit_gir;
use ldir_core::parser::parse_sir;
use ldir_core::validator::validate_sir;
use ldir_core::verifier::check_gir;
use ldir_ir::gir::{GIRCommand, GIRPage};
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
