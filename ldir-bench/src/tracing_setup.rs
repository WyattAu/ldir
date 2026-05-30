use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Initialize tracing with chrome trace output (for flamegraph viewing).
pub fn init_tracing() {
    let (chrome_layer, _guard) = tracing_chrome::ChromeLayerBuilder::new().build();

    tracing_subscriber::registry()
        .with(chrome_layer)
        .with(tracing_subscriber::EnvFilter::new("info"))
        .init();
}
