use clap::Parser;

mod cli;
mod domain;

fn main() {
    let _cli = cli::Cli::parse();
}
