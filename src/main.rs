use clap::Parser;

mod cli;
mod domain;

fn main() {
    let _cli = cli::args::Cli::parse();
}
