//! Example: TeX/LaTeX input through the LDIR pipeline.
//!
//! Demonstrates parsing a LaTeX string into S-IR v1, inspecting the
//! resulting instruction stream, and converting to S-IR v2 for
//! downstream compilation.
//!
//! ```sh
//! cargo run --example tex-basic
//! ```

fn main() {
    use ldir_ir::sir::{BlockType, SIROpcode};
    use ldir_tex::parse_tex;

    let tex = r"\documentclass{article}\begin{document}Hello, \textbf{World}!\end{document}";
    let doc = parse_tex(tex);

    println!("Parsed {} S-IR instructions", doc.len());

    for instr in doc.iter() {
        match instr.opcode() {
            SIROpcode::PushBlock => {
                if let Some(payload) = doc.payload().get(instr.payload_offset(), 1) {
                    if let Some(bt) = BlockType::from_u8(payload[0]) {
                        println!("  PushBlock({:?})", bt);
                    }
                }
            }
            SIROpcode::SetContent => {
                if let Some(text) = doc.payload_text(instr) {
                    println!("  SetContent(\"{}\")", text);
                }
            }
            SIROpcode::ApplyStyle => {
                println!("  ApplyStyle");
            }
            SIROpcode::LinkData => {
                if let Some(url) = doc.payload_text(instr) {
                    println!("  LinkData(\"{}\")", url);
                }
            }
            _ => {}
        }
    }

    if !doc.footnotes.is_empty() {
        println!("Footnotes: {}", doc.footnotes.len());
        for (num, text) in &doc.footnotes {
            println!("  [{}]: {}", num, text);
        }
    }

    let module = ldir_core::compiler::v1_to_v2::convert_v1_to_v2(&doc);
    println!("\nConverted to S-IR v2: {} nodes", module.body.len());

    let mut heading_count = 0;
    let mut paragraph_count = 0;
    for node in module.body.iter() {
        match node.node_type {
            ldir_ir::sir::v2::NodeType::Section
            | ldir_ir::sir::v2::NodeType::Subsection
            | ldir_ir::sir::v2::NodeType::Subsubsection => heading_count += 1,
            ldir_ir::sir::v2::NodeType::Paragraph => paragraph_count += 1,
            _ => {}
        }
    }
    println!(
        "  Headings: {}, Paragraphs: {}",
        heading_count, paragraph_count
    );
}
