/// Initialize tracing with chrome trace output (for flamegraph viewing).
pub fn init_tracing() {
    ldir_otelkit::init_chrome();
}
