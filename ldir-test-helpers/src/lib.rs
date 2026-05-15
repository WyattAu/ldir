//! Shared test utilities for LDIR crates.
//!
//! Provides access to the bundled DejaVu Sans test font fixture
//! with automatic fallback to system-installed fonts.

use std::path::PathBuf;

/// Known system paths for DejaVu Sans on various platforms.
const SYSTEM_FONT_PATHS: &[&str] = &[
    // Debian/Ubuntu
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    // Fedora/RHEL
    "/usr/share/fonts/dejavu/DejaVuSans.ttf",
    // macOS (Homebrew)
    "/opt/homebrew/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/local/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    // Windows (MSYS2/Chocolatey)
    "C:\\Windows\\Fonts\\DejaVuSans.ttf",
];

/// Returns the path to the DejaVu Sans test font.
///
/// Search order:
/// 1. `LDIR_TEST_FONT` environment variable (if set)
/// 2. Bundled fixture at `tests/fixtures/DejaVuSans.ttf` (relative to workspace root)
/// 3. System-installed DejaVu Sans (cross-platform paths)
///
/// Panics if no font is found.
pub fn test_font_path() -> PathBuf {
    // 1. Environment variable override
    if let Ok(path) = std::env::var("LDIR_TEST_FONT") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return p;
        }
        panic!("LDIR_TEST_FONT set to non-existent path: {}", path);
    }

    // 2. Bundled fixture
    if let Some(root) = workspace_root() {
        let fixture = root.join("tests/fixtures/DejaVuSans.ttf");
        if fixture.exists() {
            return fixture;
        }
    }

    // 3. System paths
    for path in SYSTEM_FONT_PATHS {
        let p = PathBuf::from(path);
        if p.exists() {
            return p;
        }
    }

    panic!(
        "DejaVu Sans test font not found. Set LDIR_TEST_FONT, install fonts-dejavu-core, \
         or ensure tests/fixtures/DejaVuSans.ttf exists in the workspace root."
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
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists()
            && let Ok(content) = std::fs::read_to_string(&cargo_toml)
            && content.contains("[workspace]")
        {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Returns a list of all known font paths for DejaVu Sans.
/// Useful for font database tests that load multiple system fonts.
pub fn system_font_search_paths() -> &'static [&'static str] {
    SYSTEM_FONT_PATHS
}
