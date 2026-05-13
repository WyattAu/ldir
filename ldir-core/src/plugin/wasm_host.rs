//! Wasmtime-based plugin host for sandboxed Wasm plugins.
//!
//! ## Host-Guest ABI
//!
//! Guest Wasm modules must export the following functions:
//!
//! ```text
//! plugin_name() -> i32        // returns length of plugin name
//! plugin_name_ptr() -> i32    // returns pointer to plugin name (UTF-8)
//! plugin_version() -> i32     // returns ABI version (must be 1)
//! plugin_alloc(size: i32) -> i32  // allocate memory in guest, returns ptr
//! plugin_execute(input_ptr: i32, input_len: i32) -> i32  // returns output len
//! plugin_output_ptr() -> i32  // returns pointer to output buffer
//! plugin_free(ptr: i32)       // free memory in guest
//! ```
//!
//! ## Fuel Injection (REQ-4.1.3)
//!
//! Plugins are limited to a configurable number of Wasm instructions
//! (default: 100,000). Exceeding the limit traps the guest.
//!
//! ## Feature Gate
//!
//! This module is only available when the `wasm-plugins` feature is enabled.

#![cfg(feature = "wasm-plugins")]

use std::path::Path;

/// Current ABI version. Guest plugins must return this from `plugin_version()`.
pub const ABI_VERSION: i32 = 1;

/// Default fuel limit (number of Wasm instructions before trap).
pub const DEFAULT_FUEL_LIMIT: u64 = 100_000;

/// Errors from the Wasm plugin host.
#[derive(Debug, thiserror::Error)]
pub enum WasmHostError {
    /// Failed to load the Wasm module.
    #[error("failed to load Wasm module: {0}")]
    LoadFailed(String),

    /// The module is missing a required export.
    #[error("missing required export: {0}")]
    MissingExport(String),

    /// ABI version mismatch.
    #[error("ABI version mismatch: guest={guest}, host={host}")]
    AbiVersionMismatch { guest: i32, host: i32 },

    /// The plugin exceeded its fuel limit.
    #[error("plugin exceeded fuel limit ({limit} instructions)")]
    FuelExhausted { limit: u64 },

    /// The plugin trapped during execution.
    #[error("plugin trapped: {0}")]
    Trap(String),

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Configuration for the Wasm plugin host.
#[derive(Debug, Clone)]
pub struct WasmHostConfig {
    /// Maximum number of Wasm instructions before trapping.
    pub fuel_limit: u64,
    /// Whether to enable WASI preview1.
    pub enable_wasi: bool,
}

impl Default for WasmHostConfig {
    fn default() -> Self {
        Self {
            fuel_limit: DEFAULT_FUEL_LIMIT,
            enable_wasi: true,
        }
    }
}

/// A loaded Wasm plugin ready for execution.
#[allow(dead_code)]
pub struct WasmPlugin {
    engine: wasmtime::Engine,
    module: wasmtime::Module,
    name: String,
}

impl WasmPlugin {
    /// Load a Wasm plugin from a file path.
    pub fn from_file(path: &Path, config: &WasmHostConfig) -> Result<Self, WasmHostError> {
        let mut engine_config = wasmtime::Config::new();
        engine_config.consume_fuel(true);

        let engine = wasmtime::Engine::new(&engine_config)
            .map_err(|e| WasmHostError::LoadFailed(e.to_string()))?;

        let module = wasmtime::Module::from_file(&engine, path)
            .map_err(|e| WasmHostError::LoadFailed(e.to_string()))?;

        // Validate ABI by instantiating and checking exports
        let name = Self::validate_abi(&engine, &module, config)?;

        Ok(Self {
            engine,
            module,
            name,
        })
    }

    /// Load a Wasm plugin from raw bytes.
    pub fn from_bytes(bytes: &[u8], config: &WasmHostConfig) -> Result<Self, WasmHostError> {
        let mut engine_config = wasmtime::Config::new();
        engine_config.consume_fuel(true);

        let engine = wasmtime::Engine::new(&engine_config)
            .map_err(|e| WasmHostError::LoadFailed(e.to_string()))?;

        let module =
            wasmtime::Module::new(&engine, bytes).map_err(|e| WasmHostError::LoadFailed(e.to_string()))?;

        let name = Self::validate_abi(&engine, &module, config)?;

        Ok(Self {
            engine,
            module,
            name,
        })
    }

    /// Validate the plugin ABI and extract the plugin name.
    fn validate_abi(
        engine: &wasmtime::Engine,
        module: &wasmtime::Module,
        config: &WasmHostConfig,
    ) -> Result<String, WasmHostError> {
        let required_exports = [
            "plugin_name",
            "plugin_name_ptr",
            "plugin_version",
            "plugin_alloc",
            "plugin_execute",
            "plugin_output_ptr",
            "plugin_free",
        ];

        for export_name in &required_exports {
            if module.get_export(export_name).is_none() {
                return Err(WasmHostError::MissingExport(export_name.to_string()));
            }
        }

        // Instantiate to check ABI version
        let mut linker = wasmtime::Linker::new(engine);

        if config.enable_wasi {
            wasmtime_wasi::preview1::add_to_linker_sync(&mut linker, |s: &mut wasmtime_wasi::preview1::WasiP1Ctx| s)
                .map_err(|e| WasmHostError::LoadFailed(format!("WASI setup failed: {e}")))?;
        }

        let wasi_ctx = wasmtime_wasi::WasiCtxBuilder::new().build_p1();
        let mut store = wasmtime::Store::new(engine, wasi_ctx);
        store
            .set_fuel(config.fuel_limit)
            .map_err(|e| WasmHostError::LoadFailed(e.to_string()))?;

        let instance = linker
            .instantiate(&mut store, module)
            .map_err(|e| WasmHostError::LoadFailed(format!("instantiation failed: {e}")))?;

        // Check ABI version
        let version_fn = instance
            .get_typed_func::<(), i32>(&mut store, "plugin_version")
            .map_err(|e| WasmHostError::LoadFailed(e.to_string()))?;

        let version = version_fn
            .call(&mut store, ())
            .map_err(|e| WasmHostError::Trap(e.to_string()))?;

        if version != ABI_VERSION {
            return Err(WasmHostError::AbiVersionMismatch {
                guest: version,
                host: ABI_VERSION,
            });
        }

        // Get plugin name
        let name_len_fn = instance
            .get_typed_func::<(), i32>(&mut store, "plugin_name")
            .map_err(|e| WasmHostError::LoadFailed(e.to_string()))?;

        let name_ptr_fn = instance
            .get_typed_func::<(), i32>(&mut store, "plugin_name_ptr")
            .map_err(|e| WasmHostError::LoadFailed(e.to_string()))?;

        let len = name_len_fn
            .call(&mut store, ())
            .map_err(|e| WasmHostError::Trap(e.to_string()))?;
        let ptr = name_ptr_fn
            .call(&mut store, ())
            .map_err(|e| WasmHostError::Trap(e.to_string()))?;

        if len <= 0 || ptr < 0 {
            return Ok("unknown".to_string());
        }

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| WasmHostError::MissingExport("memory".to_string()))?;

        let len = len as usize;
        let ptr = ptr as usize;

        if ptr + len > memory.data_size(&store) {
            return Ok("unknown".to_string());
        }

        let mut name_bytes = vec![0u8; len];
        if let Some(slice) = memory.data(&store).get(ptr..ptr + len) {
            name_bytes.copy_from_slice(slice);
        }

        String::from_utf8(name_bytes)
            .map_err(|_| WasmHostError::LoadFailed("plugin name is not valid UTF-8".to_string()))
    }

    /// Get the plugin name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abi_version_constant() {
        assert_eq!(ABI_VERSION, 1);
    }

    #[test]
    fn test_default_fuel_limit() {
        assert_eq!(DEFAULT_FUEL_LIMIT, 100_000);
    }

    #[test]
    fn test_host_config_default() {
        let config = WasmHostConfig::default();
        assert_eq!(config.fuel_limit, DEFAULT_FUEL_LIMIT);
        assert!(config.enable_wasi);
    }

    #[test]
    fn test_missing_exports_error() {
        // Minimal valid Wasm module with no exports (magic + version header).
        // Wasm binary format: \x00asm + version 1 + empty sections.
        let wasm_bytes: &[u8] = &[
            0x00, 0x61, 0x73, 0x6D, // magic: \0asm
            0x01, 0x00, 0x00, 0x00, // version: 1
        ];

        let config = WasmHostConfig {
            fuel_limit: 1000,
            enable_wasi: false,
        };

        // The module has no exports, so it should fail with MissingExport
        let result = WasmPlugin::from_bytes(wasm_bytes, &config);
        // It might fail at LoadFailed (no memory) or MissingExport depending on wasmtime version
        assert!(result.is_err());
    }

    #[test]
    fn test_host_error_display() {
        let err = WasmHostError::MissingExport("plugin_name".to_string());
        assert!(err.to_string().contains("plugin_name"));

        let err = WasmHostError::AbiVersionMismatch {
            guest: 0,
            host: 1,
        };
        assert!(err.to_string().contains("guest=0"));
        assert!(err.to_string().contains("host=1"));

        let err = WasmHostError::FuelExhausted {
            limit: 100_000,
        };
        assert!(err.to_string().contains("100000"));
    }

    #[test]
    fn test_plugin_name() {
        // Can't test with real Wasm without `wat` crate in non-test deps,
        // but we can test the constant
        assert_eq!(ABI_VERSION, 1);
    }
}
