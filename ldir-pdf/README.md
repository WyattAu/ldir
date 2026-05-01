# ldir-pdf

G-IR to PDF converter for the LDIR document pipeline.

Produces valid PDF files with embedded TrueType fonts, ToUnicode CMaps for text extraction, and FlateDecode compression.

## Features

- TrueType font embedding (Type0 + CIDFontType2)
- Multiple font variants: Regular, Bold, Italic, BoldItalic, Mono
- ToUnicode CMap generation for text extraction
- FlateDecode stream compression
- Fallback to viewer-resident Helvetica when no fonts are provided

## Example

```rust
use ldir_pdf::converter::gir_to_pdf;
use ldir_pdf::font::FontFace;

// Convert a G-IR document to PDF (fallback Helvetica)
let pdf_bytes = gir_to_pdf(&gir_doc);

// Convert with embedded TrueType font
let font_data = std::fs::read("fonts/DejaVuSans.ttf").unwrap();
let font = FontFace::from_bytes(&font_data).unwrap();
let pdf_bytes = ldir_pdf::converter::gir_to_pdf_with_fonts(&gir_doc, &[font]);

std::fs::write("output.pdf", &pdf_bytes).unwrap();
```

## License

MIT OR Apache-2.0
