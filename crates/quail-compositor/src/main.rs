use std::thread;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use quail_compositor::runtime::{RuntimeOptions, run_runtime};

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

    /// Socket prefix used when binding the Wayland display
    #[arg(long, default_value = "quailde")]
    socket_prefix: String,

    /// Run initialization once and exit instead of holding the process open
    #[arg(long)]
    once: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let report = run_runtime(RuntimeOptions {
        session_name: cli.session,
        socket_prefix: cli.socket_prefix,
        once: cli.once,
    })?;
    let state = report.state;

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
    println!("Compositor bootstrap is alive. Waiting for clients on the Wayland socket.");

    // The runtime loop is entered before this point when `--once` is not used.
    // This fallback is unreachable today, but keeps control flow explicit.
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
