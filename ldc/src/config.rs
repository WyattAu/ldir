//! Configuration file support for ldc.
//!
//! Reads `ldir.toml` from the current directory (or `--config PATH`).
//! All fields optional; defaults come from CLI defaults.
//! Precedence: CLI flags > config file > defaults.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::cli::Cli;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct LdirConfig {
    pub output: Option<String>,
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

pub fn load_config(path: Option<&Path>) -> Result<LdirConfig> {
    let config_path = match path {
        Some(p) => p.to_path_buf(),
        None => PathBuf::from("ldir.toml"),
    };

    if !config_path.exists() {
        return Ok(LdirConfig::default());
    }

    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("failed to read config: {}", config_path.display()))?;

    toml::from_str(&content)
        .with_context(|| format!("failed to parse config: {}", config_path.display()))
}

pub fn apply_config_to_cli(config: &LdirConfig, cli: &mut Cli) {
    if cli.output.is_none() {
        cli.output = config.output.as_ref().map(PathBuf::from);
    }
    if cli.format == "pdf"
        && let Some(ref v) = config.format
    {
        cli.format = v.clone();
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_flat_config() {
        let toml = r#"
format = "html"
font = "Noto Serif"
margin = 0.5
title = "Test Doc"
footer_right = "%page"
"#;
        let config: LdirConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.format.as_deref(), Some("html"));
        assert_eq!(config.font.as_deref(), Some("Noto Serif"));
        assert_eq!(config.margin, Some(0.5));
        assert_eq!(config.title.as_deref(), Some("Test Doc"));
        assert_eq!(config.footer_right.as_deref(), Some("%page"));
        assert!(config.output.is_none());
        assert!(config.author.is_none());
    }

    #[test]
    fn empty_config_uses_defaults() {
        let config: LdirConfig = toml::from_str("").unwrap();
        assert!(config.format.is_none());
        assert!(config.margin.is_none());
    }
}
