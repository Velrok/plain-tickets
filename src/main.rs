use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod application_types;
mod commands;
mod config;
mod domain_types;
mod git;

use application_types::{ArchiveArgs, EditArgs, NewArgs, WorkingDir};
use commands::{cmd_archive, cmd_edit, cmd_init, cmd_list, cmd_new, cmd_show, resolve_dir};
use domain_types::TicketId;

#[derive(Parser)]
#[command(name = "tickets", about = "Plain markdown ticket system")]
struct Cli {
    /// Override the tickets directory (takes precedence over TICKETS_DIR)
    #[arg(long, global = true)]
    dir: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Archive(ArchiveArgs),
    Init,
    List,
    Show { id: TicketId },
    Edit(EditArgs),
    New(NewArgs),
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let base_dir = resolve_dir(cli.dir);

    if let Commands::Init = cli.command {
        return cmd_init(base_dir);
    }

    let working_dir = WorkingDir::new(base_dir)?;
    let cfg = config::load(working_dir.as_ref())?;

    match cli.command {
        Commands::Init => unreachable!(),
        Commands::Archive(args) => cmd_archive(working_dir, &cfg, args),
        Commands::List => cmd_list(working_dir, &cfg),
        Commands::Edit(args) => cmd_edit(working_dir, &cfg, args),
        Commands::Show { id } => cmd_show(working_dir, id),
        Commands::New(args) => cmd_new(working_dir, &cfg, args),
    }
}
