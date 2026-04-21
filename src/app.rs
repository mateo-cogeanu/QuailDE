use std::env;
use std::path::PathBuf;

use anyhow::Result;

use crate::config::Config;
use crate::session::SessionManager;
use crate::shell::ShellProfile;

pub struct App {
    config: Config,
    session: SessionManager,
    shell: ShellProfile,
}

impl App {
    pub fn new(config_path: Option<PathBuf>) -> Result<Self> {
        let config = Config::load(config_path)?;
        let session = SessionManager::new(config.session_name.clone());
        let shell = ShellProfile::default();

        Ok(Self {
            config,
            session,
            shell,
        })
    }

    pub fn doctor(&self) -> Result<()> {
        println!("QuailDE doctor");
        println!("  session: {}", self.config.session_name);
        println!("  config path: {}", self.config.config_path.display());
        println!("  xdg session type: {}", env_value("XDG_SESSION_TYPE"));
        println!("  wayland display: {}", env_value("WAYLAND_DISPLAY"));
        println!("  display: {}", env_value("DISPLAY"));
        println!("  shell profile: {}", self.shell.name);
        println!("  shell traits: {}", self.shell.traits.join(", "));
        println!(
            "  startup target: {}",
            self.session.startup_target(&self.config)
        );
        println!();
        println!("Checks:");
        for check in self.session.checks(&self.config) {
            println!("  {}: {}", check.label, check.status);
        }
        println!();
        println!("This is a session bootstrap with configurable launch targets.");
        Ok(())
    }

    pub fn start(&self) -> Result<()> {
        println!("Starting QuailDE bootstrap");
        println!("Profile: {}", self.shell.name);
        println!("Vision: {}", self.shell.summary);
        println!("Config: {}", self.config.config_path.display());
        println!();
        println!("Planned startup flow:");
        for step in self.session.startup_steps(&self.config) {
            println!("  - {step}");
        }
        println!();
        let summary = self.session.start(&self.config)?;
        if summary.dry_run {
            println!("Dry run enabled. Planned launches:");
            for launched in summary.launched {
                println!("  - {launched}");
            }
        }
        Ok(())
    }

    pub fn roadmap(&self) -> Result<()> {
        println!("QuailDE roadmap");
        println!("  1. Config and session bootstrap");
        println!("  2. Compositor skeleton");
        println!("  3. Panel surface");
        println!("  4. Launcher and notifications");
        println!("  5. Lock screen and settings");
        Ok(())
    }
}

fn env_value(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| "<unset>".to_string())
}
