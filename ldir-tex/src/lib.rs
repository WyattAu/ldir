//! ldir-tex — TeX/LaTeX to S-IR parser.
//!
//! Converts a practical subset of LaTeX into an S-IR document suitable for
//! compilation by the LDIR compiler pipeline.

mod lexer;
mod parser;

use ldir_ir::sir::SIRDocument;

pub fn parse_tex(tex: &str) -> SIRDocument {
    let tokens = lexer::TeXLexer::new(tex).tokenize();
    let mut p = parser::TeXParser::new(&tokens);
    p.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::{BlockType, SIROpcode, StyleModifier};

    fn collect_all_text(doc: &SIRDocument) -> String {
        let mut out = String::new();
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::SetContent {
                if let Some(text) = doc.payload_text(instr) {
                    out.push_str(text);
                    out.push(' ');
                }
            }
        }
        out
    }

    fn find_block_type(doc: &SIRDocument, bt: BlockType) -> bool {
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::PushBlock {
                if let Some(payload) = doc.payload().get(instr.payload_offset(), 1) {
                    if payload == &[bt as u8] {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn find_content(doc: &SIRDocument, needle: &str) -> bool {
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::SetContent {
                if let Some(text) = doc.payload_text(instr) {
                    if text.contains(needle) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn has_style(doc: &SIRDocument, flag: u8) -> bool {
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::ApplyStyle {
                let packed = instr.payload_offset();
                let (mods, is_push) = StyleModifier::from_packed(packed);
                if is_push && mods.contains(flag) {
                    return true;
                }
            }
        }
        false
    }

    fn has_link(doc: &SIRDocument) -> bool {
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::LinkData {
                return true;
            }
        }
        false
    }

    #[test]
    fn test_empty_input() {
        let doc = parse_tex("");
        assert!(doc.len() >= 1, "should have at least root block");
    }

    #[test]
    fn test_single_paragraph() {
        let doc = parse_tex("Hello world");
        assert!(doc.len() >= 3, "root + paragraph + content");
        assert!(find_block_type(&doc, BlockType::Paragraph));
        assert!(find_content(&doc, "Hello world"));
    }

    #[test]
    fn test_section_heading() {
        let doc = parse_tex(r"\section{Introduction}");
        assert!(find_block_type(&doc, BlockType::Heading));
        assert!(find_content(&doc, "Introduction"));
    }

    #[test]
    fn test_subsection_heading() {
        let doc = parse_tex(r"\subsection{Details}");
        assert!(find_block_type(&doc, BlockType::Heading));
        assert!(find_content(&doc, "Details"));
    }

    #[test]
    fn test_subsubsection_heading() {
        let doc = parse_tex(r"\subsubsection{Nitty Gritty}");
        assert!(find_block_type(&doc, BlockType::Heading));
        assert!(find_content(&doc, "Nitty Gritty"));
    }

    #[test]
    fn test_multiple_sections() {
        let doc = parse_tex(
            r"\section{One}\section{Two}\section{Three}",
        );
        let text = collect_all_text(&doc);
        assert!(text.contains("One"));
        assert!(text.contains("Two"));
        assert!(text.contains("Three"));
    }

    #[test]
    fn test_textbf_emits_bold_style() {
        let doc = parse_tex(r"\textbf{bold text}");
        assert!(has_style(&doc, StyleModifier::BOLD));
        assert!(find_content(&doc, "bold text"));
    }

    #[test]
    fn test_textit_emits_italic_style() {
        let doc = parse_tex(r"\textit{italic text}");
        assert!(has_style(&doc, StyleModifier::ITALIC));
        assert!(find_content(&doc, "italic text"));
    }

    #[test]
    fn test_texttt_emits_mono_style() {
        let doc = parse_tex(r"\texttt{code text}");
        assert!(has_style(&doc, StyleModifier::MONO));
        assert!(find_content(&doc, "code text"));
    }

    #[test]
    fn test_emph_emits_italic_style() {
        let doc = parse_tex(r"\emph{emphasized}");
        assert!(has_style(&doc, StyleModifier::ITALIC));
        assert!(find_content(&doc, "emphasized"));
    }

    #[test]
    fn test_nested_textbf_textit() {
        let doc = parse_tex(r"\textbf{bold \textit{and italic}}");
        assert!(has_style(&doc, StyleModifier::BOLD));
        let text = collect_all_text(&doc);
        assert!(text.contains("bold"));
    }

    #[test]
    fn test_inline_math() {
        let doc = parse_tex(r"$E = mc^2$");
        assert!(has_style(&doc, StyleModifier::MONO));
        assert!(find_content(&doc, "E = mc^2"));
    }

    #[test]
    fn test_display_math_equation() {
        let doc = parse_tex(r"\begin{equation}E = mc^2\end{equation}");
        assert!(find_block_type(&doc, BlockType::Math));
        assert!(find_content(&doc, "E = mc^2"));
    }

    #[test]
    fn test_display_math_double_dollar() {
        let doc = parse_tex("$$E = mc^2$$");
        assert!(find_block_type(&doc, BlockType::Math));
        assert!(find_content(&doc, "E = mc^2"));
    }

    #[test]
    fn test_greek_letter_substitution() {
        let doc = parse_tex(r"$\alpha + \beta = \gamma$");
        let text = collect_all_text(&doc);
        assert!(text.contains("α"), "should substitute \\alpha, got: {text}");
        assert!(text.contains("β"), "should substitute \\beta, got: {text}");
        assert!(text.contains("γ"), "should substitute \\gamma, got: {text}");
    }

    #[test]
    fn test_frac_substitution() {
        let doc = parse_tex(r"$\frac{a}{b}$");
        assert!(find_content(&doc, "a/b"));
    }

    #[test]
    fn test_sqrt_substitution() {
        let doc = parse_tex(r"$\sqrt{x}$");
        assert!(find_content(&doc, "√x"));
    }

    #[test]
    fn test_itemize_list() {
        let doc = parse_tex(
            r"\begin{itemize}\item first\item second\end{itemize}",
        );
        assert!(find_block_type(&doc, BlockType::List));
        assert!(find_content(&doc, "first"));
        assert!(find_content(&doc, "second"));
    }

    #[test]
    fn test_enumerate_list() {
        let doc = parse_tex(
            r"\begin{enumerate}\item one\item two\end{enumerate}",
        );
        assert!(find_block_type(&doc, BlockType::List));
        assert!(find_content(&doc, "one"));
    }

    #[test]
    fn test_quote_environment() {
        let doc = parse_tex(r"\begin{quote}Quoted text\end{quote}");
        assert!(find_block_type(&doc, BlockType::BlockQuote));
        assert!(find_content(&doc, "Quoted text"));
    }

    #[test]
    fn test_verbatim_environment() {
        let doc = parse_tex(r"\begin{verbatim}for x in y:\end{verbatim}");
        assert!(find_block_type(&doc, BlockType::Code));
        assert!(find_content(&doc, "for x in y:"));
    }

    #[test]
    fn test_abstract_environment() {
        let doc = parse_tex(r"\begin{abstract}This is the abstract.\end{abstract}");
        assert!(find_block_type(&doc, BlockType::BlockQuote));
        assert!(find_content(&doc, "This is the abstract"));
    }

    #[test]
    fn test_url_command() {
        let doc = parse_tex(r"\url{https://example.com}");
        assert!(has_link(&doc));
        assert!(find_content(&doc, "https://example.com"));
    }

    #[test]
    fn test_uppercase_greek() {
        let doc = parse_tex(r"$\Gamma \Delta \Sigma$");
        let text = collect_all_text(&doc);
        assert!(text.contains('Γ'));
        assert!(text.contains('Δ'));
        assert!(text.contains('Σ'));
    }

    #[test]
    fn test_math_operators() {
        let doc = parse_tex(r"$\sum \prod \int$");
        let text = collect_all_text(&doc);
        assert!(text.contains('∑'));
        assert!(text.contains('∏'));
        assert!(text.contains('∫'));
    }

    #[test]
    fn test_math_relations() {
        let doc = parse_tex(r"$\leq \geq \neq \approx$");
        let text = collect_all_text(&doc);
        assert!(text.contains('≤'));
        assert!(text.contains('≥'));
        assert!(text.contains('≠'));
        assert!(text.contains('≈'));
    }

    #[test]
    fn test_label_ref_passthrough() {
        let doc = parse_tex(r"\section{Intro}\label{sec:intro}See \ref{sec:intro}");
        let text = collect_all_text(&doc);
        assert!(text.contains("\\label{sec:intro}"), "should pass \\label through");
        assert!(text.contains("\\ref{sec:intro}"), "should pass \\ref through");
    }

    #[test]
    fn test_eqref_passthrough() {
        let doc = parse_tex(r"See \eqref{eq:pyth}");
        let text = collect_all_text(&doc);
        assert!(text.contains("\\eqref{eq:pyth}"));
    }

    #[test]
    fn test_numbered_equation_flag() {
        let doc = parse_tex(r"\begin{equation}E = mc^2\end{equation}");
        let mut found_numbered = false;
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::PushBlock {
                let payload = doc.payload().get(instr.payload_offset(), 2);
                if let Some(bytes) = payload {
                    if bytes[0] == BlockType::Math as u8 && bytes.len() >= 2 && bytes[1] == 1 {
                        found_numbered = true;
                    }
                }
            }
        }
        assert!(found_numbered, "equation env should have numbered flag");
    }

    #[test]
    fn test_unnumbered_equation_star() {
        let doc = parse_tex(r"\begin{equation*}E = mc^2\end{equation*}");
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::PushBlock {
                let payload = doc.payload().get(instr.payload_offset(), 2);
                if let Some(bytes) = payload {
                    if bytes[0] == BlockType::Math as u8 {
                        assert_eq!(bytes[1], 0, "equation* should not be numbered");
                        return;
                    }
                }
            }
        }
        panic!("no Math block found");
    }

    #[test]
    fn test_cases_in_equation() {
        let doc = parse_tex(
            r"\begin{equation}f(x) = \begin{cases} x^2 & \text{if } x > 0 \\ -x^2 & \text{if } x < 0 \end{cases}\end{equation}",
        );
        assert!(find_block_type(&doc, BlockType::Math));
        let text = collect_all_text(&doc);
        assert!(text.contains("\\begin{cases}"), "cases env should be preserved, got: {text}");
        assert!(text.contains("\\end{cases}"), "end cases should be preserved");
    }

    #[test]
    fn test_pmatrix_in_equation() {
        let doc = parse_tex(
            r"\begin{equation}A = \begin{pmatrix} 1 & 2 \\ 3 & 4 \end{pmatrix}\end{equation}",
        );
        assert!(find_block_type(&doc, BlockType::Math));
        let text = collect_all_text(&doc);
        assert!(text.contains("\\begin{pmatrix}"), "pmatrix should be preserved, got: {text}");
        assert!(text.contains("\\end{pmatrix}"));
    }

    #[test]
    fn test_left_right_passthrough() {
        let doc = parse_tex(
            r"\begin{equation}\left( \frac{a}{b} \right)\end{equation}",
        );
        assert!(find_block_type(&doc, BlockType::Math));
        let text = collect_all_text(&doc);
        assert!(text.contains("\\left("), "left paren should be preserved, got: {text}");
        assert!(text.contains("\\right)"));
    }

    #[test]
    fn test_left_right_braces_passthrough() {
        let doc = parse_tex(
            r"\begin{equation}\left\{ x \right.\end{equation}",
        );
        assert!(find_block_type(&doc, BlockType::Math));
        let text = collect_all_text(&doc);
        assert!(text.contains("\\left{"), "left brace should be preserved, got: {text}");
        assert!(text.contains("\\right."));
    }

    #[test]
    fn test_text_in_math() {
        let doc = parse_tex(r"$\text{hello}$");
        assert!(find_content(&doc, "\\text{hello}"));
    }

    #[test]
    fn test_footnote_emits_fnmark() {
        let doc = parse_tex(r"Hello\footnote{A note.}");
        assert!(find_content(&doc, "\\fnmark{1}"));
    }

    #[test]
    fn test_footnote_stores_text() {
        let doc = parse_tex(r"Hello\footnote{A note.}");
        assert_eq!(doc.footnotes.len(), 1);
        assert_eq!(doc.footnotes[0].0, 1);
        assert_eq!(doc.footnotes[0].1, "A note.");
    }

    #[test]
    fn test_multiple_footnotes() {
        let doc = parse_tex(r"Text\footnote{First.} more\footnote{Second.}");
        assert!(find_content(&doc, "\\fnmark{1}"));
        assert!(find_content(&doc, "\\fnmark{2}"));
        assert_eq!(doc.footnotes.len(), 2);
        assert_eq!(doc.footnotes[0].0, 1);
        assert_eq!(doc.footnotes[1].0, 2);
        assert_eq!(doc.footnotes[1].1, "Second.");
    }

    #[test]
    fn test_footnote_with_nested_braces() {
        let doc = parse_tex(r"Text\footnote{Note with {nested} braces.}");
        assert_eq!(doc.footnotes.len(), 1);
        assert_eq!(doc.footnotes[0].1, "Note with {nested} braces.");
    }

    #[test]
    fn test_footnote_in_paragraph() {
        let doc = parse_tex(
            r"\begin{document}This is text\footnote{A note.} and more.\end{document}",
        );
        assert!(find_content(&doc, "\\fnmark{1}"));
        assert_eq!(doc.footnotes.len(), 1);
    }
}

