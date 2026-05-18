//! Generate shell completions and man pages for ldc.
//!
//! Usage:
//!   cargo run -p ldc --example generate-completions
//!
//! Outputs to completions/ and man/ in the target directory.

use std::fs;
use std::path::Path;

use clap::CommandFactory;
use clap_complete::Shell;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap_or_else(|_| "target".into());
    let out = Path::new(&out_dir);

    let completions_dir = out.join("completions");
    let man_dir = out.join("man");

    fs::create_dir_all(&completions_dir).expect("create completions dir");
    fs::create_dir_all(&man_dir).expect("create man dir");

    let mut cmd = ldc::Cli::command();
    let bin_name = cmd.get_name().to_string();

    // Shell completions
    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
        clap_complete::generate_to(shell, &mut cmd, &bin_name, &completions_dir)
            .expect("generate completion");
    }

    // Man pages
    clap_mangen::generate_to(cmd, &man_dir).expect("generate man pages");

    println!("Completions written to: {}", completions_dir.display());
    println!("Man pages written to:   {}", man_dir.display());
}
