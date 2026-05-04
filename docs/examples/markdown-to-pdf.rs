//! Example: Markdown to PDF via the LDIR pipeline.
//!
//! Demonstrates parsing a Markdown string into S-IR v1, converting to
//! S-IR v2, and compiling through the LDIR pipeline to produce output.
//!
//! ```sh
//! cargo run --example markdown-to-pdf
//! ```

fn main() {
    use ldir_ir::sir::SIROpcode;
    use ldir_md::parse_markdown;

    let markdown = "# Hello\n\nThis has **bold** text.\n\n- Item 1\n- Item 2";
    let doc = parse_markdown(markdown);

    println!("Parsed {} S-IR instructions", doc.len());

    let mut all_text = String::new();
    for instr in doc.iter() {
        match instr.opcode() {
            SIROpcode::PushBlock => {
                if let Some(payload) = doc.payload().get(instr.payload_offset(), 1) {
                    if let Some(bt) = ldir_ir::sir::BlockType::from_u8(payload[0]) {
                        println!("  PushBlock({:?})", bt);
                    }
                }
            }
            SIROpcode::SetContent => {
                if let Some(text) = doc.payload_text(instr) {
                    println!("  SetContent(\"{}\")", text);
                    all_text.push_str(text);
                    all_text.push(' ');
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

    println!("\nExtracted text: {}", all_text.trim());

    let module = ldir_core::compiler::v1_to_v2::convert_v1_to_v2(&doc);
    println!("Converted to S-IR v2: {} nodes", module.body.len());

    let mut heading_count = 0;
    let mut paragraph_count = 0;
    let mut list_count = 0;
    for node in module.body.iter() {
        match node.node_type {
            ldir_ir::sir::v2::NodeType::Section
            | ldir_ir::sir::v2::NodeType::Subsection
            | ldir_ir::sir::v2::NodeType::Subsubsection => heading_count += 1,
            ldir_ir::sir::v2::NodeType::Paragraph => paragraph_count += 1,
            ldir_ir::sir::v2::NodeType::List { .. } => list_count += 1,
            _ => {}
        }
    }
    println!(
        "  Headings: {}, Paragraphs: {}, Lists: {}",
        heading_count, paragraph_count, list_count
    );

    println!("\nTo compile to PDF, use the ldc CLI:");
    println!("  ldc input.md -o output.pdf");
    println!("\nTo compile to HTML or plain text:");
    println!("  ldc input.md -f html -o output.html");
    println!("  ldc input.md -f txt  -o output.txt");
}
