use clap::Parser;

mod domain;

/// Markdown tickets with YAML front matter.
#[derive(Parser)]
#[command(version, about)]
struct Cli {}

fn main() {
    let _cli = Cli::parse();
}
