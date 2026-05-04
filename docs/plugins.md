# LDIR Plugin System

LDIR's plugin system allows you to extend the compiler with custom input parsers (frontends) and output generators (backends). Plugins are trait-based and registered through a central `PluginRegistry`.

## Overview

```rust
use ldir_core::plugin::{PluginRegistry, FrontendPlugin, BackendPlugin};

let mut registry = PluginRegistry::new();
registry.register_frontend(Box::new(MyFrontend));
registry.register_backend(Box::new(MyBackend));
```

The plugin system lives in `ldir-core` and works with S-IR v2 (`SIRModuleV2`) as the interchange format. Frontends parse files into S-IR v2; backends convert S-IR v2 to output bytes.

## FrontendPlugin Trait

Implement `FrontendPlugin` to add a new input format:

```rust
use ldir_core::plugin::{FrontendPlugin, PluginError};
use ldir_ir::sir::v2::SIRModuleV2;
use std::path::Path;

pub struct MyFrontend;

impl FrontendPlugin for MyFrontend {
    fn name(&self) -> &str {
        "MyFormat"
    }

    fn extensions(&self) -> &[&str] {
        &["myf", "myformat"]
    }

    fn parse_file(&self, path: &Path) -> Result<SIRModuleV2, PluginError> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| PluginError::IoError(e.to_string()))?;
        self.parse_string(&text, &path.to_string_lossy())
    }

    fn parse_string(&self, text: &str, source_name: &str) -> Result<SIRModuleV2, PluginError> {
        let mut module = SIRModuleV2::new();
        // ... parse `text` into S-IR v2 nodes ...
        // For example, add a paragraph:
        // let paragraph = SIRNodeV2 {
        //     id: 0,
        //     node_type: NodeType::Paragraph,
        //     parent_id: Some(ROOT_ID),
        //     child_ids: vec![],
        //     content: text.to_string().into(),
        //     ..Default::default()
        // };
        // module.body.push(paragraph);
        Ok(module)
    }
}
```

### Trait methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `name()` | `&self) -> &str` | Human-readable name for the format (e.g., "Markdown", "LaTeX") |
| `extensions()` | `&self) -> &[&str]` | File extensions handled (e.g., `["md", "markdown"]`) |
| `parse_file()` | `&self, path: &Path) -> Result<SIRModuleV2, PluginError>` | Parse a file on disk |
| `parse_string()` | `&self, text: &str, source_name: &str) -> Result<SIRModuleV2, PluginError>` | Parse a string in memory |

The `source_name` parameter in `parse_string` is used for error reporting (e.g., the file path or a synthetic name like `<stdin>`).

## BackendPlugin Trait

Implement `BackendPlugin` to add a new output format:

```rust
use ldir_core::plugin::{BackendPlugin, GenerateOptions, PluginError};
use ldir_ir::sir::v2::SIRModuleV2;

pub struct MyBackend;

impl BackendPlugin for MyBackend {
    fn name(&self) -> &str {
        "MyOutput"
    }

    fn extension(&self) -> &str {
        "myout"
    }

    fn generate(
        &self,
        module: &SIRModuleV2,
        options: &GenerateOptions,
    ) -> Result<Vec<u8>, PluginError> {
        // ... convert S-IR v2 to your output format ...
        let output = format!("/* MyOutput: {} nodes */\n", module.body.len());
        Ok(output.into_bytes())
    }
}
```

### Trait methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `name()` | `&self) -> &str` | Human-readable name for the format (e.g., "PDF", "HTML") |
| `extension()` | `&self) -> &str` | File extension for output files (e.g., "pdf") |
| `generate()` | `&self, module: &SIRModuleV2, options: &GenerateOptions) -> Result<Vec<u8>, PluginError>` | Convert S-IR v2 to output bytes |

### GenerateOptions

```rust
pub struct GenerateOptions {
    pub output_path: Option<PathBuf>,     // Output file path (if known)
    pub page_size: Option<String>,         // "a4", "letter", etc.
    pub font_family: Option<String>,       // Font family name
    pub extra: HashMap<String, String>,    // Format-specific options
}
```

Default implementation provides `None` for all fields.

## Registering Plugins

Create a registry and register your plugins:

```rust
use ldir_core::plugin::PluginRegistry;

let mut registry = PluginRegistry::new();

// Register frontends
registry.register_frontend(Box::new(MyFrontend));

// Register backends
registry.register_backend(Box::new(MyBackend));
```

### Looking up plugins

```rust
// Find frontend by file extension
let frontend = registry.find_frontend("myf");
if let Some(fe) = frontend {
    let module = fe.parse_string("content here", "<test>")?;
}

// Find backend by name or extension
let backend = registry.find_backend("MyOutput");  // by name
let backend = registry.find_backend("myout");     // by extension

if let Some(be) = backend {
    let bytes = be.generate(&module, &GenerateOptions::default())?;
}
```

### Listing plugins

```rust
let frontend_names = registry.frontend_names();  // Vec<&str>
let backend_names = registry.backend_names();    // Vec<&str>
println!("Frontends: {:?}", frontend_names);
println!("Backends: {:?}", backend_names);
```

## Example: Statistics Backend

This backend walks the S-IR v2 tree and outputs document statistics as plain text:

```rust
use ldir_core::plugin::{BackendPlugin, GenerateOptions, PluginError};
use ldir_ir::sir::v2::{SIRModuleV2, NodeType};

pub struct StatsBackend;

impl BackendPlugin for StatsBackend {
    fn name(&self) -> &str {
        "Statistics"
    }

    fn extension(&self) -> &str {
        "stats"
    }

    fn generate(
        &self,
        module: &SIRModuleV2,
        _options: &GenerateOptions,
    ) -> Result<Vec<u8>, PluginError> {
        let mut sections = 0u32;
        let mut paragraphs = 0u32;
        let mut lists = 0u32;
        let mut code_blocks = 0u32;
        let mut total_chars = 0u32;

        for node in &module.body {
            match node.node_type {
                NodeType::Section
                | NodeType::Subsection
                | NodeType::Subsubsection => sections += 1,
                NodeType::Paragraph => paragraphs += 1,
                NodeType::List { .. } => lists += 1,
                NodeType::CodeBlock => code_blocks += 1,
                _ => {}
            }
            if let Some(ref content) = node.content {
                total_chars += content.chars().count() as u32;
            }
        }

        let report = format!(
            "LDIR Document Statistics\n\
             ========================\n\
             Total nodes:      {}\n\
             Sections:         {}\n\
             Paragraphs:       {}\n\
             Lists:            {}\n\
             Code blocks:      {}\n\
             Total characters: {}\n",
            module.body.len(),
            sections,
            paragraphs,
            lists,
            code_blocks,
            total_chars,
        );

        Ok(report.into_bytes())
    }
}
```

Usage:

```rust
use ldir_core::plugin::PluginRegistry;

let mut registry = PluginRegistry::new();
registry.register_backend(Box::new(StatsBackend));

// Parse input with a frontend (e.g., markdown)
let module = parse_markdown_to_sir_v2("# My Doc\n\nSome text.");

// Generate statistics
let backend = registry.find_backend("stats").unwrap();
let stats = backend.generate(&module, &GenerateOptions::default())?;
println!("{}", String::from_utf8_lossy(&stats));
```

## Error Handling

Plugins return `PluginError` variants:

```rust
pub enum PluginError {
    ParseFailed(String),      // Input parsing failed
    GenerateFailed(String),   // Output generation failed
    UnsupportedFormat(String),// Format not supported
    IoError(String),          // I/O error
}
```

`PluginError` implements `Display` and `Error`, so it integrates with `anyhow` and `std::error`:
