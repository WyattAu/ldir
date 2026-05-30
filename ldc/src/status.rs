//! CLI status reporting with timing and styled output.

use std::io::Write;
use std::time::Instant;

/// Color support flag (set by --color).
pub(crate) static COLOR_ENABLED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(true);

pub fn set_color(enabled: bool) {
    COLOR_ENABLED.store(enabled, std::sync::atomic::Ordering::Relaxed);
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Color {
    Green,
    Yellow,
    Red,
    Cyan,
    Bold,
    Dim,
}

impl Color {
    fn ansi_code(self) -> &'static str {
        match self {
            Color::Green => "\x1b[32m",
            Color::Yellow => "\x1b[33m",
            Color::Red => "\x1b[31m",
            Color::Cyan => "\x1b[36m",
            Color::Bold => "\x1b[1m",
            Color::Dim => "\x1b[2m",
        }
    }
}

const RESET: &str = "\x1b[0m";

/// Format a styled string.
pub fn styled(text: &str, color: Color) -> String {
    if !COLOR_ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
        return text.to_string();
    }
    format!("{}{}{}", color.ansi_code(), text, RESET)
}

/// Simple pipeline progress tracker.
pub struct PipelineTimer {
    start: Instant,
    step_count: usize,
    current_step: usize,
}

impl PipelineTimer {
    pub fn new(total_steps: usize) -> Self {
        Self {
            start: Instant::now(),
            step_count: total_steps,
            current_step: 0,
        }
    }

    /// Print a step status with timing.
    pub fn step(&mut self, msg: &str) {
        self.current_step += 1;
        let elapsed = self.start.elapsed();
        let step = styled(
            &format!("[{}/{}]", self.current_step, self.step_count),
            Color::Dim,
        );
        let icon = styled(">", Color::Green);
        eprintln!("{} {} {} {:.1}s", step, icon, msg, elapsed.as_secs_f64());
    }

    /// Print a warning message.
    pub fn warn(&self, msg: &str) {
        let prefix = styled("[ldir]", Color::Yellow);
        let label = styled(" warning:", Color::Yellow);
        eprintln!("{} {} {}", prefix, label, msg);
    }

    /// Print an info message (the standard `[ldir]` prefix).
    #[allow(dead_code)]
    pub fn info(&self, msg: &str) {
        let prefix = styled("[ldir]", Color::Cyan);
        eprintln!("{} {}", prefix, msg);
    }

    /// Print an error message.
    #[allow(dead_code)]
    pub fn error(&self, msg: &str) {
        let prefix = styled("[ldir]", Color::Red);
        let label = styled("error:", Color::Red);
        eprintln!("{} {} {}", prefix, label, msg);
    }

    /// Print the final summary.
    pub fn finish(&self, msg: &str) {
        let elapsed = self.start.elapsed();
        let prefix = styled("[ldir]", Color::Cyan);
        let time = styled(&format!("({:.1}s)", elapsed.as_secs_f64()), Color::Dim);
        eprintln!("{} {} {}", prefix, msg, time);
    }

    /// Flush stderr to ensure progress output is visible.
    pub fn flush(&self) {
        let _ = std::io::stderr().flush();
    }
}
