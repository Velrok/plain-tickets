use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "tickets", about = "Plain markdown ticket system")]
struct Cli {
    /// Data directory (overrides TICKETS_DIR env var)
    #[arg(long, global = true)]
    dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialise tickets directory
    Init,
}

fn resolve_dir(cli_dir: Option<PathBuf>) -> PathBuf {
    cli_dir
        .or_else(|| std::env::var("TICKETS_DIR").ok().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("tickets"))
}

fn cmd_init(dir: PathBuf) {
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

    let git_dir = dir.join(".git");
    if git_dir.exists() {
        println!("  git     already initialised");
    } else {
        print!("  git     no .git found — initialise git repo? [y/N] ");
        use std::io::{self, Write};
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        if input.trim().eq_ignore_ascii_case("y") {
            let status = process::Command::new("git")
                .args(["init", dir.to_str().unwrap_or(".")])
                .status();
            match status {
                Ok(s) if s.success() => println!("  git     initialised"),
                Ok(s) => {
                    eprintln!("error: git init exited with {}", s);
                    process::exit(1);
                }
                Err(e) => {
                    eprintln!("error: could not run git: {}", e);
                    process::exit(1);
                }
            }
        } else {
            println!("  git     skipped");
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let dir = resolve_dir(cli.dir);

    match cli.command {
        Some(Commands::Init) | None => cmd_init(dir),
    }
}
