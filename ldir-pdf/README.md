# ldir-pdf

PDF generation backend for the LDIR document pipeline. Converts G-IR
rendering commands into valid PDF files with embedded TrueType fonts,
ToUnicode CMaps, and FlateDecode compression.

## Features

- TrueType font embedding (Type0 + CIDFontType2) with subsetting
- Multiple font variants: Regular, Bold, Italic, BoldItalic, Mono
- ToUnicode CMap generation for text extraction
- FlateDecode stream compression
- PNG and JPEG image embedding
- PDF/A-4 logical structure (tagged PDF)
- Configurable headers, footers, and document metadata
- L-IR to G-IR rendering pipeline

## API Overview

| Function / Type | Description |
|-----------------|-------------|
| `converter::gir_to_pdf` | Convert G-IR to PDF (fallback Helvetica) |
| `converter::gir_to_pdf_with_fonts` | Convert G-IR to PDF with embedded fonts |
| `converter::PdfOptions` | PDF metadata and header/footer config |
| `font::FontFace` | TrueType font handle for embedding |
| `lir_render::render_lir_to_gir` | Convert L-IR layout tree to G-IR |
| `image::ImageData` | Decoded image data for embedding |

## Usage

```rust
use ldir_pdf::converter::gir_to_pdf;

// Convert a G-IR document to PDF (fallback Helvetica)
let pdf_bytes = gir_to_pdf(&gir_doc);
std::fs::write("output.pdf", &pdf_bytes).unwrap();
```

## Input / Output

- **Input**: `GIRDocument` (from `ldir-ir::gir`) or `LIRDocument` (from `ldir-ir::lir`)
- **Output**: PDF byte stream (`Vec<u8>`)

## License

MIT OR Apache-2.0

## Repository

[https://github.com/WyattAu/ldir](https://github.com/WyattAu/ldir)
