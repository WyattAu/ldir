//! # ldir-pdf
//!
//! PDF generation backend for the LDIR document pipeline. Converts G-IR
//! rendering commands into valid PDF files with embedded TrueType fonts,
//! ToUnicode CMaps for text extraction, and FlateDecode compression.

#![warn(clippy::unwrap_used, clippy::expect_used)]
#![deny(missing_docs)]
#![deny(unsafe_code)]
//!
//! ## Key Types
//!
//! - [`converter::gir_to_pdf`] — Convert G-IR to PDF bytes (fallback Helvetica)
//! - [`converter::gir_to_pdf_with_fonts`] — Convert with embedded TrueType fonts
//! - [`conformance::PdfConformance`] — PDF/A conformance level (4, 2b, 3b)
//! - [`converter::PdfOptions`] — PDF metadata and header/footer configuration
//! - [`font::FontFace`] — TrueType font handle for embedding
//! - [`lir_render::render_lir_to_gir`] — Convert L-IR layout tree to G-IR
//! - [`image::ImageData`] — Decoded image data for embedding
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use ldir_pdf::converter::gir_to_pdf;
//!
//! // Convert a G-IR document to PDF (fallback Helvetica)
//! let pdf_bytes = gir_to_pdf(&gir_doc);
//! std::fs::write("output.pdf", &pdf_bytes).unwrap();
//! ```
//!
//! ## Features
//!
//! - TrueType font embedding (Type0 + CIDFontType2)
//! - Multiple font variants: Regular, Bold, Italic, BoldItalic, Mono
//! - ToUnicode CMap generation for text extraction
//! - FlateDecode stream compression
//! - PNG and JPEG image embedding
//! - PDF/A-4 logical structure
//! - L-IR to G-IR rendering pipeline
//!
//! ## References
//!
//! - [Repository](https://github.com/WyattAu/ldir)

/// G-IR to PDF converter with TrueType font embedding.
pub mod conformance;

/// G-IR to PDF converter with TrueType font embedding.
pub mod converter;

/// Font loading, metrics, subsetting, and TrueType embedding.
pub mod font;

/// Image decoding (PNG, JPEG) for PDF embedding.
pub mod image;

/// L-IR to G-IR rendering pipeline.
pub mod lir_render;

/// ICC color profiles and color space conversion for print output.
pub mod color;

/// PDF/A logical structure types.
pub mod structure;

/// XMP metadata generation for PDF/A.
pub mod xmp;

#[cfg(test)]
mod pdf_test;

pub(crate) mod writer;

pub(crate) mod streaming_writer;
