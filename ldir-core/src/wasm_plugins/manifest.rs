use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use std::collections::HashMap;

/// Plugin manifest describing a ldir plugin.
/// Plugins declare their capabilities and requirements in a manifest file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginManifest {
    /// Plugin name (unique identifier)
    pub name: String,
    /// Semantic version (e.g., "0.1.0")
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Author or organization
    pub author: String,
    /// License (SPDX identifier)
    pub license: String,
    /// Homepage URL
    pub homepage: Option<String>,
    /// Repository URL
    pub repository: Option<String>,
    /// Plugin capabilities
    pub capabilities: Vec<PluginCapability>,
    /// Resource limits requested by the plugin
    pub resource_limits: ResourceLimits,
    /// Required ldir version range (semver constraint)
    pub requires_ldir: Option<String>,
    /// Plugin-specific configuration schema
    pub config_schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// A single capability declared by a plugin.
pub struct PluginCapability {
    /// Capability type
    pub kind: CapabilityKind,
    /// Human-readable description
    pub description: String,
    /// File extensions this capability handles (for input/output plugins)
    pub file_extensions: Vec<String>,
    /// MIME types this capability handles
    pub mime_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// The pipeline stage a plugin capability hooks into.
pub enum CapabilityKind {
    /// Transforms text before parsing (macro expansion, include resolution)
    PreProcessor,
    /// Custom output format writer
    OutputFormat,
    /// Custom input format reader
    InputFormat,
    /// Modifies S-IR during compilation
    IrTransformer,
    /// Modifies L-IR during layout
    LayoutModifier,
    /// Adds custom citation style
    CitationStyle,
    /// Adds custom code block renderer
    CodeRenderer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// Resource limits a plugin may consume; defaults are conservative.
pub struct ResourceLimits {
    /// Maximum fuel (WASM instructions) per execution
    pub max_fuel: u64,
    /// Maximum memory allocation (bytes)
    pub max_memory_mb: u32,
    /// Maximum execution time (milliseconds)
    pub max_time_ms: u32,
    /// Maximum output size (bytes)
    pub max_output_kb: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_fuel: 100_000,
            max_memory_mb: 64,
            max_time_ms: 1000,
            max_output_kb: 1024,
        }
    }
}

/// Parse a plugin manifest from JSON
pub fn parse_manifest(json: &str) -> Result<PluginManifest, ManifestError> {
    serde_json::from_str(json).map_err(|e| ManifestError::ParseError(e.to_string()))
}

/// Serialize a plugin manifest to JSON
pub fn serialize_manifest(manifest: &PluginManifest) -> String {
    serde_json::to_string_pretty(manifest).unwrap_or_default()
}

#[derive(Debug, thiserror::Error)]
/// Errors from parsing or validating a plugin manifest.
pub enum ManifestError {
    #[error("Parse error: {0}")]
    /// The manifest is not valid JSON. Contains the serde error message.
    ParseError(String),
    #[error("Validation error: {0}")]
    /// The manifest is well-formed but invalid. Contains a description.
    ValidationError(String),
}

/// Validate a plugin manifest
pub fn validate_manifest(manifest: &PluginManifest) -> Result<(), ManifestError> {
    if manifest.name.is_empty() {
        return Err(ManifestError::ValidationError(
            "Plugin name cannot be empty".into(),
        ));
    }
    if manifest.version.is_empty() {
        return Err(ManifestError::ValidationError(
            "Plugin version cannot be empty".into(),
        ));
    }
    if manifest.capabilities.is_empty() {
        return Err(ManifestError::ValidationError(
            "Plugin must declare at least one capability".into(),
        ));
    }
    if manifest.resource_limits.max_memory_mb == 0 {
        return Err(ManifestError::ValidationError(
            "max_memory_mb must be > 0".into(),
        ));
    }
    if manifest.resource_limits.max_fuel == 0 {
        return Err(ManifestError::ValidationError(
            "max_fuel must be > 0".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_manifest_json() -> String {
        r#"{
  "name": "test-plugin",
  "version": "0.1.0",
  "description": "A test plugin",
  "author": "Test Author",
  "license": "MIT",
  "homepage": null,
  "repository": null,
  "capabilities": [
    {
      "kind": "output_format",
      "description": "Outputs HTML",
      "file_extensions": ["html"],
      "mime_types": ["text/html"]
    }
  ],
  "resource_limits": {
    "max_fuel": 50000,
    "max_memory_mb": 32,
    "max_time_ms": 500,
    "max_output_kb": 512
  },
  "requires_ldir": ">=0.5.0",
  "config_schema": null
}"#
        .to_string()
    }

    #[test]
    fn test_parse_valid() {
        let json = valid_manifest_json();
        let manifest = parse_manifest(&json).expect("parse should succeed");
        assert_eq!(manifest.name, "test-plugin");
        assert_eq!(manifest.version, "0.1.0");
        assert_eq!(manifest.capabilities.len(), 1);
        assert_eq!(manifest.capabilities[0].kind, CapabilityKind::OutputFormat);
        assert_eq!(manifest.resource_limits.max_fuel, 50_000);
        assert_eq!(manifest.resource_limits.max_memory_mb, 32);
    }

    #[test]
    fn test_parse_invalid_json() {
        let result = parse_manifest("not json at all");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, ManifestError::ParseError(_)),
            "expected ParseError, got {err:?}"
        );
    }

    #[test]
    fn test_validate_ok() {
        let json = valid_manifest_json();
        let manifest = parse_manifest(&json).unwrap();
        validate_manifest(&manifest).expect("validation should pass");
    }

    #[test]
    fn test_validate_errors() {
        let base_json = valid_manifest_json();

        let cases: Vec<(fn(&mut PluginManifest), &str)> = vec![
            (|m| m.name.clear(), "name"),
            (|m| m.version.clear(), "version"),
            (|m| m.capabilities.clear(), "capabilities"),
            (|m| m.resource_limits.max_memory_mb = 0, "max_memory_mb"),
            (|m| m.resource_limits.max_fuel = 0, "max_fuel"),
        ];

        for (mutate, label) in cases {
            let mut manifest = parse_manifest(&base_json).unwrap();
            mutate(&mut manifest);
            let err = validate_manifest(&manifest).expect_err(&format!("{label} should fail"));
            assert!(
                matches!(err, ManifestError::ValidationError(_)),
                "{label}: expected ValidationError, got {err:?}"
            );
        }
    }

    #[test]
    fn test_default_limits() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_fuel, 100_000);
        assert_eq!(limits.max_memory_mb, 64);
        assert_eq!(limits.max_time_ms, 1000);
        assert_eq!(limits.max_output_kb, 1024);
    }

    #[test]
    fn test_manifest_roundtrip() {
        let json = valid_manifest_json();
        let manifest = parse_manifest(&json).unwrap();
        let serialized = serialize_manifest(&manifest);
        let deserialized = parse_manifest(&serialized).unwrap();
        assert_eq!(manifest, deserialized);
    }
}
