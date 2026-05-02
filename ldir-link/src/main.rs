use clap::Parser;
use std::fs;
use std::path::PathBuf;

/// LDIR IR Linker — merges multiple S-IR v2 modules into one.
#[derive(Parser)]
#[command(name = "ldir-link", version, about)]
struct Cli {
    /// Input files (.ldir text or .sir2 binary)
    inputs: Vec<PathBuf>,

    /// Output file
    #[arg(short, long)]
    output: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    if cli.inputs.is_empty() {
        eprintln!("[ldir-link] Error: no input files specified");
        std::process::exit(1);
    }

    let mut modules = Vec::new();
    for path in &cli.inputs {
        let module = load_module(path);
        modules.push(module);
    }

    let result = ldir_link::link_modules(modules);
    let output = match result {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[ldir-link] Link error: {}", e);
            std::process::exit(1);
        }
    };

    let bytes = ldir_ir::sir::v2::serialize_module(&output);
    fs::write(&cli.output, &bytes).unwrap_or_else(|e| {
        eprintln!("[ldir-link] Error writing {}: {}", cli.output.display(), e);
        std::process::exit(1);
    });
    eprintln!(
        "[ldir-link] Linked {} modules into {} ({} bytes)",
        cli.inputs.len(),
        cli.output.display(),
        bytes.len()
    );
}

fn load_module(path: &PathBuf) -> ldir_ir::sir::v2::SIRModuleV2 {
    let bytes = fs::read(path).unwrap_or_else(|e| {
        eprintln!("[ldir-link] Error reading {}: {}", path.display(), e);
        std::process::exit(1);
    });

    if bytes.starts_with(b"LDIR") {
        ldir_ir::sir::v2::deserialize_module(&bytes).unwrap_or_else(|e| {
            eprintln!("[ldir-link] Error deserializing {}: {}", path.display(), e);
            std::process::exit(1);
        })
    } else {
        let text = String::from_utf8(bytes).unwrap_or_else(|e| {
            eprintln!(
                "[ldir-link] Error reading {}: not valid binary or UTF-8: {}",
                path.display(),
                e
            );
            std::process::exit(1);
        });
        ldir_ir::sir::v2::text_to_module(&text).unwrap_or_else(|e| {
            eprintln!("[ldir-link] Parse error in {}: {}", path.display(), e);
            std::process::exit(1);
        })
    }
}
