//! Tracing and observability integration (REQ-8.1, REQ-11.4).
//!
//! Re-exports the `tracing` crate with LDIR-specific span definitions
//! for every major compilation phase. Uses compile-time gated `tracing`
//! macros to keep overhead < 1% on hot paths.
//!
//! ## References
//!
//! - REQ-8.1: `tracing` ecosystem integration
//! - REQ-11.4.1: Nanosecond-resolution tracing of major layout functions

#[allow(unused_imports)]
pub use tracing::{
    Level, debug, debug_span, error, error_span, info, info_span, instrument, span, trace,
    trace_span, warn, warn_span,
};

use std::time::Duration;

/// Instrumentation macro that wraps a closure with a tracing span and measures timing.
///
/// Creates a span with the given name at `Level::INFO`, executes the closure inside it,
/// and returns the closure's result along with the elapsed duration.
///
/// # Example
///
/// ```ignore
/// let (result, elapsed) = trace_phase!("compile_sir", {
///     compiler.compile(&sir)
/// });
/// ```
#[macro_export]
macro_rules! trace_phase {
    ($name:expr, $body:expr) => {{
        let _span = $crate::trace::info_span!($name).entered();
        let start = std::time::Instant::now();
        let result = $body;
        let elapsed = start.elapsed();
        (result, elapsed)
    }};
}

/// Returns a tracing span for a standard LDIR compilation phase.
///
/// | Phase         | Level      |
/// |---------------|------------|
/// | `parse_sir`   | `INFO`     |
/// | `validate_sir`| `INFO`     |
/// | `compile_sir` | `INFO`     |
/// | `emit_gir`    | `INFO`     |
pub fn phase_span(name: &str) -> tracing::Span {
    match name {
        "parse_sir" => info_span!("parse_sir"),
        "validate_sir" => info_span!("validate_sir"),
        "compile_sir" => info_span!("compile_sir"),
        "emit_gir" => info_span!("emit_gir"),
        _ => {
            let s = info_span!("ldir_phase", name);
            s
        }
    }
}

/// Records the dynamic phase name into a span's field.
pub fn record_phase_name(span: &tracing::Span, name: &str) {
    span.record("name", name);
}

/// All LDIR compilation phase names.
pub static PHASE_NAMES: &[&str] = &["parse_sir", "validate_sir", "compile_sir", "emit_gir"];

/// Result of a traced phase execution.
#[derive(Debug, Clone)]
pub struct PhaseResult<T> {
    /// The phase name.
    pub phase: String,
    /// The result of the closure.
    pub value: T,
    /// Wall-clock elapsed time.
    pub elapsed: Duration,
}

/// Wraps a closure in a named phase span, returning a `PhaseResult`.
pub fn trace_phase_fn<F, T>(name: &str, f: F) -> PhaseResult<T>
where
    F: FnOnce() -> T,
{
    let span = phase_span(name);
    let _guard = span.enter();
    let start = std::time::Instant::now();
    let value = f();
    let elapsed = start.elapsed();
    PhaseResult {
        phase: name.to_owned(),
        value,
        elapsed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_phase_macro_returns_elapsed() {
        let (result, elapsed) = trace_phase!("test_phase", {
            std::thread::sleep(std::time::Duration::from_micros(100));
            42
        });
        assert_eq!(result, 42);
        assert!(elapsed.as_micros() >= 100);
    }

    #[test]
    fn trace_phase_fn_returns_correct_phase() {
        let pr = trace_phase_fn("compile_sir", || 99);
        assert_eq!(pr.phase, "compile_sir");
        assert_eq!(pr.value, 99);
        assert!(pr.elapsed.as_nanos() > 0);
    }

    #[test]
    fn phase_span_returns_valid_span_for_all_phases() {
        for &name in PHASE_NAMES {
            let _span = phase_span(name);
        }
    }

    #[test]
    fn phase_span_handles_unknown_name() {
        let _span = phase_span("some_custom_phase");
    }
}
