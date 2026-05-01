//! Example: Markdown to PDF via LDIR pipeline.
//!
//! This example shows the intended workflow for converting Markdown
//! input to PDF output through the LDIR compilation pipeline.
//!
//! NOTE: `ldir-md` and `ldir-pdf` backends are not yet implemented.
//! This file serves as a design reference for the planned API.
//!
//! # Planned API (subject to change)
//!
//! ```ignore
//! use ldir_md::parse_markdown;
//! use ldir_core::{compile_sir, emit_gir, validator::validate_sir};
//! use ldir_pdf::render_pdf;
//!
//! // Parse Markdown to S-IR
//! let sir = parse_markdown("# Hello\n\nWorld");
//!
//! // Validate, compile, and emit
//! validate_sir(&sir)?;
//! let gir = compile_sir(&sir)?;
//! let bytes = emit_gir(&gir);
//!
//! // Render to PDF
//! let pdf_bytes = render_pdf(&gir)?;
//! std::fs::write("output.pdf", pdf_bytes)?;
//! ```

fn main() {
    println!("markdown-to-pdf: ldir-md and ldir-pdf backends are not yet implemented.");
    println!("See the module-level documentation for the planned API.");
}
