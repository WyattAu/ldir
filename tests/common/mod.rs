//! Shared test utilities for LDIR integration tests.
//!
//! Provides access to the bundled test font fixture at `tests/fixtures/DejaVuSans.ttf`.
//! Falls back to system-installed DejaVu Sans if the fixture is not found.

use std::path::PathBuf;

/// Known system paths for DejaVu Sans on various Linux distributions.
const SYSTEM_FONT_PATHS: &[&str] = &[
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
];

/// Returns the path to the DejaVu Sans test font.
///
/// Search order:
/// 1. Bundled fixture at `tests/fixtures/DejaVuSans.ttf` (relative to workspace root)
/// 2. System-installed DejaVu Sans (Debian/Ubuntu/Fedora paths)
///
/// Panics if no font is found.
pub fn test_font_path() -> PathBuf {
    // Try bundled fixture first
    if let Some(root) = workspace_root() {
        let fixture = root.join("tests/fixtures/DejaVuSans.ttf");
        if fixture.exists() {
            return fixture;
        }
    }

    // Fall back to system paths
    for path in SYSTEM_FONT_PATHS {
        let p = PathBuf::from(path);
        if p.exists() {
            return p;
        }
    }

    panic!(
        "DejaVu Sans test font not found. Install fonts-dejavu-core or ensure tests/fixtures/DejaVuSans.ttf exists."
    );
}

/// Returns the raw bytes of the DejaVu Sans test font.
pub fn test_font_data() -> Vec<u8> {
    let path = test_font_path();
    std::fs::read(&path).unwrap_or_else(|e| {
        panic!("Failed to read test font at {}: {}", path.display(), e)
    })
}

/// Attempts to find the workspace root by walking up from CARGO_MANIFEST_DIR.
fn workspace_root() -> Option<PathBuf> {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Walk up looking for Cargo.toml with [workspace]
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                if content.contains("[workspace]") {
                    return Some(dir);
                }
            }
        }
        if !dir.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_found() {
        let path = test_font_path();
        assert!(path.exists(), "Test font not found at {}", path.display());
    }

    #[test]
    fn test_font_data_loads() {
        let data = test_font_data();
        assert!(data.len() > 1000, "Test font too small ({} bytes)", data.len());
    }
}
