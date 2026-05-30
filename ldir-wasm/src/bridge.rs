//! WASM Bridge — interface between host and WASM guest.
//!
//! `WasmBridge` manages the lifecycle of a WASM module, providing
//! methods to load modules and invoke compile/render functions.
//!
//! All actual WASM runtime interaction is behind the optional
//! `wasmtime` feature. Without it, the bridge operates in mock mode.

/// Error type for bridge operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeError {
    /// WASM module loading failed.
    LoadFailed(String),
    /// WASM instantiation failed.
    InstantiationFailed(String),
    /// Function invocation failed.
    InvocationFailed(String),
    /// Returned pointer is out of bounds.
    PointerOutOfBounds {
        /// The invalid pointer value.
        ptr: u32,
        /// The requested length.
        len: u32,
    },
    /// Version mismatch between host and guest.
    VersionMismatch {
        /// The expected version.
        expected: u32,
        /// The actual version reported by the guest.
        actual: u32,
    },
    /// WASM runtime not available (feature not enabled).
    RuntimeUnavailable,
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LoadFailed(msg) => write!(f, "failed to load WASM module: {msg}"),
            Self::InstantiationFailed(msg) => write!(f, "failed to instantiate WASM module: {msg}"),
            Self::InvocationFailed(msg) => write!(f, "WASM invocation failed: {msg}"),
            Self::PointerOutOfBounds { ptr, len } => {
                write!(f, "WASM pointer out of bounds: ptr={ptr}, len={len}")
            }
            Self::VersionMismatch { expected, actual } => {
                write!(f, "version mismatch: expected {expected}, got {actual}")
            }
            Self::RuntimeUnavailable => write!(f, "WASM runtime not available"),
        }
    }
}

impl std::error::Error for BridgeError {}

/// Result type for bridge operations.
pub type BridgeResult<T> = std::result::Result<T, BridgeError>;

/// State tracking for a loaded WASM module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeState {
    /// No module loaded.
    Unloaded,
    /// Module loaded but not instantiated.
    Loaded,
    /// Module instantiated and ready for calls.
    Ready,
    /// An error occurred.
    Error,
}

/// Interface between host and WASM guest.
///
/// Manages loading WASM modules and invoking exported compile/render
/// functions. Operates in mock mode without the `wasmtime` feature.
#[derive(Debug, Clone)]
pub struct WasmBridge {
    state: BridgeState,
    #[allow(dead_code)]
    module_size: usize,
}

impl WasmBridge {
    /// Create a new bridge in unloaded state.
    pub fn new() -> Self {
        Self {
            state: BridgeState::Unloaded,
            module_size: 0,
        }
    }

    /// Get the current bridge state.
    pub fn state(&self) -> BridgeState {
        self.state
    }

    /// Load a WASM module from bytes.
    ///
    /// Validates basic WASM magic bytes (0x00 0x61 0x73 0x6D).
    /// Without the `wasmtime` feature, this only validates the header.
    pub fn load_wasm(&mut self, wasm_bytes: &[u8]) -> BridgeResult<()> {
        if wasm_bytes.len() < 8 {
            return Err(BridgeError::LoadFailed("module too small".to_string()));
        }

        if wasm_bytes[0..4] != [0x00, 0x61, 0x73, 0x6D] {
            return Err(BridgeError::LoadFailed(
                "invalid WASM magic number".to_string(),
            ));
        }

        self.module_size = wasm_bytes.len();
        self.state = BridgeState::Loaded;
        Ok(())
    }

    /// Invoke the WASM compile function.
    ///
    /// Takes a pointer and length into WASM linear memory containing
    /// S-IR data. Returns a pointer to the compiled G-IR output.
    ///
    /// In mock mode, returns 0 (null pointer).
    pub fn call_compile(&mut self, sir_ptr: u32, sir_len: u32) -> BridgeResult<u32> {
        match self.state {
            BridgeState::Ready => {}
            BridgeState::Loaded => {
                // Auto-instantiate in mock mode
                self.state = BridgeState::Ready;
            }
            _ => {
                return Err(BridgeError::InvocationFailed(
                    "bridge not ready".to_string(),
                ));
            }
        }

        if sir_ptr == 0 && sir_len > 0 {
            return Err(BridgeError::PointerOutOfBounds {
                ptr: sir_ptr,
                len: sir_len,
            });
        }

        Ok(0)
    }

    /// Invoke the WASM render function.
    ///
    /// Takes a pointer to G-IR data and target dimensions.
    /// Returns a pointer to the rendered RGBA pixel buffer.
    ///
    /// In mock mode, returns 0 (null pointer).
    pub fn call_render(&mut self, gir_ptr: u32, width: u32, height: u32) -> BridgeResult<u32> {
        match self.state {
            BridgeState::Ready => {}
            BridgeState::Loaded => {
                self.state = BridgeState::Ready;
            }
            _ => {
                return Err(BridgeError::InvocationFailed(
                    "bridge not ready".to_string(),
                ));
            }
        }

        if gir_ptr == 0 {
            return Err(BridgeError::PointerOutOfBounds {
                ptr: gir_ptr,
                len: width * height * 4,
            });
        }

        Ok(0)
    }

    /// Unload the current module and reset state.
    pub fn unload(&mut self) {
        self.state = BridgeState::Unloaded;
        self.module_size = 0;
    }

    /// Check if the bridge is ready for function calls.
    pub fn is_ready(&self) -> bool {
        self.state == BridgeState::Ready
    }
}

impl Default for WasmBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_new() {
        let bridge = WasmBridge::new();
        assert_eq!(bridge.state(), BridgeState::Unloaded);
        assert!(!bridge.is_ready());
    }

    #[test]
    fn test_bridge_default() {
        let bridge = WasmBridge::default();
        assert_eq!(bridge.state(), BridgeState::Unloaded);
    }

    #[test]
    fn test_load_wasm_valid_magic() {
        let mut bridge = WasmBridge::new();
        // Minimal valid-looking WASM header: magic + version
        let wasm_bytes: &[u8] = &[0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
        let result = bridge.load_wasm(wasm_bytes);
        assert!(result.is_ok());
        assert_eq!(bridge.state(), BridgeState::Loaded);
    }

    #[test]
    fn test_load_wasm_invalid_magic() {
        let mut bridge = WasmBridge::new();
        let wasm_bytes: &[u8] = &[0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x00, 0x00, 0x00];
        let result = bridge.load_wasm(wasm_bytes);
        assert!(result.is_err());
        assert_eq!(bridge.state(), BridgeState::Unloaded);
    }

    #[test]
    fn test_load_wasm_too_small() {
        let mut bridge = WasmBridge::new();
        let wasm_bytes: &[u8] = &[0x00, 0x61];
        let result = bridge.load_wasm(wasm_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_call_compile_not_ready() {
        let mut bridge = WasmBridge::new();
        let result = bridge.call_compile(0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_call_compile_auto_instantiate() {
        let mut bridge = WasmBridge::new();
        let wasm_bytes: &[u8] = &[0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
        bridge.load_wasm(wasm_bytes).unwrap();
        let result = bridge.call_compile(1024, 100);
        assert!(result.is_ok());
        assert!(bridge.is_ready());
    }

    #[test]
    fn test_call_compile_null_pointer_with_length() {
        let mut bridge = WasmBridge::new();
        let wasm_bytes: &[u8] = &[0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
        bridge.load_wasm(wasm_bytes).unwrap();
        bridge.state = BridgeState::Ready;
        let result = bridge.call_compile(0, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_call_render_not_ready() {
        let mut bridge = WasmBridge::new();
        let result = bridge.call_render(0, 100, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_call_render_auto_instantiate() {
        let mut bridge = WasmBridge::new();
        let wasm_bytes: &[u8] = &[0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
        bridge.load_wasm(wasm_bytes).unwrap();
        let result = bridge.call_render(1024, 100, 100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_render_null_pointer() {
        let mut bridge = WasmBridge::new();
        let wasm_bytes: &[u8] = &[0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
        bridge.load_wasm(wasm_bytes).unwrap();
        bridge.state = BridgeState::Ready;
        let result = bridge.call_render(0, 100, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_unload() {
        let mut bridge = WasmBridge::new();
        let wasm_bytes: &[u8] = &[0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
        bridge.load_wasm(wasm_bytes).unwrap();
        bridge.unload();
        assert_eq!(bridge.state(), BridgeState::Unloaded);
        assert!(!bridge.is_ready());
    }
}
