use clap::Parser;
use std::fs;
use std::path::PathBuf;

/// LDIR IR Optimizer — applies transformation passes to S-IR v2 modules.
#[derive(Parser)]
#[command(name = "ldir-opt", version, about)]
struct Cli {
    /// Input file (.ldir text or .sir2 binary)
    #[arg(default_value = "-")]
    input: PathBuf,

    /// Output file (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Specific passes to run (empty = all)
    #[arg(long)]
    passes: Vec<String>,

    /// List available passes
    #[arg(long)]
    list_passes: bool,

    /// Output format: "binary" or "text"
    #[arg(short = 'f', long, default_value = "binary")]
    output_format: String,
}

fn main() {
    let cli = Cli::parse();

    if cli.list_passes {
        let passes = ldir_opt::all_passes();
        for p in &passes {
            println!("{}", p.name());
        }
        return;
    }

    let module = if cli.input.as_os_str() == "-" {
        use std::io::Read;
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf).unwrap_or_else(|e| {
            eprintln!("[ldir-opt] Error reading stdin: {}", e);
            std::process::exit(1);
        });
        parse_bytes(&buf)
    } else {
        load_module(&cli.input)
    };

    let mut pm = ldir_opt::PassManager::new().max_iterations(10);
    if cli.passes.is_empty() {
        for pass in ldir_opt::all_passes() {
            pm.add_pass(pass);
        }
    } else {
        let all = ldir_opt::all_passes();
        let by_name: std::collections::HashMap<&str, _> =
            all.iter().map(|p| (p.name(), p.as_ref())).collect();
        for name in &cli.passes {
            match by_name.get(name.as_str()) {
                Some(_) => pm.add_pass(Box::new(DummyPass(name.clone()))),
                None => {
                    eprintln!("[ldir-opt] Unknown pass: {}", name);
                    std::process::exit(1);
                }
            }
        }
    }

    let mut module = module;
    let report = pm.run(&mut module);

    for pr in &report.pass_reports {
        eprintln!(
            "[ldir-opt] pass: changed={}, removed={}, added={}, {}",
            pr.changed, pr.nodes_removed, pr.nodes_added, pr.details
        );
    }
    eprintln!(
        "[ldir-opt] Total: {} passes, {} iterations, {} nodes removed, {} nodes added",
        report.passes_run, report.iterations, report.total_nodes_removed, report.total_nodes_added
    );

    let output_bytes = match cli.output_format.as_str() {
        "binary" => ldir_ir::sir::v2::serialize_module(&module),
        "text" => ldir_ir::sir::v2::module_to_text(&module).into_bytes(),
        other => {
            eprintln!(
                "[ldir-opt] Unknown format: {} (use 'binary' or 'text')",
                other
            );
            std::process::exit(1);
        }
    };

    match cli.output {
        Some(ref out) => {
            fs::write(out, &output_bytes).unwrap_or_else(|e| {
                eprintln!("[ldir-opt] Error writing {}: {}", out.display(), e);
                std::process::exit(1);
            });
            eprintln!(
                "[ldir-opt] Wrote {} bytes to {}",
                output_bytes.len(),
                out.display()
            );
        }
        None => {
            use std::io::Write;
            std::io::stdout()
                .write_all(&output_bytes)
                .unwrap_or_else(|e| {
                    eprintln!("[ldir-opt] Error writing stdout: {}", e);
                    std::process::exit(1);
                });
        }
    }
}

fn load_module(path: &PathBuf) -> ldir_ir::sir::v2::SIRModuleV2 {
    let bytes = fs::read(path).unwrap_or_else(|e| {
        eprintln!("[ldir-opt] Error reading {}: {}", path.display(), e);
        std::process::exit(1);
    });
    parse_bytes(&bytes)
}

fn parse_bytes(bytes: &[u8]) -> ldir_ir::sir::v2::SIRModuleV2 {
    if bytes.starts_with(b"LDIR") {
        ldir_ir::sir::v2::deserialize_module(bytes).unwrap_or_else(|e| {
            eprintln!("[ldir-opt] Error deserializing: {}", e);
            std::process::exit(1);
        })
    } else {
        let text = String::from_utf8(bytes.to_vec()).unwrap_or_else(|e| {
            eprintln!(
                "[ldir-opt] Error: not valid binary S-IR or UTF-8 text: {}",
                e
            );
            std::process::exit(1);
        });
        ldir_ir::sir::v2::text_to_module(&text).unwrap_or_else(|e| {
            eprintln!("[ldir-opt] Parse error: {}", e);
            std::process::exit(1);
        })
    }
}

struct DummyPass(String);
impl ldir_opt::Pass for DummyPass {
    fn name(&self) -> &str {
        &self.0
    }
    fn run(&self, _: &mut ldir_ir::sir::v2::SIRModuleV2) -> ldir_opt::PassResult {
        ldir_opt::PassResult {
            changed: false,
            nodes_removed: 0,
            nodes_added: 0,
            details: "no-op".to_string(),
        }
    }
}
