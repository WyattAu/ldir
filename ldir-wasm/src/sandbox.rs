//! Sandbox configuration for WASM execution.
//!
//! Provides resource limits (fuel, memory) to constrain WASM module
//! execution per REQ-7.3.

/// Default fuel limit: 100,000 instructions (REQ-7.3).
pub const DEFAULT_FUEL_LIMIT: u64 = 100_000;

/// Default memory limit: 256 MB.
pub const DEFAULT_MEMORY_LIMIT: u64 = 256 * 1024 * 1024;

/// Sandbox configuration for WASM execution.
///
/// Controls resource limits for WASM module execution:
/// - **Fuel**: Maximum number of instructions the WASM module may execute.
/// - **Memory**: Maximum linear memory allocation in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SandboxConfig {
    /// Maximum number of WASM instructions (fuel).
    /// Per REQ-7.3, default is 100,000.
    pub fuel_limit: u64,
    /// Maximum WASM linear memory in bytes.
    /// Default is 256 MB.
    pub memory_limit: u64,
}

impl SandboxConfig {
    /// Create a new sandbox config with the given limits.
    pub fn new(fuel_limit: u64, memory_limit: u64) -> Self {
        Self {
            fuel_limit,
            memory_limit,
        }
    }

    /// Create a config with default limits.
    ///
    /// - Fuel: 100,000 instructions
    /// - Memory: 256 MB
    pub fn default_limits() -> Self {
        Self {
            fuel_limit: DEFAULT_FUEL_LIMIT,
            memory_limit: DEFAULT_MEMORY_LIMIT,
        }
    }

    /// Check if a fuel consumption amount exceeds the limit.
    pub fn exceeds_fuel(&self, consumed: u64) -> bool {
        consumed > self.fuel_limit
    }

    /// Check if a memory allocation exceeds the limit.
    pub fn exceeds_memory(&self, allocated: u64) -> bool {
        allocated > self.memory_limit
    }

    /// Get remaining fuel after some consumption.
    pub fn remaining_fuel(&self, consumed: u64) -> u64 {
        self.fuel_limit.saturating_sub(consumed)
    }

    /// Get remaining memory after some allocation.
    pub fn remaining_memory(&self, allocated: u64) -> u64 {
        self.memory_limit.saturating_sub(allocated)
    }

    /// Clamp a fuel consumption to the limit, returning the actual
    /// allowed amount and whether the limit was exceeded.
    pub fn clamp_fuel(&self, requested: u64) -> (u64, bool) {
        if requested <= self.fuel_limit {
            (requested, false)
        } else {
            (self.fuel_limit, true)
        }
    }
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self::default_limits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SandboxConfig::default();
        assert_eq!(config.fuel_limit, DEFAULT_FUEL_LIMIT);
        assert_eq!(config.fuel_limit, 100_000);
        assert_eq!(config.memory_limit, DEFAULT_MEMORY_LIMIT);
        assert_eq!(config.memory_limit, 256 * 1024 * 1024);
    }

    #[test]
    fn test_default_limits() {
        let config = SandboxConfig::default_limits();
        assert_eq!(config.fuel_limit, 100_000);
        assert_eq!(config.memory_limit, 256 * 1024 * 1024);
    }

    #[test]
    fn test_new() {
        let config = SandboxConfig::new(50_000, 128 * 1024 * 1024);
        assert_eq!(config.fuel_limit, 50_000);
        assert_eq!(config.memory_limit, 128 * 1024 * 1024);
    }

    #[test]
    fn test_exceeds_fuel() {
        let config = SandboxConfig::default();
        assert!(!config.exceeds_fuel(50_000));
        assert!(!config.exceeds_fuel(100_000));
        assert!(config.exceeds_fuel(100_001));
    }

    #[test]
    fn test_exceeds_memory() {
        let config = SandboxConfig::default();
        assert!(!config.exceeds_memory(100 * 1024 * 1024));
        assert!(!config.exceeds_memory(256 * 1024 * 1024));
        assert!(config.exceeds_memory(256 * 1024 * 1024 + 1));
    }

    #[test]
    fn test_remaining_fuel() {
        let config = SandboxConfig::default();
        assert_eq!(config.remaining_fuel(0), 100_000);
        assert_eq!(config.remaining_fuel(50_000), 50_000);
        assert_eq!(config.remaining_fuel(100_000), 0);
        assert_eq!(config.remaining_fuel(200_000), 0);
    }

    #[test]
    fn test_remaining_memory() {
        let config = SandboxConfig::default();
        let half = 128 * 1024 * 1024;
        assert_eq!(config.remaining_memory(half), half);
        assert_eq!(config.remaining_memory(DEFAULT_MEMORY_LIMIT), 0);
        assert_eq!(config.remaining_memory(DEFAULT_MEMORY_LIMIT + 1), 0);
    }

    #[test]
    fn test_clamp_fuel_within_limit() {
        let config = SandboxConfig::default();
        let (allowed, exceeded) = config.clamp_fuel(50_000);
        assert_eq!(allowed, 50_000);
        assert!(!exceeded);
    }

    #[test]
    fn test_clamp_fuel_exceeds_limit() {
        let config = SandboxConfig::default();
        let (allowed, exceeded) = config.clamp_fuel(200_000);
        assert_eq!(allowed, 100_000);
        assert!(exceeded);
    }

    #[test]
    fn test_clamp_fuel_exact() {
        let config = SandboxConfig::default();
        let (allowed, exceeded) = config.clamp_fuel(100_000);
        assert_eq!(allowed, 100_000);
        assert!(!exceeded);
    }

    #[test]
    fn test_fuel_limit_enforcement() {
        let config = SandboxConfig::new(1000, 1024);

        let mut fuel_used: u64 = 0;
        let mut fuel_exhausted = false;

        for _ in 0..2000 {
            if config.exceeds_fuel(fuel_used) {
                fuel_exhausted = true;
                break;
            }
            fuel_used += 1;
        }

        assert!(fuel_exhausted);
        assert_eq!(fuel_used, 1001);
    }

    #[test]
    fn test_copy_and_equality() {
        let a = SandboxConfig::default();
        let b = a;
        assert_eq!(a, b);
    }
}
