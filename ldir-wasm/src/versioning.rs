//! WASM Interface Version Negotiation.
//!
//! Provides version checking between the host and WASM guest to
//! ensure compatibility.

/// Current WASM interface version.
pub const CURRENT_VERSION: u32 = 1;

/// Minimum supported version.
pub const MIN_VERSION: u32 = 1;

/// Version negotiation result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionCheck {
    /// Versions are compatible.
    Compatible,
    /// Guest version is too old.
    GuestTooOld {
        /// Minimum supported version.
        expected: u32,
        /// The guest's reported version.
        actual: u32,
    },
    /// Guest version is too new (host may need update).
    GuestTooNew {
        /// Maximum supported version.
        expected: u32,
        /// The guest's reported version.
        actual: u32,
    },
}

/// WASM interface version negotiation.
///
/// Provides methods to check compatibility between the host's
/// expected version and the guest module's reported version.
pub struct WasmInterfaceVersion {
    /// The host's expected interface version.
    host_version: u32,
}

impl WasmInterfaceVersion {
    /// Create a new version negotiator with the current host version.
    pub fn new() -> Self {
        Self {
            host_version: CURRENT_VERSION,
        }
    }

    /// Create a version negotiator with a specific host version.
    pub fn with_version(version: u32) -> Self {
        Self {
            host_version: version,
        }
    }

    /// Get the host's expected version.
    pub fn host_version(&self) -> u32 {
        self.host_version
    }

    /// Check if a guest version is compatible with the host version.
    ///
    /// A guest is compatible if its version is >= `MIN_VERSION` and
    /// <= `host_version`.
    pub fn check(&self, guest_version: u32) -> VersionCheck {
        if guest_version < MIN_VERSION {
            VersionCheck::GuestTooOld {
                expected: MIN_VERSION,
                actual: guest_version,
            }
        } else if guest_version > self.host_version {
            VersionCheck::GuestTooNew {
                expected: self.host_version,
                actual: guest_version,
            }
        } else {
            VersionCheck::Compatible
        }
    }

    /// Check compatibility, returning `Ok(())` if compatible.
    pub fn ensure_compatible(&self, guest_version: u32) -> Result<(), VersionCheck> {
        match self.check(guest_version) {
            VersionCheck::Compatible => Ok(()),
            incompat => Err(incompat),
        }
    }
}

impl Default for WasmInterfaceVersion {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_version() {
        assert_eq!(CURRENT_VERSION, 1);
        assert_eq!(MIN_VERSION, 1);
    }

    #[test]
    fn test_version_new() {
        let v = WasmInterfaceVersion::new();
        assert_eq!(v.host_version(), CURRENT_VERSION);
    }

    #[test]
    fn test_version_default() {
        let v = WasmInterfaceVersion::default();
        assert_eq!(v.host_version(), CURRENT_VERSION);
    }

    #[test]
    fn test_version_with_version() {
        let v = WasmInterfaceVersion::with_version(2);
        assert_eq!(v.host_version(), 2);
    }

    #[test]
    fn test_check_compatible() {
        let v = WasmInterfaceVersion::new();
        assert_eq!(v.check(1), VersionCheck::Compatible);
    }

    #[test]
    fn test_check_guest_too_old() {
        let v = WasmInterfaceVersion::new();
        assert_eq!(
            v.check(0),
            VersionCheck::GuestTooOld {
                expected: 1,
                actual: 0
            }
        );
    }

    #[test]
    fn test_check_guest_too_new() {
        let v = WasmInterfaceVersion::new();
        assert_eq!(
            v.check(2),
            VersionCheck::GuestTooNew {
                expected: 1,
                actual: 2
            }
        );
    }

    #[test]
    fn test_ensure_compatible_ok() {
        let v = WasmInterfaceVersion::new();
        assert!(v.ensure_compatible(1).is_ok());
    }

    #[test]
    fn test_ensure_compatible_err_old() {
        let v = WasmInterfaceVersion::new();
        assert!(v.ensure_compatible(0).is_err());
    }

    #[test]
    fn test_ensure_compatible_err_new() {
        let v = WasmInterfaceVersion::new();
        assert!(v.ensure_compatible(5).is_err());
    }

    #[test]
    fn test_version_negotiation_range() {
        let v = WasmInterfaceVersion::with_version(3);
        assert_eq!(v.check(1), VersionCheck::Compatible);
        assert_eq!(v.check(2), VersionCheck::Compatible);
        assert_eq!(v.check(3), VersionCheck::Compatible);
        assert!(matches!(v.check(0), VersionCheck::GuestTooOld { .. }));
        assert!(matches!(v.check(4), VersionCheck::GuestTooNew { .. }));
    }
}
