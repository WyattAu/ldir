# Architecture

## Compilation Pipeline

LDIR uses a multi-stage compilation pipeline that transforms source documents through several intermediate representations:

```
Input Document
     |
     v
  [Parser]  (format-specific)
     |
     v
  GIR (Generic IR)
     |
     v
  S-IR (Semantic IR)
     |
     v
  [Layout Engine]
     |
     v
  L-IR (Layout IR) / G-IR (Graphic IR)
     |
     v
  [Renderer]  (format-specific)
     |
     v
  Output Document
```

## Intermediate Representations

### GIR (Generic IR)

Format-agnostic document tree. Each input parser produces a GIR that normalizes document structure across formats. Defined in `ldir-ir/src/gir.rs`.

### S-IR (Semantic IR)

Typed, semantic intermediate representation with versioned serialization. S-IR v2 is the current format, supporting rich document semantics including cross-references, bibliography, and mathematical expressions. Defined in `ldir-ir/src/sir/`.

### L-IR (Layout IR)

Page-level layout representation with precise positioning, line breaking, and pagination. Supports the Knuth-Plass line-breaking algorithm. Defined in `ldir-core/src/layout/`.

### G-IR (Graphic IR)

Low-level graphic primitives for rendering: paths, text runs, images, and transformations. Defined in `ldir-core/src/gir/`.

## Text Shaping

LDIR uses HarfBuzz for text shaping with a fast-path optimization for ASCII text. The shaping pipeline:

1. **Fast path**: Pure-Rust `ttf_parser` for simple ASCII (no ligatures, no kerning)
2. **Full path**: HarfBuzz FFI for complex scripts (CJK, Arabic, Indic, etc.)
3. **Cache**: Shaped glyph runs are cached by font+text hash

## Font System

- Font discovery via `fontdb` (system font scanning)
- Font parsing via `ttf-parser`
- TrueType/OpenType glyph extraction
- Font subsetting (planned)

## Concurrency Model

- Document parsing: sequential (format-dependent state)
- Layout computation: sequential (per-page, potential for parallel)
- PDF generation: sequential (streaming planned)
- Font loading: cached, thread-safe via `Arc`

## Error Handling

- Typed error enums with `thiserror` for library crates
- `anyhow::Result` for CLI and application-level errors
- No panics in library code (3 guarded `unwrap`/`expect` in production)
- Integration tests validate error recovery paths
