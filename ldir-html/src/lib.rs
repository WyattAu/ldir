//! LDIR HTML Backend -- converts S-IR v2 to semantic HTML5.

#![warn(clippy::unwrap_used, clippy::expect_used)]
#![deny(missing_docs)]
#![deny(unsafe_code)]
//!
//! # Usage
//!
//! ```text
//! use ldir_html::HtmlRenderer;
//! let html = HtmlRenderer::new().render(&module);
//! ```

mod render;
/// HTML theme definitions.
pub mod themes;

pub use render::HtmlOptions;
pub use render::HtmlRenderOptions;
pub use render::HtmlRenderer;
pub use render::MathFormat;
pub use themes::HtmlTheme;
