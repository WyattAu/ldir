use ldir_core::compiler::compile_sir;
use ldir_core::emitter::{emit_gir, parse_gir};
use ldir_core::parser::parse_sir;
use ldir_core::validator::validate_sir;
use ldir_core::verifier::check_gir;
use ldir_ir::sir::{ROOT_SENTINEL, SIRInstruction, SIROpcode};

#[test]
fn test_parse_validate_compile_emit_verify_simple() {
    let mut doc = ldir_ir::sir::SIRDocument::new();
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        0,
        ROOT_SENTINEL,
        0,
    ));
    doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));

    validate_sir(&doc).expect("S-IR validation should pass");

    let gir = compile_sir(&doc).expect("compilation should succeed");
    assert!(gir.is_well_formed());
    assert!(check_gir(&gir).is_ok());

    let bytes = emit_gir(&gir);
    let parsed = parse_gir(&bytes).expect("G-IR parse should succeed");
    assert_eq!(gir, parsed);
}

#[test]
fn test_parse_validate_compile_emit_verify_nested() {
    let mut doc = ldir_ir::sir::SIRDocument::new();
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        0,
        ROOT_SENTINEL,
        0,
    ));
    doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));
    doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 2, 0, 0));
    doc.push(SIRInstruction::new(SIROpcode::SetContent, 3, 0, 0));

    validate_sir(&doc).expect("S-IR validation should pass");

    let gir = compile_sir(&doc).expect("compilation should succeed");
    assert!(gir.is_well_formed());
    assert!(check_gir(&gir).is_ok());

    let bytes = emit_gir(&gir);
    let parsed = parse_gir(&bytes).expect("G-IR parse should succeed");
    assert_eq!(gir, parsed);
}

#[test]
fn test_full_pipeline_with_serialization() {
    let mut doc = ldir_ir::sir::SIRDocument::new();
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        0,
        ROOT_SENTINEL,
        0,
    ));
    doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));
    doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 2, 0, 0));
    doc.push(SIRInstruction::new(SIROpcode::InsertMath, 3, 0, 0));
    doc.push(SIRInstruction::new(SIROpcode::LinkData, 4, 0, 0));

    let sir_bytes = doc.to_bytes();
    let reparsed_sir = parse_sir(&sir_bytes).expect("S-IR parse should succeed");
    assert_eq!(doc, reparsed_sir);

    validate_sir(&reparsed_sir).expect("S-IR validation should pass");
    let gir = compile_sir(&reparsed_sir).expect("compilation should succeed");
    assert!(check_gir(&gir).is_ok());

    let gir_bytes = emit_gir(&gir);
    let reparsed_gir = parse_gir(&gir_bytes).expect("G-IR parse should succeed");
    assert_eq!(gir, reparsed_gir);
}

#[test]
fn test_invalid_sir_rejected_by_validator() {
    let mut doc = ldir_ir::sir::SIRDocument::new();
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        0,
        ROOT_SENTINEL,
        0,
    ));
    doc.push(SIRInstruction::new(SIROpcode::SetContent, 0, 0, 0));

    let result = validate_sir(&doc);
    assert!(result.is_err(), "duplicate entity ID should be rejected");
}

#[test]
fn test_cyclic_sir_rejected() {
    let mut doc = ldir_ir::sir::SIRDocument::new();
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        0,
        ROOT_SENTINEL,
        0,
    ));
    doc.push(SIRInstruction::new(SIROpcode::PushBlock, 1, 2, 0));
    doc.push(SIRInstruction::new(SIROpcode::PushBlock, 2, 1, 0));

    let result = validate_sir(&doc);
    assert!(result.is_err(), "cyclic document should be rejected");
}

#[test]
fn test_compiled_gir_passes_verifier() {
    let mut doc = ldir_ir::sir::SIRDocument::new();
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        0,
        ROOT_SENTINEL,
        0,
    ));
    for i in 1..20u32 {
        doc.push(SIRInstruction::new(SIROpcode::SetContent, i, 0, 0));
    }

    validate_sir(&doc).unwrap();
    let gir = compile_sir(&doc).unwrap();
    assert!(
        check_gir(&gir).is_ok(),
        "compiled output should pass verifier"
    );
}

#[test]
fn test_empty_sir_fails_pipeline() {
    let doc = ldir_ir::sir::SIRDocument::new();
    assert!(validate_sir(&doc).is_err());
    assert!(compile_sir(&doc).is_err());
}

#[test]
fn test_gir_emit_parse_roundtrip() {
    use ldir_ir::gir::{GIRCommand, GIRDocument, GIRPage};

    let mut doc = GIRDocument::new();
    let mut page = GIRPage::new();
    page.push(GIRCommand::new_set_font(1));
    page.push(GIRCommand::new_push_stack());
    page.push(GIRCommand::new_move_xy(640, 1280));
    page.push(GIRCommand::new_put_glyph(65, 640));
    page.push(GIRCommand::new_pop_stack());
    page.push(GIRCommand::new_draw_rule(0, 0, 468, 64));
    page.push(GIRCommand::new_attach_metadata(0, 10, 4, 8));
    doc.push_page(page);

    let bytes = emit_gir(&doc);
    let parsed = parse_gir(&bytes).unwrap();
    assert_eq!(doc, parsed);
}

#[test]
fn test_multi_page_pipeline() {
    let mut doc = ldir_ir::sir::SIRDocument::new();
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        0,
        ROOT_SENTINEL,
        0,
    ));
    for i in 1..10u32 {
        doc.push(SIRInstruction::new(SIROpcode::SetContent, i, 0, 0));
    }

    validate_sir(&doc).unwrap();
    let gir = compile_sir(&doc).unwrap();
    assert!(gir.is_well_formed());
    assert!(check_gir(&gir).is_ok());

    let bytes = emit_gir(&gir);
    let parsed = parse_gir(&bytes).unwrap();
    assert_eq!(gir, parsed);
}
