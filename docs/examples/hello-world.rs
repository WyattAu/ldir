//! Example: Hello World — minimal LDIR document.
//!
//! Builds a simple document tree, validates it, compiles to G-IR,
//! and emits the binary output.
//!
//! Usage: cargo run --example hello-world --package ldir-core

use ldir_core::compiler::compile_sir;
use ldir_core::emitter::emit_gir;
use ldir_core::validator::validate_sir;
use ldir_ir::sir::{ROOT_SENTINEL, SIRDocument, SIRInstruction, SIROpcode};

fn main() {
    // 1. Create S-IR instructions
    let mut doc = SIRDocument::new();

    // Root document node (parent = ROOT_SENTINEL)
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        0,
        ROOT_SENTINEL,
        0,
    ));

    // Text content directly under the document root
    doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));

    // 2. Validate the S-IR document
    validate_sir(&doc).expect("S-IR document should be well-formed");

    // 3. Compile S-IR to G-IR
    let gir = compile_sir(&doc).expect("compilation should succeed");
    println!(
        "Compiled {} page(s), {} total commands",
        gir.page_count(),
        gir.total_commands()
    );
    assert!(gir.is_well_formed(), "G-IR must be well-formed");

    // 4. Emit G-IR to binary bytes
    let bytes = emit_gir(&gir);
    println!("Emitted {} bytes", bytes.len());
    assert!(bytes.len() >= 8, "minimum header size is 8 bytes");
    assert_eq!(&bytes[0..4], b"GIR0", "magic bytes must be GIR0");

    // 5. Verify round-trip
    let restored = ldir_core::emitter::parse_gir(&bytes).expect("round-trip parse should succeed");
    assert_eq!(gir, restored, "round-trip must be identical");

    println!("Hello World example passed all checks.");
}
