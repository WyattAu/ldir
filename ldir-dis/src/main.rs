use clap::Parser;
use std::fs;
use std::path::PathBuf;

/// LDIR IR Disassembler — converts binary S-IR to human-readable .ldir text.
#[derive(Parser)]
#[command(name = "ldir-dis", version, about)]
struct Cli {
    /// Input file (binary .sir2 or .ldir text file)
    input: PathBuf,

    /// Output file (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output format
    #[arg(short = 'f', long, value_name = "FORMAT", default_value = "text")]
    format: String,
}

fn main() {
    let cli = Cli::parse();

    let bytes = fs::read(&cli.input).unwrap_or_else(|e| {
        eprintln!("[ldir-dis] Error reading {}: {}", cli.input.display(), e);
        std::process::exit(1);
    });

    let module = match ldir_ir::sir::v2::deserialize_module(&bytes) {
        Ok(m) => m,
        Err(_) => {
            let text = String::from_utf8(bytes).unwrap_or_else(|e| {
                eprintln!("[ldir-dis] Error: not valid binary S-IR or UTF-8 text: {}", e);
                std::process::exit(1);
            });
            if let Some(ref out) = cli.output {
                fs::write(out, &text).unwrap_or_else(|e| {
                    eprintln!("[ldir-dis] Error writing {}: {}", out.display(), e);
                    std::process::exit(1);
                });
            } else {
                print!("{}", text);
            }
            return;
        }
    };

    let output = match cli.format.as_str() {
        "text" => ldir_ir::sir::v2::module_to_text(&module),
        "json" => serde_json::to_string_pretty(&module).unwrap_or_else(|e| {
            eprintln!("[ldir-dis] Error serializing to JSON: {}", e);
            std::process::exit(1);
        }),
        other => {
            eprintln!("[ldir-dis] Unknown format: {} (use 'text' or 'json')", other);
            std::process::exit(1);
        }
    };

    match cli.output {
        Some(ref out) => {
            fs::write(out, &output).unwrap_or_else(|e| {
                eprintln!("[ldir-dis] Error writing {}: {}", out.display(), e);
                std::process::exit(1);
            });
            eprintln!("[ldir-dis] Wrote {} bytes to {}", output.len(), out.display());
        }
        None => {
            print!("{}", output);
        }
    }
}
