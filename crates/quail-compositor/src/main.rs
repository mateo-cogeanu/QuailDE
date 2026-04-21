use std::thread;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;

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

    println!("quail-compositor boot");
    println!("  session: {}", cli.session);
    println!("  stage: skeleton");
    println!("  renderer: not connected");
    println!("  outputs: not enumerated");
    println!("  shell surface: not attached");
    println!();
    println!("Startup phases:");
    println!("  1. initialize Wayland display");
    println!("  2. create renderer backend");
    println!("  3. register input and output state");
    println!("  4. attach shell surfaces");

    if cli.once {
        println!();
        println!("Initialization preview complete.");
        return Ok(());
    }

    println!();
    println!("Compositor skeleton is alive. Waiting for the real backend implementation.");

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
