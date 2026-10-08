use clap::Parser;
use cli::args::{Cli, Command};

mod cli;
mod config;
mod domain;

fn main() {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().unwrap_or_else(|e| fail(format!("cannot read cwd: {e}")));

    if let Command::Init = cli.command {
        let path = config::init(&cwd).unwrap_or_else(|e| fail(e));
        println!("created {}", path.display());
        return;
    }

    let _config = config::load(&cwd).unwrap_or_else(|e| fail(e));
}

fn fail(message: String) -> ! {
    eprintln!("error: {message}");
    std::process::exit(1);
}
