# ldir-tex

TeX/LaTeX to S-IR parser for the LDIR document pipeline.

Converts a practical subset of LaTeX into an S-IR document tree suitable for compilation by `ldir-core`.

## Supported LaTeX

| Element | S-IR Mapping |
|---|---|
| `\section`–`\subsubsection` | `PushBlock(Heading)` with level payload |
| `\textbf`, `\textit`, `\texttt` | `ApplyStyle(BOLD/ITALIC/MONO)` |
| `\emph` | `ApplyStyle(ITALIC)` |
| `\[...\]`, `$$...$$` | `PushBlock(Math)` |
| `\begin{equation}` | `PushBlock(Math)` with numbered flag |
| `\begin{itemize}`, `\begin{enumerate}` | `PushBlock(List)` |
| `\begin{verbatim}` | `PushBlock(Code)` |
| `\begin{quote}`, `\begin{abstract}` | `PushBlock(BlockQuote)` |
| `\begin{figure}`, `\includegraphics` | `PushBlock(Figure)` + `PushBlock(Image)` |
| `\begin{table}`, `\begin{tabular}` | `PushBlock(Table)` + rows/cells |
| `\footnote{}` | Footnote mark + stored footnote text |
| `\label`, `\ref`, `\eqref` | Passed through as text |
| Greek letters, operators | Unicode substitution |

## Example

```rust
use ldir_tex::parse_tex;

let latex = r#"\section{Introduction}
This is \textbf{bold} and \textit{italic}.
\begin{equation}E = mc^2\end{equation}
"#;

let doc = parse_tex(latex);
println!("Parsed {} S-IR instructions", doc.len());

// Validate and compile
ldir_core::validator::validate_sir(&doc).expect("well-formed");
let gir = ldir_core::compiler::compile_sir(&doc).unwrap();
```

## License

MIT OR Apache-2.0
