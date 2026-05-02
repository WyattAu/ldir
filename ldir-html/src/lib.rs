//! LDIR HTML Backend — converts S-IR v2 to semantic HTML5.
//!
//! # Usage
//!
//! ```text
//! use ldir_html::HtmlRenderer;
//! let html = HtmlRenderer::new().render(&module);
//! ```

mod render;

pub use render::HtmlOptions;
pub use render::HtmlRenderer;
pub use render::MathFormat;
