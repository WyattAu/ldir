//! Rust-native test plugins demonstrating the plugin ABI pattern
//! for future Wasm plugins (REQ-4.1.2 zero-copy interface).

/// Plugin manifest format: capabilities, resource limits, validation.
pub mod manifest;
/// Test plugin that injects running page headers.
pub mod test_header;
/// Test plugin that expands simple `{{macro}}` placeholders.
pub mod test_macro;
/// Test plugin that applies paragraph style overrides via selectors.
pub mod test_style;
