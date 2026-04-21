mod app;
mod config;
mod session;
mod shell;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::app::App;

#[derive(Debug, Parser)]
#[command(
    name = "quailde",
    version,
    about = "Bootstrap a lightweight Linux desktop environment"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Override the config path
    #[arg(long)]
    config: Option<PathBuf>,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    /// Print environment and project diagnostics
    Doctor,
    /// Start the QuailDE bootstrap process
    Start,
    /// Print the current project roadmap
    Roadmap,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Command::Start);

    let app = App::new(cli.config)?;

    match command {
        Command::Doctor => app.doctor(),
        Command::Start => app.start(),
        Command::Roadmap => app.roadmap(),
    }
}
