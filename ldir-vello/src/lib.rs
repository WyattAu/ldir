//! LDIR Vello/GPU Rendering Integration.

#![warn(clippy::unwrap_used, clippy::expect_used)]
//!
//! Provides GPU-accelerated rendering of G-IR documents using the Vello
//! 2D graphics engine and wgpu compute shaders.
//!
//! ## Modules
//!
//! - [`renderer`] — `VelloRenderer` wrapping Vello scene construction
//! - [`gir_to_scene`] — G-IR to Vello Scene conversion
//! - [`viewport`] — Viewport with pan/zoom and coordinate transforms

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod gir_to_scene;
pub mod renderer;
pub mod viewport;
