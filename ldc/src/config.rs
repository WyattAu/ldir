//! Configuration file support for ldc.
//!
//! Reads `ldir.toml` from multiple search paths:
//! 1. `./ldir.toml` or `./.ldir.toml` (current directory)
//! 2. Parent directories up to filesystem root
//! 3. `$XDG_CONFIG_HOME/ldir/config.toml` or `~/.config/ldir/config.toml`
//!
//! Supports both structured (nested `[section]`) and flat (legacy) TOML formats.
//! Precedence: CLI flags > config file > defaults.

use std::fmt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::cli::Cli;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ConfigError {
    InvalidValue { field: String, reason: String },
    UnknownFormat(String),
    EmptyFontName(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidValue { field, reason } => {
                write!(f, "invalid value for '{field}': {reason}")
            }
            Self::UnknownFormat(v) => {
                write!(
                    f,
                    "unknown format '{v}'. expected: {}",
                    KNOWN_FORMATS.join(", ")
                )
            }
            Self::EmptyFontName(n) => write!(f, "empty font name for '{n}'"),
        }
    }
}

impl std::error::Error for ConfigError {}

const KNOWN_FORMATS: &[&str] = &[
    "pdf", "html", "epub", "docx", "gir", "sir", "sir2", "ldir", "txt",
];
const KNOWN_PDFA_LEVELS: &[&str] = &["off", "1b", "2b", "3b", "4"];
const KNOWN_PDF_VERSIONS: &[&str] = &["1.4", "1.5", "1.6", "1.7"];
const KNOWN_CITATION_STYLES: &[&str] = &["ieee", "apa", "chicago"];

// ---------------------------------------------------------------------------
// Section structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct OutputConfig {
    pub format: Option<String>,
    pub pdfa_level: Option<String>,
    pub pdf_version: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct LayoutConfig {
    pub page_width: Option<String>,
    pub page_height: Option<String>,
    pub margin: Option<String>,
    pub font_size: Option<String>,
    pub line_height: Option<f64>,
    pub columns: Option<u32>,
    pub column_gap: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct FontsConfig {
    pub serif: Option<String>,
    pub sans: Option<String>,
    pub mono: Option<String>,
    pub math: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct TypographyConfig {
    pub hyphenation: Option<bool>,
    pub hyphenation_lang: Option<String>,
    pub ligatures: Option<bool>,
    pub kerning: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct LatexConfig {
    pub macros: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CitationsConfig {
    pub bibliography: Option<String>,
    pub style: Option<String>,
}

// ---------------------------------------------------------------------------
// Output field -- handles both `[output]` section and `output = "path"` legacy
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum OutputField {
    Section(OutputConfig),
    Path(String),
}

impl Default for OutputField {
    fn default() -> Self {
        OutputField::Section(OutputConfig::default())
    }
}

// ---------------------------------------------------------------------------
// Top-level config
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct LdirConfig {
    // Structured sections (new format)
    pub output: Option<OutputField>,
    pub layout: Option<LayoutConfig>,
    pub fonts: Option<FontsConfig>,
    pub typography: Option<TypographyConfig>,
    pub latex: Option<LatexConfig>,
    pub citations: Option<CitationsConfig>,

    // Legacy flat fields (backward compat)
    pub format: Option<String>,
    pub font: Option<String>,
    pub font_mono: Option<String>,
    pub font_path: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub margin: Option<f64>,
    pub page_size: Option<String>,
    pub page_width: Option<f64>,
    pub page_height: Option<f64>,
    pub header_left: Option<String>,
    pub header_center: Option<String>,
    pub header_right: Option<String>,
    pub footer_left: Option<String>,
    pub footer_center: Option<String>,
    pub footer_right: Option<String>,
    pub no_header_rule: Option<bool>,
    pub no_footer_rule: Option<bool>,
    pub drop_caps: Option<bool>,
    pub bibliography: Option<String>,
    pub pdfa_level: Option<String>,
    pub ot_features: Option<String>,
}

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

impl LdirConfig {
    /// Load config. If `no_config` is true, returns defaults without any file I/O.
    /// If `path` is given, loads from that exact path.
    /// Otherwise searches standard locations.
    pub fn load(path: Option<&Path>, no_config: bool) -> Result<Self> {
        if no_config {
            return Ok(Self::default());
        }
        if let Some(p) = path {
            return Self::load_from_path(p);
        }
        if let Some(result) = Self::search_and_load() {
            return result;
        }
        Ok(Self::default())
    }

    fn load_from_path(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read config: {}", path.display()))?;
        toml::from_str::<Self>(&content)
            .with_context(|| format!("failed to parse config: {}", path.display()))
    }

    fn search_and_load() -> Option<Result<Self>> {
        // Current directory
        for name in ["ldir.toml", ".ldir.toml"] {
            let p = PathBuf::from(name);
            if p.is_file() {
                return Some(Self::load_from_path(&p));
            }
        }
        // Walk up parent directories (like git searching for .git)
        let mut dir = std::env::current_dir().ok()?;
        loop {
            for name in ["ldir.toml", ".ldir.toml"] {
                let p = dir.join(name);
                if p.is_file() {
                    return Some(Self::load_from_path(&p));
                }
            }
            if !dir.pop() {
                break;
            }
        }
        // XDG config home
        if let Some(xdg) = xdg_config_dir() {
            let p = xdg.join("ldir").join("config.toml");
            if p.is_file() {
                return Some(Self::load_from_path(&p));
            }
        }
        None
    }

    // -----------------------------------------------------------------------
    // Validation
    // -----------------------------------------------------------------------

    pub fn validate(&self) -> std::result::Result<(), ConfigError> {
        let format = self.resolved_format();
        if !KNOWN_FORMATS.contains(&format.as_str()) {
            return Err(ConfigError::UnknownFormat(format));
        }
        if let Some(ref level) = self.resolved_pdfa_level()
            && !KNOWN_PDFA_LEVELS.contains(&level.as_str())
        {
            return Err(ConfigError::InvalidValue {
                field: "output.pdfa_level".into(),
                reason: format!(
                    "'{level}'. expected one of: {}",
                    KNOWN_PDFA_LEVELS.join(", ")
                ),
            });
        }
        if let Some(ref ver) = self.resolved_pdf_version()
            && !KNOWN_PDF_VERSIONS.contains(&ver.as_str())
        {
            return Err(ConfigError::InvalidValue {
                field: "output.pdf_version".into(),
                reason: format!(
                    "'{ver}'. expected one of: {}",
                    KNOWN_PDF_VERSIONS.join(", ")
                ),
            });
        }
        if let Some(ref fonts) = self.fonts {
            for (name, val) in [
                ("fonts.serif", &fonts.serif),
                ("fonts.sans", &fonts.sans),
                ("fonts.mono", &fonts.mono),
                ("fonts.math", &fonts.math),
            ] {
                if let Some(v) = val
                    && v.trim().is_empty()
                {
                    return Err(ConfigError::EmptyFontName(name.into()));
                }
            }
        }
        if let Some(ref layout) = self.layout {
            if let Some(ref w) = layout.page_width {
                parse_dim(w).map_err(|e| ConfigError::InvalidValue {
                    field: "layout.page_width".into(),
                    reason: e,
                })?;
            }
            if let Some(ref h) = layout.page_height {
                parse_dim(h).map_err(|e| ConfigError::InvalidValue {
                    field: "layout.page_height".into(),
                    reason: e,
                })?;
            }
            if let Some(ref m) = layout.margin {
                parse_dim(m).map_err(|e| ConfigError::InvalidValue {
                    field: "layout.margin".into(),
                    reason: e,
                })?;
            }
            if let Some(ref fs) = layout.font_size {
                parse_dim(fs).map_err(|e| ConfigError::InvalidValue {
                    field: "layout.font_size".into(),
                    reason: e,
                })?;
            }
            if let Some(ref cg) = layout.column_gap {
                parse_dim(cg).map_err(|e| ConfigError::InvalidValue {
                    field: "layout.column_gap".into(),
                    reason: e,
                })?;
            }
            if let Some(lh) = layout.line_height
                && !(0.5..=5.0).contains(&lh)
            {
                return Err(ConfigError::InvalidValue {
                    field: "layout.line_height".into(),
                    reason: "must be between 0.5 and 5.0".into(),
                });
            }
            if let Some(cols) = layout.columns
                && cols == 0
            {
                return Err(ConfigError::InvalidValue {
                    field: "layout.columns".into(),
                    reason: "must be at least 1".into(),
                });
            }
        }
        if let Some(ref citations) = self.citations
            && let Some(ref style) = citations.style
            && !KNOWN_CITATION_STYLES.contains(&style.as_str())
        {
            return Err(ConfigError::InvalidValue {
                field: "citations.style".into(),
                reason: format!(
                    "'{style}'. expected one of: {}",
                    KNOWN_CITATION_STYLES.join(", ")
                ),
            });
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Resolved helpers (sections take precedence over flat fields)
    // -----------------------------------------------------------------------

    fn resolved_format(&self) -> String {
        self.output
            .as_ref()
            .and_then(|o| match o {
                OutputField::Section(s) => s.format.clone(),
                OutputField::Path(_) => None,
            })
            .or_else(|| self.format.clone())
            .unwrap_or_else(|| "pdf".into())
    }

    fn resolved_pdfa_level(&self) -> Option<String> {
        self.output
            .as_ref()
            .and_then(|o| match o {
                OutputField::Section(s) => s.pdfa_level.clone(),
                _ => None,
            })
            .or_else(|| self.pdfa_level.clone())
    }

    fn resolved_pdf_version(&self) -> Option<String> {
        self.output.as_ref().and_then(|o| match o {
            OutputField::Section(s) => s.pdf_version.clone(),
            _ => None,
        })
    }

    // -----------------------------------------------------------------------
    // Merge with CLI args (CLI takes precedence)
    // -----------------------------------------------------------------------

    pub fn merge_with_cli(mut self, cli: &Cli) -> Self {
        // Fold section values into flat fields first (like apply_config_to_cli does)
        if let Some(ref fonts) = self.fonts {
            if self.font.is_none() {
                self.font = fonts.sans.clone();
            }
            if self.font_mono.is_none() {
                self.font_mono = fonts.mono.clone();
            }
        }
        if let Some(ref citations) = self.citations
            && self.bibliography.is_none()
        {
            self.bibliography = citations.bibliography.clone();
        }

        // Now CLI takes precedence
        if cli.format != "pdf" {
            self.set_format(Some(cli.format.clone()));
        }
        if cli.pdfa_level != "4" {
            self.set_pdfa_level(Some(cli.pdfa_level.clone()));
        }
        if cli.font.is_some() {
            self.font = cli.font.clone();
        }
        if cli.font_mono.is_some() {
            self.font_mono = cli.font_mono.clone();
        }
        if cli.font_path.is_some() {
            self.font_path = cli
                .font_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string());
        }
        if cli.title.is_some() {
            self.title = cli.title.clone();
        }
        if cli.author.is_some() {
            self.author = cli.author.clone();
        }
        if cli.subject.is_some() {
            self.subject = cli.subject.clone();
        }
        if (cli.margin - 1.0).abs() > f64::EPSILON {
            self.margin = Some(cli.margin);
        }
        if cli.page_size.is_some() {
            self.page_size = cli.page_size.clone();
        }
        if cli.page_width.is_some() {
            self.page_width = cli.page_width;
        }
        if cli.page_height.is_some() {
            self.page_height = cli.page_height;
        }
        if cli.header_left.is_some() {
            self.header_left = cli.header_left.clone();
        }
        if cli.header_center.is_some() {
            self.header_center = cli.header_center.clone();
        }
        if cli.header_right.is_some() {
            self.header_right = cli.header_right.clone();
        }
        if cli.footer_left.is_some() {
            self.footer_left = cli.footer_left.clone();
        }
        if cli.footer_center.is_some() {
            self.footer_center = cli.footer_center.clone();
        }
        if cli.footer_right.is_some() {
            self.footer_right = cli.footer_right.clone();
        }
        if cli.no_header_rule {
            self.no_header_rule = Some(true);
        }
        if cli.no_footer_rule {
            self.no_footer_rule = Some(true);
        }
        if cli.drop_caps {
            self.drop_caps = Some(true);
        }
        if cli.bibliography.is_some() {
            self.bibliography = cli
                .bibliography
                .as_ref()
                .map(|p| p.to_string_lossy().to_string());
        }
        if cli.ot_features.is_some() {
            self.ot_features = cli.ot_features.clone();
        }
        self
    }

    fn set_format(&mut self, format: Option<String>) {
        match &mut self.output {
            Some(OutputField::Section(s)) => {
                s.format = format;
            }
            _ => {
                self.output = Some(OutputField::Section(OutputConfig {
                    format,
                    pdfa_level: None,
                    pdf_version: None,
                }));
            }
        }
    }

    fn set_pdfa_level(&mut self, level: Option<String>) {
        match &mut self.output {
            Some(OutputField::Section(s)) => {
                s.pdfa_level = level;
            }
            _ => {
                self.output = Some(OutputField::Section(OutputConfig {
                    format: None,
                    pdfa_level: level,
                    pdf_version: None,
                }));
            }
        }
    }

    // -----------------------------------------------------------------------
    // Dump as TOML
    // -----------------------------------------------------------------------

    pub fn dump_toml(&self) -> String {
        let mut out = String::new();

        // [output]
        let format = self.resolved_format();
        let pdfa = self.resolved_pdfa_level();
        let pdf_ver = self.resolved_pdf_version();
        let has_output_section =
            self.output.is_some() || format != "pdf" || pdfa.as_deref() != Some("4");

        if has_output_section {
            out.push_str("[output]\n");
            out.push_str(&format!("format = \"{}\"\n", format));
            if let Some(ref v) = pdfa {
                out.push_str(&format!("pdfa_level = \"{}\"\n", v));
            }
            if let Some(ref v) = pdf_ver {
                out.push_str(&format!("pdf_version = \"{}\"\n", v));
            }
            out.push('\n');
        }

        // [layout]
        if let Some(ref layout) = self.layout {
            out.push_str("[layout]\n");
            if let Some(ref v) = layout.page_width {
                out.push_str(&format!("page_width = \"{}\"\n", v));
            }
            if let Some(ref v) = layout.page_height {
                out.push_str(&format!("page_height = \"{}\"\n", v));
            }
            if let Some(ref v) = layout.margin {
                out.push_str(&format!("margin = \"{}\"\n", v));
            }
            if let Some(ref v) = layout.font_size {
                out.push_str(&format!("font_size = \"{}\"\n", v));
            }
            if let Some(v) = layout.line_height {
                out.push_str(&format!("line_height = {}\n", v));
            }
            if let Some(v) = layout.columns {
                out.push_str(&format!("columns = {}\n", v));
            }
            if let Some(ref v) = layout.column_gap {
                out.push_str(&format!("column_gap = \"{}\"\n", v));
            }
            out.push('\n');
        }

        // [fonts]
        if let Some(ref fonts) = self.fonts {
            out.push_str("[fonts]\n");
            if let Some(ref v) = fonts.serif {
                out.push_str(&format!("serif = \"{}\"\n", v));
            }
            if let Some(ref v) = fonts.sans {
                out.push_str(&format!("sans = \"{}\"\n", v));
            }
            if let Some(ref v) = fonts.mono {
                out.push_str(&format!("mono = \"{}\"\n", v));
            }
            if let Some(ref v) = fonts.math {
                out.push_str(&format!("math = \"{}\"\n", v));
            }
            out.push('\n');
        }

        // [typography]
        if let Some(ref typo) = self.typography {
            out.push_str("[typography]\n");
            if let Some(v) = typo.hyphenation {
                out.push_str(&format!("hyphenation = {}\n", v));
            }
            if let Some(ref v) = typo.hyphenation_lang {
                out.push_str(&format!("hyphenation_lang = \"{}\"\n", v));
            }
            if let Some(v) = typo.ligatures {
                out.push_str(&format!("ligatures = {}\n", v));
            }
            if let Some(v) = typo.kerning {
                out.push_str(&format!("kerning = {}\n", v));
            }
            out.push('\n');
        }

        // [latex]
        if let Some(ref latex) = self.latex
            && let Some(ref macros) = latex.macros
            && !macros.is_empty()
        {
            out.push_str("[latex]\n");
            out.push_str("macros = [");
            for (i, m) in macros.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&format!("\"{}\"", m));
            }
            out.push_str("]\n\n");
        }

        // [citations]
        if let Some(ref cit) = self.citations {
            out.push_str("[citations]\n");
            if let Some(ref v) = cit.bibliography {
                out.push_str(&format!("bibliography = \"{}\"\n", v));
            }
            if let Some(ref v) = cit.style {
                out.push_str(&format!("style = \"{}\"\n", v));
            }
            out.push('\n');
        }

        // Legacy flat fields
        for (key, val) in [
            ("format", self.format.as_deref()),
            ("font", self.font.as_deref()),
            ("font_mono", self.font_mono.as_deref()),
            ("font_path", self.font_path.as_deref()),
            ("title", self.title.as_deref()),
            ("author", self.author.as_deref()),
            ("subject", self.subject.as_deref()),
            ("page_size", self.page_size.as_deref()),
            ("header_left", self.header_left.as_deref()),
            ("header_center", self.header_center.as_deref()),
            ("header_right", self.header_right.as_deref()),
            ("footer_left", self.footer_left.as_deref()),
            ("footer_center", self.footer_center.as_deref()),
            ("footer_right", self.footer_right.as_deref()),
            ("ot_features", self.ot_features.as_deref()),
        ] {
            if let Some(v) = val {
                out.push_str(&format!("{} = \"{}\"\n", key, v));
            }
        }
        if let Some(v) = self.margin {
            out.push_str(&format!("margin = {}\n", v));
        }
        if let Some(v) = self.page_width {
            out.push_str(&format!("page_width = {}\n", v));
        }
        if let Some(v) = self.page_height {
            out.push_str(&format!("page_height = {}\n", v));
        }
        if let Some(v) = self.no_header_rule {
            out.push_str(&format!("no_header_rule = {}\n", v));
        }
        if let Some(v) = self.no_footer_rule {
            out.push_str(&format!("no_footer_rule = {}\n", v));
        }
        if let Some(v) = self.drop_caps {
            out.push_str(&format!("drop_caps = {}\n", v));
        }

        out
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn xdg_config_dir() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        Some(PathBuf::from(xdg))
    } else if let Ok(home) = std::env::var("HOME") {
        Some(PathBuf::from(home).join(".config"))
    } else {
        None
    }
}

fn parse_dim(s: &str) -> std::result::Result<f64, String> {
    let s = s.trim();
    let (num_str, _unit) = if let Some(n) = s.strip_suffix("mm") {
        (n.trim(), "mm")
    } else if let Some(n) = s.strip_suffix("cm") {
        (n.trim(), "cm")
    } else if let Some(n) = s.strip_suffix("in") {
        (n.trim(), "in")
    } else if let Some(n) = s.strip_suffix("pt") {
        (n.trim(), "pt")
    } else {
        (s, "pt")
    };
    let val: f64 = num_str
        .parse()
        .map_err(|_| format!("cannot parse dimension '{}'", s))?;
    if val <= 0.0 {
        return Err(format!("must be positive, got '{}'", s));
    }
    Ok(val)
}

// ---------------------------------------------------------------------------
// Backward-compatible API
// ---------------------------------------------------------------------------

/// Load config from explicit path (backward compat wrapper).
/// Use `LdirConfig::load()` for the full API including `--no-config`.
pub fn load_config(path: Option<&Path>) -> Result<LdirConfig> {
    LdirConfig::load(path, false)
}

/// Apply config values to CLI struct. Config values serve as defaults;
/// CLI flags always take precedence.
pub fn apply_config_to_cli(config: &LdirConfig, cli: &mut Cli) {
    // Resolve effective values (sections > flat fields)
    let effective_format = config
        .output
        .as_ref()
        .and_then(|o| match o {
            OutputField::Section(s) => s.format.clone(),
            OutputField::Path(_) => None,
        })
        .or_else(|| config.format.clone());

    let effective_pdfa = config
        .output
        .as_ref()
        .and_then(|o| match o {
            OutputField::Section(s) => s.pdfa_level.clone(),
            _ => None,
        })
        .or_else(|| config.pdfa_level.clone());

    // Handle output path from legacy `output = "path"` string
    if let Some(OutputField::Path(path)) = &config.output
        && cli.output.is_none()
    {
        cli.output = Some(PathBuf::from(path));
    }

    // Apply effective values (CLI defaults take lower precedence)
    if cli.format == "pdf"
        && let Some(f) = effective_format
    {
        cli.format = f;
    }

    if cli.pdfa_level == "4"
        && let Some(level) = effective_pdfa
    {
        cli.pdfa_level = level;
    }

    // [fonts] section -> CLI fields
    if let Some(ref fonts) = config.fonts {
        if cli.font.is_none() {
            cli.font = fonts.sans.clone();
        }
        if cli.font_mono.is_none() {
            cli.font_mono = fonts.mono.clone();
        }
    }

    // [citations] section -> CLI fields
    if let Some(ref citations) = config.citations
        && cli.bibliography.is_none()
    {
        cli.bibliography = citations.bibliography.as_ref().map(PathBuf::from);
    }

    // Legacy flat fields (CLI takes precedence)
    if cli.font.is_none() {
        cli.font = config.font.clone();
    }
    if cli.font_mono.is_none() {
        cli.font_mono = config.font_mono.clone();
    }
    if cli.font_path.is_none() {
        cli.font_path = config.font_path.as_ref().map(PathBuf::from);
    }
    if cli.title.is_none() {
        cli.title = config.title.clone();
    }
    if cli.author.is_none() {
        cli.author = config.author.clone();
    }
    if cli.subject.is_none() {
        cli.subject = config.subject.clone();
    }
    if (cli.margin - 1.0).abs() < f64::EPSILON
        && let Some(v) = config.margin
    {
        cli.margin = v;
    }
    if cli.page_size.is_none() {
        cli.page_size = config.page_size.clone();
    }
    if cli.page_width.is_none() {
        cli.page_width = config.page_width;
    }
    if cli.page_height.is_none() {
        cli.page_height = config.page_height;
    }
    if cli.header_left.is_none() {
        cli.header_left = config.header_left.clone();
    }
    if cli.header_center.is_none() {
        cli.header_center = config.header_center.clone();
    }
    if cli.header_right.is_none() {
        cli.header_right = config.header_right.clone();
    }
    if cli.footer_left.is_none() {
        cli.footer_left = config.footer_left.clone();
    }
    if cli.footer_center.is_none() {
        cli.footer_center = config.footer_center.clone();
    }
    if cli.footer_right.is_none() {
        cli.footer_right = config.footer_right.clone();
    }
    if !cli.no_header_rule {
        cli.no_header_rule = config.no_header_rule.unwrap_or(false);
    }
    if !cli.no_footer_rule {
        cli.no_footer_rule = config.no_footer_rule.unwrap_or(false);
    }
    if !cli.drop_caps {
        cli.drop_caps = config.drop_caps.unwrap_or(false);
    }
    if cli.bibliography.is_none() {
        cli.bibliography = config.bibliography.as_ref().map(PathBuf::from);
    }
    if cli.pdfa_level == "4"
        && let Some(ref v) = config.pdfa_level
    {
        cli.pdfa_level = v.clone();
    }
    if cli.ot_features.is_none() {
        cli.ot_features = config.ot_features.clone();
    }
}

/// Print the effective (merged) configuration as TOML.
pub fn dump_effective_config(cli: &Cli) -> String {
    let config = LdirConfig {
        output: Some(OutputField::Section(OutputConfig {
            format: Some(cli.format.clone()),
            pdfa_level: Some(cli.pdfa_level.clone()),
            pdf_version: None,
        })),
        font: cli.font.clone(),
        font_mono: cli.font_mono.clone(),
        font_path: cli
            .font_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string()),
        title: cli.title.clone(),
        author: cli.author.clone(),
        subject: cli.subject.clone(),
        margin: if (cli.margin - 1.0).abs() > f64::EPSILON {
            Some(cli.margin)
        } else {
            None
        },
        page_size: cli.page_size.clone(),
        page_width: cli.page_width,
        page_height: cli.page_height,
        header_left: cli.header_left.clone(),
        header_center: cli.header_center.clone(),
        header_right: cli.header_right.clone(),
        footer_left: cli.footer_left.clone(),
        footer_center: cli.footer_center.clone(),
        footer_right: cli.footer_right.clone(),
        no_header_rule: if cli.no_header_rule { Some(true) } else { None },
        no_footer_rule: if cli.no_footer_rule { Some(true) } else { None },
        drop_caps: if cli.drop_caps { Some(true) } else { None },
        bibliography: cli
            .bibliography
            .as_ref()
            .map(|p| p.to_string_lossy().to_string()),
        ot_features: cli.ot_features.clone(),
        ..Default::default()
    };
    config.dump_toml()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    // -- test_config_loading --

    #[test]
    fn test_config_loading_structured() {
        let toml = r#"
[output]
format = "html"
pdfa_level = "2b"
pdf_version = "1.7"

[layout]
page_width = "210mm"
page_height = "297mm"
margin = "25mm"
font_size = "11pt"
line_height = 1.5
columns = 2
column_gap = "15pt"

[fonts]
serif = "DejaVu Serif"
sans = "DejaVu Sans"
mono = "DejaVu Sans Mono"
math = "DejaVu Sans"

[typography]
hyphenation = true
hyphenation_lang = "en"
ligatures = true
kerning = true

[latex]
macros = ["macros.tex", "custom.tex"]

[citations]
bibliography = "refs.bib"
style = "ieee"
"#;
        let config: LdirConfig = toml::from_str(toml).unwrap();

        // Output section
        match &config.output {
            Some(OutputField::Section(o)) => {
                assert_eq!(o.format.as_deref(), Some("html"));
                assert_eq!(o.pdfa_level.as_deref(), Some("2b"));
                assert_eq!(o.pdf_version.as_deref(), Some("1.7"));
            }
            other => panic!("expected OutputField::Section, got {:?}", other),
        }

        // Layout
        let layout = config.layout.as_ref().unwrap();
        assert_eq!(layout.page_width.as_deref(), Some("210mm"));
        assert_eq!(layout.page_height.as_deref(), Some("297mm"));
        assert_eq!(layout.margin.as_deref(), Some("25mm"));
        assert_eq!(layout.font_size.as_deref(), Some("11pt"));
        assert_eq!(layout.line_height, Some(1.5));
        assert_eq!(layout.columns, Some(2));
        assert_eq!(layout.column_gap.as_deref(), Some("15pt"));

        // Fonts
        let fonts = config.fonts.as_ref().unwrap();
        assert_eq!(fonts.serif.as_deref(), Some("DejaVu Serif"));
        assert_eq!(fonts.sans.as_deref(), Some("DejaVu Sans"));
        assert_eq!(fonts.mono.as_deref(), Some("DejaVu Sans Mono"));

        // Typography
        let typo = config.typography.as_ref().unwrap();
        assert_eq!(typo.hyphenation, Some(true));
        assert_eq!(typo.hyphenation_lang.as_deref(), Some("en"));
        assert_eq!(typo.ligatures, Some(true));
        assert_eq!(typo.kerning, Some(true));

        // Latex
        let latex = config.latex.as_ref().unwrap();
        let macros = latex.macros.as_ref().unwrap();
        assert_eq!(macros.len(), 2);
        assert_eq!(macros[0], "macros.tex");
        assert_eq!(macros[1], "custom.tex");

        // Citations
        let cit = config.citations.as_ref().unwrap();
        assert_eq!(cit.bibliography.as_deref(), Some("refs.bib"));
        assert_eq!(cit.style.as_deref(), Some("ieee"));
    }

    #[test]
    fn test_config_loading_legacy_output_path() {
        let toml = r#"
output = "result.pdf"
format = "html"
"#;
        let config: LdirConfig = toml::from_str(toml).unwrap();
        match &config.output {
            Some(OutputField::Path(p)) => assert_eq!(p, "result.pdf"),
            other => panic!("expected OutputField::Path, got {:?}", other),
        }
        assert_eq!(config.format.as_deref(), Some("html"));
    }

    #[test]
    fn test_config_loading_flat() {
        let toml = r#"
format = "html"
font = "Noto Serif"
margin = 0.5
title = "Test Doc"
footer_right = "%page"
"#;
        let config: LdirConfig = toml::from_str(toml).unwrap();
        assert!(config.output.is_none());
        assert_eq!(config.format.as_deref(), Some("html"));
        assert_eq!(config.font.as_deref(), Some("Noto Serif"));
        assert_eq!(config.margin, Some(0.5));
        assert_eq!(config.title.as_deref(), Some("Test Doc"));
        assert_eq!(config.footer_right.as_deref(), Some("%page"));
        assert!(config.author.is_none());
    }

    // -- test_config_merge --

    #[test]
    fn test_config_merge() {
        let toml = r#"
[output]
format = "html"
pdfa_level = "2b"

[fonts]
sans = "Config Sans"
mono = "Config Mono"
"#;
        let config: LdirConfig = toml::from_str(toml).unwrap();
        let cli = Cli::try_parse_from(["ldc", "input.md", "--format", "epub", "--font", "CliFont"])
            .unwrap();

        let merged = config.merge_with_cli(&cli);

        // CLI overrides config format
        assert_eq!(merged.resolved_format(), "epub");
        // CLI overrides font
        assert_eq!(merged.font.as_deref(), Some("CliFont"));
        // Config mono preserved (CLI didn't override)
        assert_eq!(merged.font_mono.as_deref(), Some("Config Mono"));
        // Config pdfa preserved (CLI didn't override)
        assert_eq!(merged.resolved_pdfa_level().as_deref(), Some("2b"));
    }

    #[test]
    fn test_config_merge_cli_overrides_all() {
        let toml = r#"
format = "html"
font = "ConfigFont"
margin = 0.5
title = "Config Title"
no_header_rule = true
"#;
        let config: LdirConfig = toml::from_str(toml).unwrap();
        let cli = Cli::try_parse_from([
            "ldc",
            "input.md",
            "--format",
            "epub",
            "--font",
            "CliFont",
            "--margin",
            "2.0",
            "--title",
            "Cli Title",
            "--no-header-rule",
        ])
        .unwrap();

        let merged = config.merge_with_cli(&cli);
        assert_eq!(merged.resolved_format(), "epub");
        assert_eq!(merged.font.as_deref(), Some("CliFont"));
        assert_eq!(merged.margin, Some(2.0));
        assert_eq!(merged.title.as_deref(), Some("Cli Title"));
        assert_eq!(merged.no_header_rule, Some(true));
    }

    // -- test_config_defaults --

    #[test]
    fn test_config_defaults() {
        let config = LdirConfig::default();
        assert!(config.output.is_none());
        assert!(config.layout.is_none());
        assert!(config.fonts.is_none());
        assert!(config.typography.is_none());
        assert!(config.latex.is_none());
        assert!(config.citations.is_none());
        assert!(config.format.is_none());
        assert!(config.font.is_none());
        assert!(config.margin.is_none());
        assert_eq!(config.resolved_format(), "pdf");
        assert!(config.resolved_pdfa_level().is_none());
    }

    #[test]
    fn empty_config_uses_defaults() {
        let config: LdirConfig = toml::from_str("").unwrap();
        assert!(config.format.is_none());
        assert!(config.margin.is_none());
        assert!(config.output.is_none());
    }

    // -- test_config_validation --

    #[test]
    fn test_config_validation_ok() {
        let toml = r#"
[output]
format = "pdf"
pdfa_level = "2b"
pdf_version = "1.7"

[layout]
page_width = "210mm"
page_height = "297mm"
margin = "25mm"
line_height = 1.5
columns = 2

[fonts]
serif = "DejaVu Serif"
sans = "DejaVu Sans"
mono = "DejaVu Mono"
math = "DejaVu Math"

[citations]
style = "ieee"
"#;
        let config: LdirConfig = toml::from_str(toml).unwrap();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_unknown_format() {
        let toml = r#"
[output]
format = "xyz"
"#;
        let config: LdirConfig = toml::from_str(toml).unwrap();
        let err = config.validate().unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("unknown format"), "got: {msg}");
    }

    #[test]
    fn test_config_validation_invalid_pdfa() {
        let toml = r#"
[output]
pdfa_level = "99z"
"#;
        let config: LdirConfig = toml::from_str(toml).unwrap();
        let err = config.validate().unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("output.pdfa_level"), "got: {msg}");
    }

    #[test]
    fn test_config_validation_negative_dimension() {
        let toml = r#"
[layout]
page_width = "-10mm"
"#;
        let config: LdirConfig = toml::from_str(toml).unwrap();
        let err = config.validate().unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("layout.page_width"), "got: {msg}");
    }

    #[test]
    fn test_config_validation_empty_font() {
        let toml = r#"
[fonts]
serif = "  "
sans = ""
"#;
        let config: LdirConfig = toml::from_str(toml).unwrap();
        let err = config.validate().unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("empty font name"), "got: {msg}");
    }

    #[test]
    fn test_config_validation_zero_columns() {
        let toml = r#"
[layout]
columns = 0
"#;
        let config: LdirConfig = toml::from_str(toml).unwrap();
        let err = config.validate().unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("layout.columns"), "got: {msg}");
    }

    #[test]
    fn test_config_validation_bad_citation_style() {
        let toml = r#"
[citations]
style = "unknown_style"
"#;
        let config: LdirConfig = toml::from_str(toml).unwrap();
        let err = config.validate().unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("citations.style"), "got: {msg}");
    }

    #[test]
    fn test_config_validation_line_height_out_of_range() {
        let toml = r#"
[layout]
line_height = 0.1
"#;
        let config: LdirConfig = toml::from_str(toml).unwrap();
        let err = config.validate().unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("layout.line_height"), "got: {msg}");
    }

    #[test]
    fn test_dump_toml_roundtrip() {
        let toml = r#"
[output]
format = "html"
pdfa_level = "2b"
pdf_version = "1.7"

[layout]
page_width = "210mm"
page_height = "297mm"
margin = "25mm"

[fonts]
serif = "DejaVu Serif"
sans = "DejaVu Sans"
"#;
        let config: LdirConfig = toml::from_str(toml).unwrap();
        let dumped = config.dump_toml();
        assert!(dumped.contains("[output]"), "missing [output] section");
        assert!(dumped.contains("format = \"html\""), "missing format");
        assert!(dumped.contains("pdfa_level = \"2b\""), "missing pdfa_level");
        assert!(dumped.contains("[layout]"), "missing [layout] section");
        assert!(
            dumped.contains("page_width = \"210mm\""),
            "missing page_width"
        );
        assert!(dumped.contains("[fonts]"), "missing [fonts] section");
        assert!(dumped.contains("serif = \"DejaVu Serif\""), "missing serif");
    }

    #[test]
    fn test_load_no_config_returns_defaults() {
        let config = LdirConfig::load(None, true).unwrap();
        assert!(config.output.is_none());
        assert!(config.format.is_none());
    }
}
