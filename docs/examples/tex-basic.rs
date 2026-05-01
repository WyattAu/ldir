//! Example: TeX-like input through LDIR.
//!
//! This example shows the intended workflow for parsing TeX-like
//! input and compiling it through the LDIR pipeline.
//!
//! NOTE: `ldir-tex` backend is not yet implemented.
//! This file serves as a design reference for the planned API.
//!
//! # Planned API (subject to change)
//!
//! ```ignore
//! use ldir_tex::parse_tex;
//! use ldir_core::{compile_sir, emit_gir, validator::validate_sir};
//!
//! // Parse TeX to S-IR
//! let sir = parse_tex(r"\documentclass{article}\begin{document}Hello\end{document}");
//!
//! // Validate, compile, and emit
//! validate_sir(&sir)?;
//! let gir = compile_sir(&sir)?;
//! let bytes = emit_gir(&gir);
//! ```
//!
//! # TeX mapping
//!
//! | TeX construct       | S-IR representation        |
//! |---------------------|---------------------------|
//! | `\documentclass`    | PushBlock(Document)       |
//! | `\begin{document}`  | PushBlock(Document)       |
//! | `\section{...}`     | PushBlock(Heading)        |
//! | `\begin{itemize}`   | PushBlock(List)           |
//! | Text content        | SetContent                |
//! | `\textbf{...}`      | ApplyStyle + SetContent   |
//! | `$...$`             | InsertMath                |
//! | `\href{...}{...}`   | LinkData                  |

fn main() {
    println!("tex-basic: ldir-tex backend is not yet implemented.");
    println!("See the module-level documentation for the planned API and TeX mapping.");
}
