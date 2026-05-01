//! Frame profiling support (REQ-8.2, REQ-11.4.2).
//!
//! `FrameProfiler` records per-phase timing and exports to
//! [Chrome Trace Format](https://docs.google.com/document/d/1CvAClvFfyA5R-PhYUmn5OOQtYMH4h6I0nSsKchNAySU)
//! JSON loadable in `chrome://tracing`.
//!
//! ## References
//!
//! - REQ-8.2: Exportable traces in Chrome Trace Format
//! - REQ-11.4.2: Chrome Trace Format and Tracy export

use std::time::{Duration, Instant};

/// A recorded profiling span with start/end timestamps.
#[derive(Debug, Clone)]
pub struct ProfileSpan {
    /// Human-readable name of the span (e.g. `"compile_sir"`).
    pub name: String,
    /// Monotonic start time.
    pub start: Instant,
    /// Monotonic end time.
    pub end: Instant,
}

impl ProfileSpan {
    /// Elapsed duration of this span.
    pub fn duration(&self) -> Duration {
        self.end.duration_since(self.start)
    }
}

/// Opaque handle returned by [`FrameProfiler::begin`].
#[derive(Debug, Clone, Copy)]
pub struct ProfileHandle {
    index: usize,
}

/// Collects profiling spans and exports to Chrome Trace Format JSON.
#[derive(Debug, Clone)]
pub struct FrameProfiler {
    base: Instant,
    spans: Vec<Option<ProfileSpan>>,
}

impl FrameProfiler {
    /// Creates a new `FrameProfiler` with the current instant as the base timestamp.
    pub fn new() -> Self {
        FrameProfiler {
            base: Instant::now(),
            spans: Vec::new(),
        }
    }

    /// Begins a new profiling span with the given name.
    ///
    /// Returns a [`ProfileHandle`] that must be passed to [`FrameProfiler::end`]
    /// to close the span and record the elapsed time.
    pub fn begin(&mut self, name: &str) -> ProfileHandle {
        let index = self.spans.len();
        self.spans.push(Some(ProfileSpan {
            name: name.to_owned(),
            start: Instant::now(),
            end: Instant::now(),
        }));
        ProfileHandle { index }
    }

    /// Ends the span identified by `handle`, returning the elapsed duration.
    ///
    /// # Panics
    ///
    /// Panics if the handle is invalid or the span has already been ended.
    pub fn end(&mut self, handle: ProfileHandle) -> Duration {
        let span = self.spans[handle.index]
            .as_mut()
            .unwrap_or_else(|| panic!("profile handle {handle:?} already consumed or invalid"));
        span.end = Instant::now();
        span.duration()
    }

    /// Returns all completed spans.
    pub fn spans(&self) -> Vec<&ProfileSpan> {
        self.spans.iter().filter_map(|s| s.as_ref()).collect()
    }

    /// Exports recorded spans to Chrome Trace Format JSON.
    ///
    /// The output is a JSON array of trace events suitable for loading at `chrome://tracing`.
    /// Each event uses `X` (complete) ph format with microsecond-resolution timestamps.
    pub fn export_chrome_trace(&self) -> String {
        let events: Vec<String> = self
            .spans
            .iter()
            .filter_map(|s| s.as_ref())
            .map(|span| {
                let ts = span.start.duration_since(self.base).as_micros();
                let dur = span.duration().as_micros();
                format!(
                    r#"{{"name":"{}","ph":"X","ts":{},"dur":{},"pid":1,"tid":1}}"#,
                    span.name, ts, dur
                )
            })
            .collect();
        let mut json = String::from("{\"traceEvents\":[");
        json.push_str(&events.join(","));
        json.push_str("],\"displayTimeUnit\":\"us\"}");
        json
    }
}

impl Default for FrameProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_timing_start_before_end() {
        let mut profiler = FrameProfiler::new();
        let handle = profiler.begin("test");
        std::thread::sleep(Duration::from_micros(50));
        let elapsed = profiler.end(handle);
        let spans = profiler.spans();
        assert_eq!(spans.len(), 1);
        assert!(spans[0].start <= spans[0].end);
        assert!(elapsed.as_micros() >= 50);
    }

    #[test]
    fn profile_timing_accuracy() {
        let mut profiler = FrameProfiler::new();
        let handle = profiler.begin("accurate");
        let sleep = Duration::from_millis(1);
        std::thread::sleep(sleep);
        let elapsed = profiler.end(handle);
        assert!(elapsed >= sleep);
    }

    #[test]
    fn chrome_trace_is_valid_json() {
        let mut profiler = FrameProfiler::new();
        let h1 = profiler.begin("parse_sir");
        std::thread::sleep(Duration::from_micros(10));
        profiler.end(h1);
        let h2 = profiler.begin("compile_sir");
        std::thread::sleep(Duration::from_micros(10));
        profiler.end(h2);

        let json = profiler.export_chrome_trace();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["traceEvents"].is_array());
        assert_eq!(parsed["traceEvents"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["displayTimeUnit"], "us");
    }

    #[test]
    fn chrome_trace_empty_profiler() {
        let profiler = FrameProfiler::new();
        let json = profiler.export_chrome_trace();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["traceEvents"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn multiple_spans_ordered() {
        let mut profiler = FrameProfiler::new();
        let h1 = profiler.begin("first");
        profiler.end(h1);
        let h2 = profiler.begin("second");
        profiler.end(h2);
        let h3 = profiler.begin("third");
        profiler.end(h3);

        let spans = profiler.spans();
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].name, "first");
        assert_eq!(spans[1].name, "second");
        assert_eq!(spans[2].name, "third");
    }
}
