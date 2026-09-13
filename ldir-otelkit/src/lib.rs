//! Shared tracing subscriber initialization for LDIR binaries.
//!
//! Provides convenience functions for setting up `tracing-subscriber`
//! with common configurations used across the workspace.

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Initialize tracing with a simple `fmt` subscriber and env-filter.
///
/// Reads `RUST_LOG` for the filter level (defaults to `info`).
pub fn init() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
}

/// Initialize tracing with a chrome-trace layer (for flamegraph viewing).
///
/// Returns the chrome-layer guard; the caller must keep it alive for the
/// duration of tracing (it is leaked here for convenience in simple binaries).
pub fn init_chrome() {
    let (chrome_layer, _guard) = tracing_chrome::ChromeLayerBuilder::new().build();

    tracing_subscriber::registry()
        .with(chrome_layer)
        .with(tracing_subscriber::EnvFilter::new("info"))
        .init();

    // Leak the guard so the chrome layer stays alive.
    // In benchmarks the process exits shortly after, so this is fine.
    std::mem::forget(_guard);
}
