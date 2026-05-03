# ldir-tex

TeX/LaTeX to S-IR parser for the LDIR document pipeline. Converts a
practical subset of LaTeX into an S-IR document tree suitable for
compilation by `ldir-core`.

## Features

- LaTeX lexer with control sequence, brace, and environment handling
- Sectioning: `\section` through `\subsubsection`
- Text styling: `\textbf`, `\textit`, `\texttt`, `\emph`
- Math: inline `$...$`, display `$$...$$`, `\begin{equation}`
- Greek letters and math operators with Unicode substitution
- Lists: `\begin{itemize}`, `\begin{enumerate}`
- Environments: `quote`, `verbatim`, `abstract`, `figure`, `table`
- Footnotes with counter management
- Cross-references: `\label`, `\ref`, `\eqref` (passthrough)

## Supported LaTeX

| Element | S-IR Mapping |
|---------|-------------|
| `\section`–`\subsubsection` | `PushBlock(Heading)` with level |
| `\textbf`, `\textit`, `\texttt` | `ApplyStyle(BOLD/ITALIC/MONO)` |
| `\emph` | `ApplyStyle(ITALIC)` |
| `$...$`, `$$...$$` | Inline / display math |
| `\begin{equation}` | `PushBlock(Math)` with numbered flag |
| `\begin{itemize}`, `\begin{enumerate}` | `PushBlock(List)` |
| `\begin{verbatim}` | `PushBlock(Code)` |
| `\begin{quote}`, `\begin{abstract}` | `PushBlock(BlockQuote)` |
| `\begin{figure}`, `\includegraphics` | `PushBlock(Figure)` + `PushBlock(Image)` |
| `\begin{table}`, `\begin{tabular}` | `PushBlock(Table)` + rows/cells |
| `\footnote{}` | Footnote mark + stored text |
| Greek letters, operators | Unicode substitution |

## API Overview

| Function | Description |
|----------|-------------|
| `parse_tex` | Parse a LaTeX string into `SIRDocument` |

## Usage

```rust
use ldir_tex::parse_tex;

let doc = parse_tex(r"\section{Intro}\textbf{Bold} and $x^2$.");
println!("Parsed {} S-IR instructions", doc.len());
```

## Input / Output

- **Input**: LaTeX source string (preamble is silently skipped)
- **Output**: `SIRDocument` (from `ldir-ir::sir`)

## License

MIT OR Apache-2.0

## Repository

[https://github.com/WyattAu/ldir](https://github.com/WyattAu/ldir)
