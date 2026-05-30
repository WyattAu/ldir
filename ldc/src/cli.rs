//! CLI argument definitions for ldc.

use std::path::PathBuf;

use clap::Parser;

/// LDIR Compiler — compile documents to PDF and other formats.
///
/// Input formats: .md, .tex, .typ, .html, .htm, .adoc, .org, .docx (auto-detected)
/// Output formats: .pdf, .html, .epub, .txt, .docx, .odt, .sir2, .ldir (--format or auto-detected)
#[derive(Parser, Debug)]
#[command(name = "ldc", version, about)]
pub struct Cli {
    /// Input file(s). Multiple files are merged with offset IDs.
    /// Supported: .md, .tex, .typ, .html, .htm, .adoc, .org, .docx
    #[arg(value_name = "INPUTS")]
    pub inputs: Vec<PathBuf>,

    /// Output file path. Defaults to first input stem + extension based on format.
    #[arg(short, long, value_name = "OUTPUT")]
    pub output: Option<PathBuf>,

    /// Output format.
    #[arg(short = 'f', long, value_name = "FORMAT", default_value = "pdf",
        value_parser = ["pdf", "gir", "sir", "html", "epub", "txt", "docx", "odt", "sir2", "ldir", "pandoc", "ipynb"])]
    pub format: String,

    /// Primary font family name (e.g., "DejaVu Sans", "Noto Serif").
    /// Auto-detected from system fonts if not specified.
    #[arg(long, value_name = "FONT_FAMILY")]
    pub font: Option<String>,

    /// Monospace font family name (e.g., "DejaVu Sans Mono").
    /// Auto-detected from system fonts if not specified.
    #[arg(long, value_name = "FONT_FAMILY")]
    pub font_mono: Option<String>,

    /// Path to primary font file (.ttf/.otf).
    /// Overrides --font when specified.
    #[arg(long, value_name = "PATH")]
    pub font_path: Option<PathBuf>,

    /// List available system fonts and exit.
    #[arg(long)]
    pub list_fonts: bool,

    /// Document title for PDF metadata.
    #[arg(long, value_name = "TITLE")]
    pub title: Option<String>,

    /// Document author for PDF metadata.
    #[arg(long, value_name = "AUTHOR")]
    pub author: Option<String>,

    /// Document subject for PDF metadata.
    #[arg(long, value_name = "SUBJECT")]
    pub subject: Option<String>,

    /// Page margin in inches (applied uniformly to all sides).
    #[arg(long, value_name = "INCHES", default_value_t = 1.0)]
    pub margin: f64,

    /// Page size preset ("a4", "letter", "legal").
    #[arg(long, value_name = "SIZE")]
    pub page_size: Option<String>,

    /// Custom page width in points (overrides --page-size).
    #[arg(long, value_name = "WIDTH_PT")]
    pub page_width: Option<f64>,

    /// Custom page height in points (overrides --page-size).
    #[arg(long, value_name = "HEIGHT_PT")]
    pub page_height: Option<f64>,

    /// Header left template (supports %page, %pages, %title, %author, %date).
    #[arg(long, value_name = "TEMPLATE")]
    pub header_left: Option<String>,

    /// Header center template.
    #[arg(long, value_name = "TEMPLATE")]
    pub header_center: Option<String>,

    /// Header right template.
    #[arg(long, value_name = "TEMPLATE")]
    pub header_right: Option<String>,

    /// Footer left template.
    #[arg(long, value_name = "TEMPLATE")]
    pub footer_left: Option<String>,

    /// Footer center template.
    #[arg(long, value_name = "TEMPLATE")]
    pub footer_center: Option<String>,

    /// Footer right template (default: %page).
    #[arg(long, value_name = "TEMPLATE")]
    pub footer_right: Option<String>,

    /// Disable header rule line.
    #[arg(long)]
    pub no_header_rule: bool,

    /// Disable footer rule line.
    #[arg(long)]
    pub no_footer_rule: bool,

    /// Enable drop caps for the first paragraph after headings.
    #[arg(long, default_value_t = false)]
    pub drop_caps: bool,

    /// Path to BibTeX (.bib) file for citations.
    #[arg(long, value_name = "PATH")]
    pub bibliography: Option<PathBuf>,

    /// Use the L-IR layout pipeline (S-IR → L-IR → G-IR) instead of direct compilation.
    #[arg(long)]
    pub lir: bool,

    /// PDF/A conformance level ("4" for PDF/A-4, "2b" for PDF/A-2b).
    #[arg(long, value_name = "LEVEL", default_value = "4")]
    pub pdfa_level: String,

    /// Color output. Options: auto, always, never. Default: auto.
    #[arg(long, value_name = "WHEN", default_value = "auto")]
    pub color: String,

    /// OpenType features (e.g., "kern,liga,dlig,hlig").
    /// Prefix with - to disable: "kern,-liga". Default: HarfBuzz defaults.
    #[arg(long, value_name = "FEATURES")]
    pub ot_features: Option<String>,

    /// Path to configuration file. Default: ./ldir.toml
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Skip loading any configuration file.
    #[arg(long)]
    pub no_config: bool,

    /// Print the effective (merged) configuration as TOML and exit.
    #[arg(long)]
    pub dump_config: bool,
}
