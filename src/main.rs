use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "tickets", about = "Plain markdown ticket system")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialise tickets directory
    Init,
}

fn resolve_dir() -> PathBuf {
    std::env::var("TICKETS_DIR")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("tickets"))
}

fn init_directories(dir: &PathBuf) {
    let all = dir.join("all");
    let archived = dir.join("archived");

    for path in [&all, &archived] {
        if path.exists() {
            println!("  exists  {}", path.display());
        } else {
            std::fs::create_dir_all(path).unwrap_or_else(|e| {
                eprintln!("error: could not create {}: {}", path.display(), e);
                process::exit(1);
            });
            println!("  created {}", path.display());
        }
    }
}

fn cmd_init(dir: PathBuf) {
    init_directories(&dir);
}

fn main() {
    let cli = Cli::parse();
    let dir = resolve_dir();

    match cli.command {
        Commands::Init => cmd_init(dir),
    }
}
