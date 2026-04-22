use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use quail_compositor::backend::RuntimeBackend;
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

    /// Select the compositor implementation path.
    #[arg(long, value_enum, default_value_t = RuntimeBackend::Raw)]
    backend: RuntimeBackend,

    /// Write the current software-composed frame to a PPM image file.
    #[arg(long)]
    dump_frame: Option<PathBuf>,

    /// Linux framebuffer device used for the first live raw output path.
    #[arg(long, default_value = "/dev/fb0")]
    framebuffer: PathBuf,

    /// Linux input directory used to discover evdev devices.
    #[arg(long, default_value = "/dev/input")]
    input_dir: PathBuf,

    /// Run initialization once and exit instead of holding the process open
    #[arg(long)]
    once: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let report = run_runtime(RuntimeOptions {
        session_name: cli.session,
        socket_prefix: cli.socket_prefix,
        backend: cli.backend,
        dump_frame: cli.dump_frame,
        framebuffer: cli.framebuffer,
        input_dir: cli.input_dir,
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
    println!("QuailDE runtime exited.");
    Ok(())
}
