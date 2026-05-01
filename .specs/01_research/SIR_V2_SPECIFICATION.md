# S-IR v2 Specification

> **Structured Intermediate Representation for Documents — Version 2.0.0**
> The "LLVM of Documents": a universal, structured IR for deterministic typesetting.

## Table of Contents

1. [Overview](#1-overview)
2. [Module Structure](#2-module-structure)
3. [Type System](#3-type-system)
4. [Node Type Catalog](#4-node-type-catalog)
5. [Style System](#5-style-system)
6. [Counter System](#6-counter-system)
7. [Cross-References](#7-cross-references)
8. [Text Format (.ldir)](#8-text-format-ldir)
9. [Binary Format](#9-binary-format)
10. [Versioning](#10-versioning)
11. [Tooling](#11-tooling)
12. [Examples](#12-examples)

---

## 1. Overview

### 1.1 What is S-IR?

S-IR (Structured Intermediate Representation) is a tree-structured, self-contained document
format designed as the central hub of the LDIR typesetting system. It serves as the interface
between frontends (Markdown, LaTeX, reStructuredText) and backends (PDF, HTML, terminal).

Unlike instruction-stream IRs, S-IR v2 represents documents as **modules** containing typed
node trees, metadata, styles, resources, and annotations — analogous to how LLVM IR represents
programs as modules of functions, types, and metadata.

### 1.2 Design Goals

| Goal | Description |
|------|-------------|
| **Determinism** | Identical input always produces identical output across all backends |
| **Round-trip fidelity** | Text ↔ binary conversion is lossless |
| **Frontend-agnostic** | No bias toward any source format (LaTeX, Markdown, etc.) |
| **Backend-portable** | Contains all information needed for any rendering target |
| **Diffable** | Text format is designed for version control |
| **Extensible** | Node types and properties can be added without breaking existing tools |
| **Minimal** | Only essential document semantics; no layout decisions |

### 1.3 Comparison to LLVM IR

| LLVM IR | S-IR v2 |
|---------|---------|
| Functions, basic blocks | Document nodes, node tree |
| Types (i32, float, struct) | Node types (Section, Paragraph, Text) |
| Metadata (debug info) | Document metadata, annotations |
| Module | SIRModuleV2 |
| IRBuilder | Frontend compilers (ldir-md, ldir-tex) |
| Optimization passes | Style resolution, counter assignment, ref resolution |
| Code generation (x86, ARM) | Backends (PDF, HTML, terminal) |

### 1.4 Relationship to Frontends and Backends

```
  LaTeX ──┐                      ┌── PDF
  Markdown┤── ldir-md/tex/as ──► │── HTML
  RST ────┘    S-IR v2 Module    └── Terminal
```

Frontends produce S-IR modules. Backends consume S-IR modules. Tools like `ldir-dis` and
`ldir-as` convert between text and binary representations.

---

## 2. Module Structure

### 2.1 SIRModuleV2

The top-level container for a document. A module is fully self-contained — it carries all
metadata, resources, styles, annotations, and content needed for rendering.

```
SIRModuleV2 {
    header: ModuleHeader,       // format version and provenance
    metadata: DocumentMetadata, // title, author, page geometry
    resources: ResourceDecls,   // fonts, colors, counters
    styles: StyleDecls,         // named style declarations
    annotations: Annotations,   // labels, cross-references
    body: NodeTree,             // the document tree
}
```

### 2.2 ModuleHeader

Tracks format version and source provenance.

| Field | Type | Description |
|-------|------|-------------|
| `magic` | `[u8; 4]` | Must be `b"LDIR"` |
| `version` | `(u8, u8, u8)` | Semantic version (major.minor.patch) |
| `ir_version` | `u16` | IR version number (2 for v2) |
| `source_format` | `Option<String>` | Original format ("latex", "markdown", etc.) |
| `source_path` | `Option<String>` | Original file path |
| `created` | `u64` | Unix timestamp of creation |

Default version: `(2, 0, 0)`.

### 2.3 DocumentMetadata

Document-level metadata fields.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `title` | `Option<String>` | `None` | Document title |
| `author` | `Option<String>` | `None` | Author name(s) |
| `subject` | `Option<String>` | `None` | Subject or topic |
| `date` | `Option<String>` | `None` | Publication date |
| `language` | `String` | `"en"` | BCP 47 language tag |
| `direction` | `Direction` | `Auto` | Text direction (LTR, RTL, Auto) |
| `document_class` | `Option<String>` | `None` | Class: "article", "book", "report" |
| `page_geometry` | `Option<PageGeometry>` | US Letter | Page size and margins |

### 2.4 PageGeometry

| Field | Type | Default |
|-------|------|---------|
| `width` | `Dimension` | `8.5in` |
| `height` | `Dimension` | `11.0in` |
| `margin_top` | `Dimension` | `1.0in` |
| `margin_bottom` | `Dimension` | `1.0in` |
| `margin_left` | `Dimension` | `1.0in` |
| `margin_right` | `Dimension` | `1.0in` |
| `column_count` | `u8` | `1` |
| `column_gap` | `Dimension` | `24pt` |

### 2.5 ResourceDecls

Collections of reusable resources.

```
ResourceDecls {
    fonts: Vec<FontDecl>,
    colors: Vec<ColorDecl>,
    counters: Vec<CounterDecl>,
}
```

### 2.6 StyleDecls

Named style declarations with optional parent-based inheritance.

```
StyleDecls {
    styles: Vec<StyleDecl>,
}
```

### 2.7 Annotations

Labels and cross-references for the document.

```
Annotations {
    labels: HashMap<String, LabelInfo>,
    refs: Vec<CrossRef>,
}
```

### 2.8 NodeTree (Body)

The document body is a flat list of nodes that form a tree via parent/child references.

```
NodeTree {
    nodes: Vec<Node>,      // all nodes in document order
    root_ids: Vec<u32>,    // IDs of root nodes (no parent)
}
```

Tree structure is encoded via `parent_id` and `child_ids` fields on each `Node`.

---

## 3. Type System

### 3.1 Dimension

Physical measurements with unit conversion.

```
enum Dimension {
    Pt(f64),       // points (1/72 inch)
    Mm(f64),       // millimeters
    In(f64),       // inches
    Cm(f64),       // centimeters
    Percent(f64),  // percentage of parent (context-dependent)
}
```

Conversion to points: `Pt(v)` → `v`, `In(v)` → `v × 72`, `Mm(v)` → `v × 72/25.4`,
`Cm(v)` → `v × 72/2.54`, `Percent(v)` → `0.0` (context-dependent).

### 3.2 Direction

```
enum Direction {
    LeftToRight,
    RightToLeft,
    Auto,          // detect from language/content
}
```

### 3.3 FontWeight

```
enum FontWeight {
    Thin, ExtraLight, Light, Regular, Medium,
    SemiBold, Bold, ExtraBold, Black,
}
```
Default: `Regular`.

### 3.4 FontStyle

```
enum FontStyle {
    Normal, Italic, Oblique,
}
```
Default: `Normal`.

### 3.5 FontSource

```
enum FontSource {
    System,              // system font lookup
    File(String),        // path to font file
    Embedded,            // font data in module (future)
}
```

### 3.6 ColorValue

```
struct ColorValue {
    r: u8, g: u8, b: u8,
    a: Option<u8>,  // alpha channel
}
```

### 3.7 CounterFormat

```
enum CounterFormat {
    Arabic,         // 1, 2, 3, ...
    RomanLower,     // i, ii, iii, ...
    RomanUpper,     // I, II, III, ...
    AlphaLower,     // a, b, c, ...
    AlphaUpper,     // A, B, C, ...
    Custom(String), // e.g., "(1)", "§1", "Appendix A"
}
```

### 3.8 CounterReset

```
enum CounterReset {
    Never, PerDocument, PerPart, PerChapter,
    PerSection, PerSubsection, PerPage,
}
```

### 3.9 TextAlign

```
enum TextAlign {
    Left, Right, Center, Justify,
}
```
Default: `Left`.

### 3.10 FloatPlacement

```
enum FloatPlacement {
    Here, Top, Bottom, Page, ForceHere,
}
```

### 3.11 MathType

```
enum MathType {
    Equation, Align, Gather, Multline, Cases,
    Matrix { delimiters: (Option<char>, Option<char>) },
}
```

### 3.12 ColumnAlign

```
enum ColumnAlign {
    Left, Right, Center, Justified,
}
```

### 3.13 ListType

```
enum ListType {
    Unordered, Ordered, Description,
}
```

### 3.14 LabelCategory

```
enum LabelCategory {
    Section, Equation, Figure, Table, Footnote, Page, Custom,
}
```

---

## 4. Node Type Catalog

All 34 node types organized by category.

### 4.1 Document Structure (7 types)

| Node Type | Tag | Description |
|-----------|-----|-------------|
| `Document` | `@document` | Root container for the entire document |
| `Part` | `@part` | Top-level division (books) |
| `Chapter` | `@chapter` | Major division |
| `Section` | `@section` | Standard section heading |
| `Subsection` | `@subsection` | Sub-section heading |
| `Subsubsection` | `@subsubsection` | Sub-sub-section heading |
| `Paragraph` | `@paragraph` | Paragraph block |

Heading levels: Part=0, Chapter=1, Section=2, Subsection=3, Subsubsection=4.

### 4.2 Lists (2 types)

| Node Type | Tag | Fields | Description |
|-----------|-----|--------|-------------|
| `List` | `@list` | `list_type`, `ordered`, `start` | List container |
| `ListItem` | `@list-item` | — | Single list item |

### 4.3 Block Content (5 types)

| Node Type | Tag | Fields | Description |
|-----------|-----|--------|-------------|
| `BlockQuote` | `@blockquote` | — | Quoted block |
| `CodeBlock` | `@code-block` | `language: Option<String>` | Preformatted code |
| `MathBlock` | `@equation` | `math_type`, `numbered: bool` | Display mathematics |
| `Table` | `@table` | `col_specs`, `num_cols` | Table container |
| `TableRow` | `@table-row` | `is_header: bool` | Table row |
| `TableCell` | `@table-cell` | `colspan: u8`, `rowspan: u8` | Table cell |

### 4.4 Inline Content (11 types)

| Node Type | Tag | Fields | Description |
|-----------|-----|--------|-------------|
| `Text` | `@text` | `content: String` | Text content |
| `Styled` | `@styled` | `style_name: String` | Custom-styled span |
| `Bold` | `@bold` | — | Bold text |
| `Italic` | `@italic` | — | Italic text |
| `Mono` | `@mono` | — | Monospace text |
| `Underline` | `@underline` | — | Underlined text |
| `Strikethrough` | `@strike` | — | Struck-through text |
| `SmallCaps` | `@smallcaps` | — | Small capitals |
| `Link` | `@link` | `url: String`, `title: Option<String>` | Hyperlink |
| `Image` | `@image` | `source`, `alt`, `width`, `height` | Inline image |
| `MathInline` | `@math` | `content: String` | Inline mathematics |
| `LineBreak` | `@br` | — | Explicit line break |

### 4.5 Floats (2 types)

| Node Type | Tag | Fields | Description |
|-----------|-----|--------|-------------|
| `Figure` | `@figure` | `placement: FloatPlacement` | Figure float |
| `Caption` | `@caption` | — | Figure/table caption |

### 4.6 Special (3 types)

| Node Type | Tag | Fields | Description |
|-----------|-----|--------|-------------|
| `Footnote` | `@footnote` | `content: String` | Footnote definition |
| `FootnoteBlock` | `@footnote-block` | — | Footnote collection |
| `TableOfContents` | `@toc` | `max_depth: u8` | Auto-generated TOC |
| `PageBreak` | `@page-break` | — | Explicit page break |
| `ThematicBreak` | `@hr` | — | Horizontal rule |

### 4.7 Container (1 type)

| Node Type | Tag | Description |
|-----------|-----|-------------|
| `Group` | `@group` | Anonymous grouping node |

### 4.8 Node Fields

Every node has:

| Field | Type | Description |
|-------|------|-------------|
| `id` | `u32` | Unique node identifier |
| `node_type` | `NodeType` | The type of this node |
| `parent_id` | `Option<u32>` | Parent node (None for roots) |
| `child_ids` | `Vec<u32>` | Ordered children |
| `label` | `Option<String>` | Cross-reference label |
| `style` | `Option<String>` | Named style to apply |
| `counter` | `Option<String>` | Counter to increment |

Builder pattern: `Node::new(id, type).with_parent(pid).with_label("sec:1")`.

---

## 5. Style System

### 5.1 Style Declarations

Styles are named declarations with optional parent-based inheritance.

```
StyleDecl {
    name: String,               // unique name
    parent: Option<String>,     // parent style to inherit from
    properties: StyleProperties,
}
```

### 5.2 Style Inheritance

When a style has a `parent`, it inherits all properties from the parent that are not
explicitly set in the child. Inheritance is resolved by the backend during rendering.

Resolution order: `node.style` → `style.parent` → `style.parent.parent` → ... → defaults.

### 5.3 Style Properties

| Property | Type | Description |
|----------|------|-------------|
| `font_name` | `Option<String>` | Font family reference |
| `font_size` | `Option<Dimension>` | Font size |
| `font_weight` | `Option<FontWeight>` | Boldness |
| `font_style` | `Option<String>` | "normal", "italic", "oblique" |
| `text_color` | `Option<String>` | Text color reference |
| `background_color` | `Option<String>` | Background color reference |
| `line_height` | `Option<f64>` | Line height multiplier |
| `paragraph_indent` | `Option<Dimension>` | Paragraph indent |
| `space_before` | `Option<Dimension>` | Space before block |
| `space_after` | `Option<Dimension>` | Space after block |
| `text_align` | `Option<TextAlign>` | Text alignment |
| `keep_with_next` | `Option<bool>` | Prevent orphaned block |
| `page_break_before` | `Option<bool>` | Force page break |
| `first_line_indent` | `Option<Dimension>` | First line indent |
| `margins` | `Option<(Dim, Dim, Dim, Dim)>` | Top, right, bottom, left margins |

All properties are optional; unset properties fall through to parent or defaults.

### 5.4 Style Application

Styles are applied to nodes via the `style` field:
```
@section [id=1, style="heading-1"] { }
```

Backends resolve the style chain and apply computed properties during layout.

---

## 6. Counter System

### 6.1 Counter Declarations

Counters produce auto-numbering for sections, equations, figures, and tables.

```
CounterDecl {
    name: String,           // counter name (e.g., "section", "equation")
    format: CounterFormat,  // display format
    reset_scope: CounterReset,  // when to reset
}
```

### 6.2 Formatting

| Format | Example | Use Case |
|--------|---------|----------|
| `Arabic` | 1, 2, 3 | Sections, equations |
| `RomanLower` | i, ii, iii | Preface pages |
| `RomanUpper` | I, II, III | Book parts |
| `AlphaLower` | a, b, c | Sub-items |
| `AlphaUpper` | A, B, C | Appendices |
| `Custom(String)` | (1), §1 | Custom formatting |

### 6.3 Scoping Rules

Counters reset based on their `reset_scope`:

- `PerDocument`: resets once at document start
- `PerPart`: resets at each `Part` node
- `PerChapter`: resets at each `Chapter` node
- `PerSection`: resets at each `Section` node
- `PerSubsection`: resets at each `Subsection` node
- `PerPage`: resets on each page (backend-determined)
- `Never`: never resets

### 6.4 Counter Increment

Nodes increment counters via the `counter` field:
```
@section [id=1, counter="section"] { }
```

---

## 7. Cross-References

### 7.1 Labels

Labels identify document elements for cross-referencing.

```
LabelInfo {
    node_id: u32,             // the labeled node
    category: LabelCategory,  // type of entity
}
```

Categories: `Section`, `Equation`, `Figure`, `Table`, `Footnote`, `Page`, `Custom`.

Labels are registered in `annotations.labels` as a `HashMap<String, LabelInfo>`.

### 7.2 Cross-References

References point to labels:

```
CrossRef {
    label: String,        // target label
    ref_node_id: u32,     // node containing the reference
}
```

### 7.3 Resolution Semantics

1. Frontends emit labels and refs into the annotations
2. Reference resolution is a separate pass (not part of S-IR)
3. Backends resolve refs to display text (e.g., "Section 3.2", "Equation (1)")
4. Unresolved refs are an error condition for backends

---

## 8. Text Format (.ldir)

### 8.1 Overview

The `.ldir` text format is a human-readable representation designed for:
- Manual authoring and editing
- Version control (diff-friendly)
- Debugging and inspection
- As the input format for `ldir-as`

### 8.2 Grammar (Informal)

```
module        ::= header? meta resources? styles? body
header        ::= ";;" comment NL
meta          ::= "@meta" "{" meta_fields "}"
meta_fields   ::= (meta_field NL)*
meta_field    ::= key "=" value
resources     ::= (font_decl | counter_decl)*
font_decl     ::= "@font" STRING "{" font_fields "}"
counter_decl  ::= "@counter" STRING "{" counter_fields "}"
styles        ::= style_decl*
style_decl    ::= "@style" STRING "{" style_fields "}"
body          ::= "@body" "{" node* "}"
node          ::= "@" tag "[" attrs "]" "{" body_fields "}"
tag           ::= IDENT
attrs         ::= attr ("," attr)*
attr          ::= key "=" (NUMBER | STRING)
```

### 8.3 Comments

Lines starting with `;;` are comments:
```
;; ldir-ir v2.0.0
;; source: markdown
```

### 8.4 Metadata Block

```
@meta {
  title = "My Document"
  author = "Jane Doe"
  language = "en"
  class = "article"
}
```

### 8.5 Font Declarations

```
@font "body" { family = "Inter", weight = "regular" }
@font "heading" { family = "Inter", weight = "bold" }
```

### 8.6 Counter Declarations

```
@counter "section" { format = "arabic" }
@counter "equation" { format = "arabic" }
```

### 8.7 Style Declarations

```
@style "body-text" {
  parent = "base"
}
@style "heading-1" {
  parent = "body-text"
}
```

### 8.8 Node Declarations

Nodes are declared with tag, attributes, and body:

```
@section [id=1, label="sec:intro"] { }
@paragraph [id=2, parent=1] { }
@text [id=3, parent=2] { "Introduction text here" }
@equation [id=4, parent=1, label="eq:euler"] { numbered=true }
@link [id=5, parent=2] { url="https://example.com" }
@image [id=6, parent=2] { src="figures/diagram.png" }
@code-block [id=7, parent=1] { lang="rust" }
@toc [id=8] { depth=3 }
```

### 8.9 Text Encoding

- File encoding: UTF-8
- Line endings: LF (Unix)
- String escaping: Rust-style (`\"`, `\\`, `\n`)
- Comments: `;;` prefix, line-only (no block comments)

---

## 9. Binary Format

### 9.1 Overview

The binary format (`.sir2`) is a compact, efficient representation for:
- Fast loading and saving
- Inter-process communication
- Embedded storage in containers

### 9.2 Encoding

All multi-byte integers are little-endian. The format is:

```
Offset  Size  Field
------  ----  -----
0       4     Magic: "LDIR" (ASCII)
4       1     Version major (u8)
5       1     Version minor (u8)
6       1     Version patch (u8)
7       2     IR version (u16 LE)
9       8     Created timestamp (u64 LE)
17      2     Source format length (u16 LE)
19      var   Source format (UTF-8)
...     2     Source path length (u16 LE)
...     var   Source path (UTF-8)
--- metadata section ---
...     4     Metadata length (u32 LE)
...     var   Metadata (JSON)
--- resources section ---
...     4     Resources length (u32 LE)
...     var   Resources (JSON)
--- styles section ---
...     4     Styles length (u32 LE)
...     var   Styles (JSON)
--- annotations section ---
...     4     Annotations length (u32 LE)
...     var   Annotations (JSON)
--- body section ---
...     4     Body length (u32 LE)
...     var   Body (JSON)
```

### 9.3 Section Serialization

Each section is serialized as length-prefixed JSON using `serde_json`.
This provides:
- Human inspectability (can extract sections with `jq`)
- Schema evolution (JSON is self-describing)
- Compact enough for most documents

### 9.4 Minimum Valid File

17 bytes (magic + version + ir_version + created + zero-length strings + 5 empty sections):
```
4c 44 49 52 02 00 00 02 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
```

### 9.5 Validation

A valid `.sir2` file must:
1. Start with `b"LDIR"` magic bytes
2. Have major version `2`
3. Have exactly 5 sections after the header
4. Each section must be valid JSON for its type

---

## 10. Versioning

### 10.1 Semantic Versioning

S-IR uses semantic versioning `(major, minor, patch)`:
- **Major**: Breaking changes (incompatible binary format)
- **Minor**: Additive changes (new node types, new fields with defaults)
- **Patch**: Bug fixes, documentation

### 10.2 Compatibility

- Readers must support all versions with the same major version
- Writers must produce the latest minor version within a major version
- Unknown fields in JSON sections are ignored during deserialization
- Unknown node types produce a warning but are preserved as opaque nodes

### 10.3 Migration

Migration between major versions requires an explicit conversion tool.
No automatic migration is performed by readers.

### 10.4 Version History

| Version | Date | Changes |
|---------|------|---------|
| 2.0.0 | 2025-04 | Initial S-IR v2 release |

---

## 11. Tooling

### 11.1 ldir-dis — Disassembler

Converts binary `.sir2` to human-readable `.ldir` text (or JSON).

```
ldir-dis [OPTIONS] <input>

Options:
  -o, --output <FILE>    Output file (default: stdout)
  -f, --format <FORMAT>  Output format: "text" or "json" (default: "text")
```

Examples:
```bash
ldir-dis document.sir2                    # text to stdout
ldir-dis document.sir2 -o document.ldir   # text to file
ldir-dis document.sir2 -f json            # JSON to stdout
ldir-dis document.ldir                    # text passthrough
```

### 11.2 ldir-as — Assembler

Converts `.ldir` text to binary `.sir2`.

```
ldir-as [OPTIONS] <input>

Options:
  -o, --output <FILE>    Output file (default: stdout binary)
```

Examples:
```bash
ldir-as document.ldir                     # binary to stdout
ldir-as document.ldir -o document.sir2    # binary to file
```

### 11.3 ldir-validate — Validator (Planned)

Validates a `.ldir` or `.sir2` file for well-formedness:
- Unique node IDs
- Valid parent references
- No cycles in the node tree
- Valid label references
- Required metadata fields

### 11.4 Roundtrip Workflow

```bash
# Text → Binary → Text
ldir-as input.ldir -o intermediate.sir2
ldir-dis intermediate.sir2 -o output.ldir

# Binary → Text → Binary
ldir-dis input.sir2 -o intermediate.ldir
ldir-as intermediate.ldir -o output.sir2
```

The binary roundtrip preserves all information. The text roundtrip may normalize
formatting but preserves all semantic content.

---

## 12. Examples

### 12.1 Minimal Document

```ldir
;; ldir-ir v2.0.0

@meta {
  title = "Hello World"
  language = "en"
}

@body {
  @section [id=1] { }
  @paragraph [id=2, parent=1] { }
  @text [id=3, parent=2] { "Hello, world!" }
}
```

### 12.2 Article with Sections

```ldir
;; ldir-ir v2.0.0
;; source: markdown

@meta {
  title = "S-IR v2 Guide"
  author = "LDIR Team"
  language = "en"
  class = "article"
}

@font "body" { family = "Inter", weight = "regular" }
@font "heading" { family = "Inter", weight = "bold" }

@counter "section" { format = "arabic" }
@counter "equation" { format = "arabic" }

@style "body-text" { }
@style "heading-1" { parent = "body-text" }

@body {
  @section [id=1, label="sec:intro", style="heading-1", counter="section"] { }
  @paragraph [id=2, parent=1, style="body-text"] { }
  @text [id=3, parent=2] { "This is the introduction." }

  @section [id=4, label="sec:methods", counter="section"] { }
  @paragraph [id=5, parent=4] { }
  @text [id=6, parent=5] { "Our methods are as follows." }

  @equation [id=7, parent=4, label="eq:euler", counter="equation"] { numbered=true }

  @section [id=8, label="sec:results", counter="section"] { }
  @paragraph [id=9, parent=8] { }
  @text [id=10, parent=9] { "The results confirm our hypothesis." }

  @figure [id=11, parent=8] { }
  @caption [id=12, parent=11] { }
  @text [id=13, parent=12] { "Figure 1: Results summary" }

  @table [id=14, parent=8] { }
  @table-row [id=15, parent=14, is_header=true] { }
  @table-cell [id=16, parent=15] { }
  @text [id=17, parent=16] { "Metric" }
  @table-cell [id=18, parent=15] { }
  @text [id=19, parent=18] { "Value" }
}
```

### 12.3 Book with Parts

```ldir
;; ldir-ir v2.0.0

@meta {
  title = "The LDIR Book"
  author = "LDIR Team"
  class = "book"
}

@counter "part" { format = "roman-upper" }
@counter "chapter" { format = "arabic" }

@body {
  @part [id=1, label="part:foundations", counter="part"] { }
  @chapter [id=2, parent=1, label="ch:intro", counter="chapter"] { }
  @section [id=3, parent=2] { }
  @paragraph [id=4, parent=3] { }
  @text [id=5, parent=4] { "Foundational concepts." }

  @part [id=6, label="part:advanced", counter="part"] { }
  @chapter [id=7, parent=6, label="ch:advanced", counter="chapter"] { }
  @section [id=8, parent=7] { }
  @paragraph [id=9, parent=8] { }
  @text [id=10, parent=9] { "Advanced topics." }
}
```

### 12.4 Technical Document with Code and Math

```ldir
;; ldir-ir v2.0.0

@meta {
  title = "Algorithm Analysis"
  language = "en"
}

@body {
  @section [id=1, label="sec:algorithm"] { }
  @paragraph [id=2, parent=1] { }
  @text [id=3, parent=2] { "The time complexity is " }
  @math [id=4, parent=2] { "O(n \\log n)" }
  @text [id=5, parent=2] { " as shown below." }

  @code-block [id=6, parent=1, lang="rust"] { }

  @equation [id=7, parent=1, label="eq:complexity", counter="equation"] { numbered=true }

  @paragraph [id=8, parent=1] { }
  @text [id=9, parent=8] { "See " }
  @link [id=10, parent=8] { url="https://example.com/paper" }
  @text [id=11, parent=8] { " for the full proof." }

  @footnote [id=12] { "Additional details in the appendix." }
}
```

---

## Appendix A: Node Type Quick Reference

| Tag | NodeType | Category | Key Fields |
|-----|----------|----------|------------|
| `@document` | Document | Structure | — |
| `@part` | Part | Structure | — |
| `@chapter` | Chapter | Structure | — |
| `@section` | Section | Structure | — |
| `@subsection` | Subsection | Structure | — |
| `@subsubsection` | Subsubsection | Structure | — |
| `@paragraph` | Paragraph | Structure | — |
| `@list` | List | List | `list_type`, `ordered`, `start` |
| `@list-item` | ListItem | List | — |
| `@blockquote` | BlockQuote | Block | — |
| `@code-block` | CodeBlock | Block | `language` |
| `@equation` | MathBlock | Block | `math_type`, `numbered` |
| `@table` | Table | Block | `col_specs`, `num_cols` |
| `@table-row` | TableRow | Block | `is_header` |
| `@table-cell` | TableCell | Block | `colspan`, `rowspan` |
| `@text` | Text | Inline | `content` |
| `@styled` | Styled | Inline | `style_name` |
| `@bold` | Bold | Inline | — |
| `@italic` | Italic | Inline | — |
| `@mono` | Mono | Inline | — |
| `@underline` | Underline | Inline | — |
| `@strike` | Strikethrough | Inline | — |
| `@smallcaps` | SmallCaps | Inline | — |
| `@link` | Link | Inline | `url`, `title` |
| `@image` | Image | Inline | `source`, `alt`, `width`, `height` |
| `@math` | MathInline | Inline | `content` |
| `@br` | LineBreak | Inline | — |
| `@figure` | Figure | Float | `placement` |
| `@caption` | Caption | Float | — |
| `@footnote` | Footnote | Special | `content` |
| `@footnote-block` | FootnoteBlock | Special | — |
| `@toc` | TableOfContents | Special | `max_depth` |
| `@page-break` | PageBreak | Special | — |
| `@hr` | ThematicBreak | Special | — |
| `@group` | Group | Container | — |
