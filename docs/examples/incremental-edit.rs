//! Example: Incremental recompilation.
//!
//! Demonstrates modifying a small part of an S-IR document and
//! recompiling to get updated G-IR output. This is useful for
//! interactive editors where only a portion of the document changes.
//!
//! Usage: cargo run --example incremental-edit --package ldir-core

use ldir_core::compiler::compile_sir;
use ldir_core::emitter::emit_gir;
use ldir_core::validator::validate_sir;
use ldir_ir::sir::{BlockType, ROOT_SENTINEL, SIRDocument, SIRInstruction, SIROpcode};

fn main() {
    // Build initial document with two content blocks
    let mut doc = SIRDocument::new();

    // Document root
    doc.push_with_payload(
        SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0),
        &[BlockType::Document as u8],
    );

    // First content block
    doc.push_with_payload(
        SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0),
        b"First content block\x00",
    );

    // Second content block
    doc.push_with_payload(
        SIRInstruction::new(SIROpcode::SetContent, 2, 0, 0),
        b"Second content block\x00",
    );

    validate_sir(&doc).expect("initial document well-formed");
    let gir_v1 = compile_sir(&doc).expect("v1 compiles");
    let bytes_v1 = emit_gir(&gir_v1);
    println!(
        "V1: {} pages, {} commands, {} bytes",
        gir_v1.page_count(),
        gir_v1.total_commands(),
        bytes_v1.len()
    );

    // --- User edit: add a style instruction ---

    // Insert an ApplyStyle instruction for the document root
    doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 3, 0, 0));

    validate_sir(&doc).expect("edited document well-formed");
    let gir_v2 = compile_sir(&doc).expect("v2 compiles");
    let bytes_v2 = emit_gir(&gir_v2);
    println!(
        "V2: {} pages, {} commands, {} bytes",
        gir_v2.page_count(),
        gir_v2.total_commands(),
        bytes_v2.len()
    );

    // V2 should have more commands due to the new style
    assert!(
        gir_v2.total_commands() >= gir_v1.total_commands(),
        "edited doc should have >= commands"
    );

    // Both must be well-formed
    assert!(gir_v1.is_well_formed());
    assert!(gir_v2.is_well_formed());

    // Outputs are deterministic: rebuild the v1 document from scratch
    let mut doc_v1 = SIRDocument::new();
    doc_v1.push_with_payload(
        SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0),
        &[BlockType::Document as u8],
    );
    doc_v1.push_with_payload(
        SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0),
        b"First content block\x00",
    );
    doc_v1.push_with_payload(
        SIRInstruction::new(SIROpcode::SetContent, 2, 0, 0),
        b"Second content block\x00",
    );
    let gir_v1_again = compile_sir(&doc_v1).expect("recompile v1");
    assert_eq!(gir_v1, gir_v1_again, "compilation is deterministic");

    println!("\nIncremental edit example passed.");
}
