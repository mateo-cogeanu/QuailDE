use std::thread;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use quail_compositor::state::CompositorState;

#[derive(Debug, Parser)]
#[command(
    name = "quail-compositor",
    version,
    about = "QuailDE compositor skeleton"
)]
struct Cli {
    /// Session name to report in logs
    #[arg(long, default_value = "QuailDE")]
    session: String,

    /// Run initialization once and exit instead of holding the process open
    #[arg(long)]
    once: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let state = CompositorState::bootstrap(cli.session);

    println!("quail-compositor boot");
    for line in state.summary_lines() {
        println!("{line}");
    }
    println!();
    println!("Startup phases:");
    for (index, phase) in state.startup_phases().iter().enumerate() {
        println!("  {}. {}", index + 1, phase);
    }

    if cli.once {
        println!();
        println!("Initialization preview complete.");
        return Ok(());
    }

    println!();
    // Keep the process alive so the session bootstrap can supervise it like a
    // long-lived compositor while the real backend is still under construction.
    println!("Compositor skeleton is alive. Waiting for the real backend implementation.");

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
