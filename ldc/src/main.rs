//! ldc — LDIR Compiler CLI.
//!
//! Compiles documents in multiple formats to PDF, HTML, EPUB, TXT, DOCX, SIR2, or LDIR.
//!
//! # Usage
//!
//! ```sh
//! ldc input.md -o output.pdf
//! ldc input.typ -f html -o output.html
//! ldc input.adoc -f epub -o output.epub
//! ldc input.org -f txt  -o output.txt
//! ldc input.docx -f docx -o output.docx
//! ldc input.html -f sir2 -o output.sir2
//! ldc input.md input.typ -o output.html
//! ```

mod cli;
mod config;
mod status;

use std::fs::File;
use std::io::{BufWriter, IsTerminal};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use ldir_core::error::LdirError;
use ldir_core::font::db::FontDatabase;
use ldir_ir::gir::GIRDocument;
use ldir_ir::sir::v2::SIRModuleV2;

use cli::Cli;
use status::{Color, PipelineTimer, styled};

const V2_FORMATS: &[&str] = &["html", "epub", "txt", "docx", "sir2", "ldir"];

fn detect_input_format(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "md" | "markdown" => "markdown",
        "tex" | "latex" => "latex",
        "typ" | "typst" => "typst",
        "html" | "htm" => "html",
        "adoc" | "asciidoc" => "asciidoc",
        "org" => "org",
        "docx" => "docx",
        _ => "unknown",
    }
}

fn detect_format_from_extension(ext: &str) -> &str {
    match ext {
        "html" | "htm" => "html",
        "epub" => "epub",
        "txt" => "txt",
        "docx" => "docx",
        "sir2" => "sir2",
        "ldir" => "ldir",
        _ => "pdf",
    }
}

fn parse_to_sir_v2(text: &str, format: &str) -> SIRModuleV2 {
    match format {
        "typst" => ldir_typst::parse_typst(text),
        "html" => ldir_html_reader::parse_html(text),
        "asciidoc" => ldir_adoc::parse_asciidoc(text),
        "org" => ldir_org::parse_org(text),
        "markdown" => {
            let v1 = ldir_md::parse_markdown(text);
            ldir_core::compiler::v1_to_v2::convert_v1_to_v2(&v1)
        }
        "latex" => {
            let v1 = ldir_tex::parse_tex(text);
            ldir_core::compiler::v1_to_v2::convert_v1_to_v2(&v1)
        }
        _ => SIRModuleV2::new(),
    }
}

fn parse_docx_to_sir_v2(bytes: &[u8]) -> SIRModuleV2 {
    ldir_docx_reader::parse_docx(bytes)
}

fn parse_input_to_sir_v2(path: &Path) -> Result<SIRModuleV2> {
    let format = detect_input_format(path);
    if format == "unknown" {
        anyhow::bail!(
            "unsupported input format for '{}'. Supported: .md, .tex, .typ, .html, .htm, .adoc, .org, .docx",
            path.display()
        );
    }
    if format == "docx" {
        let bytes =
            std::fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
        let mut module = parse_docx_to_sir_v2(&bytes);
        module.header.source_path = Some(path.display().to_string());
        Ok(module)
    } else {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let mut module = parse_to_sir_v2(&text, format);
        module.header.source_path = Some(path.display().to_string());
        module.header.source_format = Some(format.to_string());
        Ok(module)
    }
}

fn merge_modules(inputs: &[PathBuf]) -> Result<SIRModuleV2> {
    let mut merged = SIRModuleV2::new();
    let mut id_offset: u32 = 0;
    for input in inputs {
        let mut module = parse_input_to_sir_v2(input)
            .with_context(|| format!("failed to parse {}", input.display()))?;
        let dim = styled("parse", Color::Dim);
        eprintln!("  {dim} {} -> {} nodes", input.display(), module.body.len());
        for node in module.body.iter_mut() {
            node.id += id_offset;
            node.parent_id = node.parent_id.map(|p| p + id_offset);
            node.child_ids = node.child_ids.iter().map(|c| c + id_offset).collect();
        }
        let count = module.body.len() as u32;
        for node in module.body.iter() {
            merged.body.push(node.clone());
        }
        id_offset += count;
    }
    Ok(merged)
}

fn write_output(module: &SIRModuleV2, output_path: &Path, format: &str) -> Result<()> {
    match format {
        "html" => {
            let html = ldir_html::HtmlRenderer::new().render(module);
            let tag = styled("html", Color::Green);
            eprintln!("  {tag} {} bytes", html.len());
            std::fs::write(output_path, &html)
                .with_context(|| format!("failed to write {}", output_path.display()))?;
        }
        "epub" => {
            let epub_bytes = ldir_epub::EpubBuilder::new()
                .build(module)
                .map_err(|e| anyhow::anyhow!(e))?;
            let tag = styled("epub", Color::Green);
            eprintln!("  {tag} {} bytes", epub_bytes.len());
            std::fs::write(output_path, &epub_bytes)
                .with_context(|| format!("failed to write {}", output_path.display()))?;
        }
        "txt" => {
            let m = module.clone();
            let text = ldir_txt::TextRenderer::new().render(&m);
            let tag = styled("txt", Color::Green);
            eprintln!("  {tag} {} bytes", text.len());
            std::fs::write(output_path, &text)
                .with_context(|| format!("failed to write {}", output_path.display()))?;
        }
        "docx" => {
            let docx_bytes = ldir_docx::DocxBuilder::new()
                .build(module)
                .map_err(|e| anyhow::anyhow!(e))?;
            let tag = styled("docx", Color::Green);
            eprintln!("  {tag} {} bytes", docx_bytes.len());
            std::fs::write(output_path, &docx_bytes)
                .with_context(|| format!("failed to write {}", output_path.display()))?;
        }
        "sir2" => {
            let bytes = ldir_ir::sir::v2::SIRBinaryWriter::write(module);
            let tag = styled("sir2", Color::Green);
            eprintln!("  {tag} {} bytes -> {}", bytes.len(), output_path.display());
            std::fs::write(output_path, &bytes)
                .with_context(|| format!("failed to write {}", output_path.display()))?;
        }
        "ldir" => {
            let text = ldir_ir::sir::v2::module_to_text(module);
            let tag = styled("ldir", Color::Green);
            eprintln!("  {tag} {} bytes -> {}", text.len(), output_path.display());
            std::fs::write(output_path, &text)
                .with_context(|| format!("failed to write {}", output_path.display()))?;
        }
        _ => anyhow::bail!("unsupported output format: {}", format),
    }
    let wrote = styled("wrote", Color::Green);
    eprintln!("  {wrote} {}", output_path.display());
    Ok(())
}

/// Try to load a font variant by searching common paths.
fn find_font_variant(base_path: &str, variant_names: &[&str]) -> Option<Vec<u8>> {
    if let Some(dir) = std::path::Path::new(base_path).parent() {
        for name in variant_names {
            let candidate = dir.join(name);
            if let Ok(data) = std::fs::read(&candidate)
                && ttf_parser::Face::parse(&data, 0).is_ok()
            {
                let dim = styled("variant", Color::Dim);
                eprintln!("  {dim} {}", candidate.display());
                return Some(data);
            }
        }
    }
    None
}

struct FontVariant {
    id: u32,
    search_names: Vec<&'static str>,
}

fn load_font_variants(primary_path: &str) -> Vec<(u32, Vec<u8>)> {
    let base_name = std::path::Path::new(primary_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    let is_dejavu = base_name.contains("DejaVu");
    let is_liberation = base_name.contains("Liberation");
    let is_noto = base_name.contains("Noto");

    let variants: Vec<FontVariant> = vec![
        FontVariant {
            id: 0,
            search_names: vec![],
        },
        FontVariant {
            id: 1,
            search_names: if is_dejavu {
                vec!["DejaVuSans-Bold.ttf"]
            } else if is_liberation {
                vec!["LiberationSans-Bold.ttf"]
            } else if is_noto {
                vec!["NotoSans-Bold.ttf"]
            } else {
                vec!["-Bold.ttf", "-Bold.otf"]
            },
        },
        FontVariant {
            id: 2,
            search_names: if is_dejavu {
                vec!["DejaVuSans-Oblique.ttf"]
            } else if is_liberation {
                vec!["LiberationSans-Italic.ttf"]
            } else if is_noto {
                vec!["NotoSans-Italic.ttf"]
            } else {
                vec!["-Italic.ttf", "-Oblique.ttf", "-Italic.otf"]
            },
        },
        FontVariant {
            id: 3,
            search_names: if is_dejavu {
                vec!["DejaVuSans-BoldOblique.ttf"]
            } else if is_liberation {
                vec!["LiberationSans-BoldItalic.ttf"]
            } else if is_noto {
                vec!["NotoSans-BoldItalic.ttf"]
            } else {
                vec!["-BoldItalic.ttf", "-BoldOblique.ttf", "-BoldItalic.otf"]
            },
        },
        FontVariant {
            id: 4,
            search_names: if is_dejavu {
                vec!["DejaVuSansMono.ttf", "../DejaVuSansMono/DejaVuSansMono.ttf"]
            } else if is_liberation {
                vec!["LiberationMono-Regular.ttf"]
            } else if is_noto {
                vec!["NotoSansMono-Regular.ttf"]
            } else {
                vec!["Mono-Regular.ttf", "-Mono.ttf", "monospace.ttf"]
            },
        },
    ];

    let mut loaded = Vec::new();
    for variant in &variants {
        if variant.id == 0 {
            continue;
        }
        if let Some(data) = find_font_variant(primary_path, &variant.search_names) {
            loaded.push((variant.id, data));
        }
    }
    loaded
}

/// Resolve font data: first by family name via database, then by file path,
/// then by scanning common system font directories.
fn resolve_font(cli: &Cli, font_db: &FontDatabase) -> Option<(Arc<Vec<u8>>, String)> {
    if let Some(ref path) = cli.font_path {
        let data = std::fs::read(path)
            .with_context(|| format!("failed to read font: {}", path.display()))
            .ok()?;
        ttf_parser::Face::parse(&data, 0)
            .with_context(|| format!("invalid font file: {}", path.display()))
            .ok()?;
        let tag = styled("font", Color::Green);
        eprintln!("  {tag} {}", path.display());
        let path_str = path.to_string_lossy().to_string();
        return Some((Arc::new(data), path_str));
    }

    if let Some(ref family) = cli.font
        && let Some(id) = font_db.query(family)
        && let Some(data) = font_db.face_data(id)
    {
        let tag = styled("font", Color::Green);
        eprintln!("  {tag} {} (system)", family);
        return Some((data, format!("system:{}", family)));
    }

    if let Some(ref family) = cli.font {
        let warn = styled("warn", Color::Yellow);
        eprintln!("  {warn}: font family '{}' not found", family);
    }

    let candidates = [
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/LiberationSans-Regular.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/opentype/noto/NotoSans-Regular.otf",
        "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
    ];
    for path in &candidates {
        if let Ok(data) = std::fs::read(path)
            && ttf_parser::Face::parse(&data, 0).is_ok()
        {
            let tag = styled("font", Color::Green);
            eprintln!("  {tag} {}", path);
            return Some((Arc::new(data), path.to_string()));
        }
    }

    if let Some(id) = font_db.query("DejaVu Sans")
        && let Some(data) = font_db.face_data(id)
    {
        let tag = styled("font", Color::Green);
        eprintln!("  {tag} DejaVu Sans (auto)");
        return Some((data, "system:DejaVu Sans".to_string()));
    }

    None
}

/// Load font variant data using the font database for style variants.
fn load_font_variants_from_db(
    cli: &Cli,
    font_db: &FontDatabase,
    primary_family: &str,
) -> Vec<(u32, Arc<Vec<u8>>)> {
    let mut variants = Vec::new();

    let family = cli.font.as_deref().unwrap_or(primary_family);

    let bold_styles = [
        (fontdb::Weight::BOLD, fontdb::Style::Normal),
        (fontdb::Weight::BOLD, fontdb::Style::Italic),
    ];
    if let Some((_, data)) = try_query_style(font_db, family, &bold_styles) {
        let tag = styled("font", Color::Dim);
        eprintln!("  {tag} bold variant loaded");
        variants.push((1, data));
    }

    let italic_styles = [
        (fontdb::Weight::NORMAL, fontdb::Style::Italic),
        (fontdb::Weight::NORMAL, fontdb::Style::Oblique),
    ];
    if let Some((_, data)) = try_query_style(font_db, family, &italic_styles) {
        let tag = styled("font", Color::Dim);
        eprintln!("  {tag} italic variant loaded");
        variants.push((2, data));
    }

    let bold_italic_styles = [
        (fontdb::Weight::BOLD, fontdb::Style::Italic),
        (fontdb::Weight::BOLD, fontdb::Style::Oblique),
    ];
    if let Some((_, data)) = try_query_style(font_db, family, &bold_italic_styles) {
        let tag = styled("font", Color::Dim);
        eprintln!("  {tag} bold-italic variant loaded");
        variants.push((3, data));
    }

    let mono_family = cli.font_mono.as_deref().unwrap_or("");
    let mono_id = if !mono_family.is_empty() {
        font_db.query(mono_family)
    } else {
        font_db.query_monospace()
    };
    if let Some(id) = mono_id
        && let Some(data) = font_db.face_data(id)
    {
        let tag = styled("font", Color::Dim);
        eprintln!("  {tag} monospace variant loaded");
        variants.push((4, data));
    }

    variants
}

fn try_query_style(
    font_db: &FontDatabase,
    family: &str,
    styles: &[(fontdb::Weight, fontdb::Style)],
) -> Option<(fontdb::ID, Arc<Vec<u8>>)> {
    for &(weight, style) in styles {
        if let Some(id) = font_db.query_family_style(family, weight, style)
            && let Some(data) = font_db.face_data(id)
        {
            return Some((id, data));
        }
    }
    None
}

fn list_system_fonts(font_db: &FontDatabase) {
    let mut families: Vec<(String, Vec<String>)> = Vec::new();

    for face in font_db.face_info_iter() {
        if let Some(name_tuple) = face.families.first() {
            let name_str = &name_tuple.0;
            if let Some(entry) = families.iter_mut().find(|(f, _)| f == name_str) {
                entry.1.push(format!("{:?}", face.style));
            } else {
                families.push((name_str.clone(), vec![format!("{:?}", face.style)]));
            }
        }
    }

    families.sort_by(|a, b| a.0.cmp(&b.0));

    eprintln!(
        "Available system fonts ({} faces, {} families):",
        font_db.face_count(),
        families.len()
    );
    for (i, (family, styles)) in families.iter().enumerate() {
        let styles_str = styles.join(", ");
        eprintln!("  {:>3}. {} ({})", i + 1, family, styles_str);
    }
}

fn emit_pdf(
    gir_doc: &GIRDocument,
    cli: &Cli,
    font_data: &Option<Arc<Vec<u8>>>,
    variant_fonts: &[(u32, Arc<Vec<u8>>)],
    output: &Path,
) -> Result<()> {
    let conformance: ldir_pdf::conformance::PdfConformance =
        cli.pdfa_level.parse().map_err(|e| anyhow::anyhow!("{e}"))?;

    let options = ldir_pdf::converter::PdfOptions {
        title: cli.title.clone(),
        author: cli.author.clone(),
        subject: cli.subject.clone(),
        header_left: cli.header_left.clone(),
        header_right: cli.header_right.clone(),
        footer_left: cli.footer_left.clone(),
        footer_right: cli.footer_right.clone(),
        conformance,
        ..Default::default()
    };

    let total_fonts = 1 + variant_fonts.len();

    let file =
        File::create(output).with_context(|| format!("failed to create {}", output.display()))?;
    let sink = BufWriter::new(file);

    let result = if let Some(primary) = font_data {
        let mut font_data_vec: Vec<Arc<Vec<u8>>> = vec![primary.clone()];
        font_data_vec.resize(5, primary.clone());
        for (id, data) in variant_fonts {
            let idx = *id as usize;
            if idx < font_data_vec.len() {
                font_data_vec[idx] = data.clone();
            }
        }

        let font_faces: Vec<ldir_pdf::font::FontFace> = font_data_vec
            .iter()
            .filter_map(|data| ldir_pdf::font::FontFace::from_bytes(data).ok())
            .collect();

        if font_faces.len() < font_data_vec.len() {
            let warn = styled("warn", Color::Yellow);
            eprintln!(
                "  {warn}: {} font variants failed to load, using fallback",
                font_data_vec.len() - font_faces.len()
            );
        }

        ldir_pdf::converter::gir_to_pdf_streaming(gir_doc, &font_faces, &options, sink)
    } else {
        let warn = styled("warn", Color::Yellow);
        eprintln!(
            "  {warn}: no embedded fonts available, using viewer-resident Helvetica fallback"
        );
        ldir_pdf::converter::gir_to_pdf_streaming(gir_doc, &[], &options, sink)
    };

    result.with_context(|| format!("failed to write {}", output.display()))?;

    let tag = styled("pdf", Color::Green);
    let embed = if font_data.is_some() {
        "embedded"
    } else {
        "fallback"
    };
    let bytes = std::fs::metadata(output).map(|m| m.len()).unwrap_or(0);
    eprintln!("  {tag} {} bytes ({}, {} fonts)", bytes, embed, total_fonts);
    let wrote = styled("wrote", Color::Green);
    eprintln!("  {wrote} {}", output.display());
    Ok(())
}

fn source_location_from_error(module: &SIRModuleV2, err: &LdirError) -> String {
    if let Some(entity_id) = err.entity_id
        && let Some(node) = module.body.get(entity_id)
        && let Some(ref span) = node.source_span
    {
        return format!(" at {}:{}", span.line, span.col);
    }
    String::new()
}

fn main() -> Result<()> {
    let mut cli = Cli::parse();

    let cfg = config::load_config(cli.config.as_deref())?;
    config::apply_config_to_cli(&cfg, &mut cli);

    // Initialize color support
    match cli.color.as_str() {
        "always" => status::set_color(true),
        "never" => status::set_color(false),
        _ => {
            // auto: enable if stderr is a terminal
            let is_tty = std::io::stderr().is_terminal();
            status::set_color(is_tty);
        }
    }

    if cli.inputs.is_empty() && !cli.list_fonts {
        anyhow::bail!("no input files specified. Usage: ldc <INPUT> [INPUTS...] -o <OUTPUT>");
    }

    let mut timer = PipelineTimer::new(4);
    let mut font_db = FontDatabase::new();
    let loaded = font_db.load_system_fonts();
    if loaded > 0 {
        timer.step(&format!("loaded {} system fonts", loaded));
    }

    if cli.list_fonts {
        list_system_fonts(&font_db);
        return Ok(());
    }

    let first_input = &cli.inputs[0];

    let effective_format = if let Some(ref output) = cli.output {
        let out_ext = output.extension().and_then(|e| e.to_str()).unwrap_or("");
        detect_format_from_extension(out_ext)
    } else {
        &cli.format
    };

    let output = match cli.output {
        Some(ref p) => p.clone(),
        None => {
            let stem = first_input
                .file_stem()
                .context("cannot determine output filename")?;
            let ext = match effective_format {
                "html" => "html",
                "epub" => "epub",
                "txt" => "txt",
                "docx" => "docx",
                "sir2" => "sir2",
                "ldir" => "ldir",
                "gir" => "gir",
                _ => "pdf",
            };
            first_input.with_file_name(stem).with_extension(ext)
        }
    };

    if V2_FORMATS.contains(&effective_format) {
        let mut module = merge_modules(&cli.inputs)?;
        if let Some(ref title) = cli.title {
            module.metadata.title = Some(title.clone());
        }
        if let Some(ref author) = cli.author {
            module.metadata.author = Some(author.clone());
        }
        timer.step(&format!(
            "parsed & merged {} files ({} nodes)",
            cli.inputs.len(),
            module.body.len()
        ));
        timer.flush();
        return write_output(&module, &output, effective_format);
    }

    let font_info = resolve_font(&cli, &font_db);
    let font_data = font_info.as_ref().map(|(arc, _)| arc.clone());
    let font_path_str = font_info.as_ref().map(|(_, s)| s.clone());
    if font_data.is_none() {
        timer.warn("no font found, using ASCII monospace stub");
    }

    let primary_family = cli.font.as_deref().unwrap_or("DejaVu Sans");
    let mut variant_fonts: Vec<(u32, Arc<Vec<u8>>)> =
        load_font_variants_from_db(&cli, &font_db, primary_family);

    if let Some(ref path_str) = font_path_str
        && !path_str.starts_with("system:")
    {
        for (id, data) in load_font_variants(path_str) {
            let tag = styled("font", Color::Dim);
            eprintln!("  {tag} variant id={} {} bytes", id, data.len());
            variant_fonts.push((id, Arc::new(data)));
        }
    }

    let margin_pt = (cli.margin * 72.0) as i32;
    let page_size_name = cli.page_size.as_deref();
    let page_dims = if let (Some(w), Some(h)) = (cli.page_width, cli.page_height) {
        Some((w as i32, h as i32))
    } else {
        None
    };
    let mut module = merge_modules(&cli.inputs)?;
    if let Some(ref title) = cli.title {
        module.metadata.title = Some(title.clone());
    }
    if let Some(ref author) = cli.author {
        module.metadata.author = Some(author.clone());
    }
    timer.step(&format!(
        "parsed & merged {} files ({} nodes)",
        cli.inputs.len(),
        module.body.len()
    ));

    for input in &cli.inputs {
        let tex_warnings = ldir_tex::parse_tex_warnings(
            &std::fs::read_to_string(input)
                .with_context(|| format!("failed to read {}", input.display()))?,
        );
        if !tex_warnings.is_empty() {
            let warn = styled("warn", Color::Yellow);
            for (span, msg) in &tex_warnings {
                eprintln!("  {warn}: {}:{}", input.display(), span);
                eprintln!("    {}", msg);
            }
        }
    }

    let (pw, ph) = if let Some((w, h)) = page_dims {
        (w, h)
    } else if let Some(name) = page_size_name {
        ldir_core::compiler::context::parse_page_size(name).unwrap_or((
            ldir_core::compiler::context::DEFAULT_PAGE_WIDTH_PT,
            ldir_core::compiler::context::DEFAULT_PAGE_HEIGHT_PT,
        ))
    } else {
        (
            ldir_core::compiler::context::DEFAULT_PAGE_WIDTH_PT,
            ldir_core::compiler::context::DEFAULT_PAGE_HEIGHT_PT,
        )
    };

    let mut ctx = ldir_core::compiler::context::CompileContext::with_font_margins_and_page(
        font_data.clone(),
        margin_pt,
        margin_pt,
        margin_pt,
        margin_pt,
        pw,
        ph,
    );
    ctx.drop_caps_enabled = cli.drop_caps;
    for (id, data) in &variant_fonts {
        ctx.set_font_variant(*id as usize, Some(data.clone()));
    }
    ctx.font_db = Some(Arc::new(font_db));
    ctx.font_family = cli
        .font
        .clone()
        .unwrap_or_else(|| "DejaVu Sans".to_string());
    ctx.font_mono_family = cli.font_mono.clone().unwrap_or_default();

    if let Some(ref features_str) = cli.ot_features {
        ctx.opentype_features = ldir_core::shaping::Feature::parse_features(features_str);
    }

    let gir_doc = if cli.lir {
        use ldir_core::compile_sir_to_lir;
        use ldir_pdf::lir_render::render_lir_to_gir;

        let source_file = module.header.source_path.as_deref().unwrap_or("<unknown>");
        timer.step("compiling via L-IR pipeline (S-IR -> L-IR -> G-IR)");
        let lir_doc = compile_sir_to_lir(&module, &ctx)
            .map_err(|e| anyhow::anyhow!("L-IR compilation failed ({source_file}): {e}"))?;
        timer.step(&format!("L-IR layout -> {} pages", lir_doc.pages.len()));
        let gir = render_lir_to_gir(&lir_doc);
        timer.step("L-IR -> G-IR render");
        gir
    } else if let Some(ref bib_path) = cli.bibliography {
        let bib_content = std::fs::read_to_string(bib_path)
            .with_context(|| format!("failed to read bibliography: {}", bib_path.display()))?;
        let bib_entries = ldir_core::compiler::bibtex::parse_bib(&bib_content)
            .map_err(|e| anyhow::anyhow!("BibTeX parse error ({}): {}", bib_path.display(), e))?;
        timer.step(&format!(
            "loaded bibliography: {} entries from {}",
            bib_entries.len(),
            bib_path.display()
        ));
        let source_file = module.header.source_path.as_deref().unwrap_or("<unknown>");
        ldir_core::compiler::v2_compile::compile_v2_document_with_bib(
            &module,
            &mut ctx,
            &bib_entries,
        )
        .map_err(|e| {
            let loc = source_location_from_error(&module, &e);
            anyhow::anyhow!("compilation failed ({}): {}{loc}", source_file, e)
        })?
    } else {
        let source_file = module.header.source_path.as_deref().unwrap_or("<unknown>");
        ldir_core::compiler::v2_compile::compile_v2_document(&module, &mut ctx).map_err(|e| {
            let loc = source_location_from_error(&module, &e);
            anyhow::anyhow!("compilation failed ({}): {}{loc}", source_file, e)
        })?
    };
    timer.step(&format!("compiled -> {} pages", gir_doc.page_count()));

    if let Err(errors) = ldir_core::verifier::check_gir(&gir_doc) {
        let source_file = module.header.source_path.as_deref().unwrap_or("<unknown>");
        timer.warn(&format!(
            "G-IR verification ({}): {} warnings",
            source_file,
            errors.len()
        ));
        for err in &errors {
            eprintln!("  {}: {}", source_file, err);
        }
    }

    match effective_format {
        "pdf" => emit_pdf(&gir_doc, &cli, &font_data, &variant_fonts, &output)?,
        "gir" => {
            let bytes = ldir_core::emitter::binary::emit_gir(&gir_doc);
            std::fs::write(&output, &bytes)
                .with_context(|| format!("failed to write {}", output.display()))?;
            timer.finish(&format!(
                "wrote G-IR binary ({} bytes) -> {}",
                bytes.len(),
                output.display()
            ));
        }
        "sir" => {
            let bytes = ldir_ir::sir::v2::serialize_module(&module);
            std::fs::write(&output, &bytes)
                .with_context(|| format!("failed to write {}", output.display()))?;
            timer.finish(&format!(
                "wrote S-IR v2 ({} bytes) -> {}",
                bytes.len(),
                output.display()
            ));
        }
        _ => anyhow::bail!("unsupported output format: {}", effective_format),
    }

    Ok(())
}
