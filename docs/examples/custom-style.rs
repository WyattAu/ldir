//! Example: Custom style application with fonts, sizes, and colors.
//!
//! Demonstrates how to construct an S-IR document with ApplyStyle
//! instructions and inspect the resulting G-IR style commands.
//!
//! Usage: cargo run --example custom-style --package ldir-core

use ldir_core::compiler::compile_sir;
use ldir_core::emitter::emit_gir;
use ldir_core::validator::validate_sir;
use ldir_ir::gir::{GIROpcode, GIRStyle, StyleTable};
use ldir_ir::sir::{BlockType, ROOT_SENTINEL, SIRDocument, SIRInstruction, SIROpcode};

fn main() {
    let mut doc = SIRDocument::new();

    // Root document node
    doc.push_with_payload(
        SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0),
        &[BlockType::Document as u8],
    );

    // Apply heading style (24pt, font ID 1)
    doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 1, 0, 0));

    // Heading content
    doc.push_with_payload(
        SIRInstruction::new(SIROpcode::SetContent, 2, 0, 0),
        b"Custom Style Example\x00",
    );

    // Apply body style (12pt, font ID 0, dark gray)
    doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 3, 0, 0));

    // Body content
    doc.push_with_payload(
        SIRInstruction::new(SIROpcode::SetContent, 4, 0, 0),
        b"This demonstrates custom font sizes and colors.\x00",
    );

    validate_sir(&doc).expect("well-formed S-IR");
    let gir = compile_sir(&doc).expect("compilation succeeds");
    assert!(gir.is_well_formed());

    // Build a style table for inspection
    let mut styles = StyleTable::new();
    styles.insert(GIRStyle::with_color(1, 1, 24 * 64, 0, 0, 0, 255));
    styles.insert(GIRStyle::with_color(2, 0, 12 * 64, 51, 51, 51, 255));

    // Print compiled commands
    for (page_idx, page) in gir.iter().enumerate() {
        println!("Page {} ({} commands):", page_idx, page.len());
        for cmd in page.iter() {
            match cmd.opcode() {
                GIROpcode::SetFont => {
                    let font_id = cmd.arg(0).unwrap_or(0);
                    if let Some(style) = styles.get(font_id as u32) {
                        println!(
                            "  SetFont: id={}, font={}, size={:.1}pt, color=({},{},{})",
                            style.id,
                            style.font_id,
                            style.size_f64(),
                            style.color_r,
                            style.color_g,
                            style.color_b
                        );
                    } else {
                        println!("  SetFont: font_id={}", font_id);
                    }
                }
                GIROpcode::PushStack => println!("  PushStack"),
                GIROpcode::PopStack => println!("  PopStack"),
                GIROpcode::MoveXY => {
                    let x = cmd.arg(0).unwrap_or(0) as f64 / 64.0;
                    let y = cmd.arg(1).unwrap_or(0) as f64 / 64.0;
                    println!("  MoveXY: ({:.1}, {:.1})", x, y);
                }
                GIROpcode::PutGlyph => {
                    let glyph = cmd.arg(0).unwrap_or(0);
                    let adv = cmd.arg(1).unwrap_or(0) as f64 / 64.0;
                    println!(
                        "  PutGlyph: char={}, advance={:.1}pt",
                        glyph as u8 as char, adv
                    );
                }
                _ => println!("  {:?}", cmd.opcode()),
            }
        }
    }

    let bytes = emit_gir(&gir);
    println!("\nEmitted {} bytes", bytes.len());
    println!("Custom style example passed.");
}
