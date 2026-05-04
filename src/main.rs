use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod application_types;
mod commands;
mod config;
mod domain_types;
mod git;
mod tui;

use application_types::{ArchiveArgs, EditArgs, ListArgs, NewArgs, WorkingDir};
use commands::{cmd_archive, cmd_edit, cmd_init, cmd_list, cmd_new, cmd_show, resolve_dir};
use domain_types::TicketId;

#[derive(Parser)]
#[command(name = "tickets", about = "Plain markdown ticket system")]
struct Cli {
    /// Override the tickets directory (takes precedence over TICKETS_DIR)
    #[arg(long, global = true)]
    dir: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Archive(ArchiveArgs),
    Init,
    List(ListArgs),
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

    match cli.command {
        None => {
            let working_dir = WorkingDir::new(base_dir)?;
            let cfg = config::load(working_dir.as_ref())?;
            tui::run(working_dir, &cfg)
        }
        Some(Commands::Init) => cmd_init(base_dir),
        Some(cmd) => {
            let working_dir = WorkingDir::new(base_dir)?;
            let cfg = config::load(working_dir.as_ref())?;
            match cmd {
                Commands::Init => unreachable!(),
                Commands::Archive(args) => cmd_archive(working_dir, &cfg, args),
                Commands::List(args) => cmd_list(working_dir, &cfg, args),
                Commands::Edit(args) => cmd_edit(working_dir, &cfg, args),
                Commands::Show { id } => cmd_show(working_dir, id),
                Commands::New(args) => cmd_new(working_dir, &cfg, args),
            }
        }
    }
}
