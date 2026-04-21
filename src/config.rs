use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Config {
    pub session_name: String,
    pub config_path: PathBuf,
    pub runtime: RuntimeConfig,
    pub launch: LaunchConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeConfig {
    #[serde(default = "default_target")]
    pub target: String,
    #[serde(default = "default_dry_run")]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LaunchConfig {
    #[serde(default = "default_compositor")]
    pub compositor: CommandSpec,
    #[serde(default)]
    pub services: Vec<CommandSpec>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommandSpec {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub optional: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct ConfigFile {
    #[serde(default = "default_session_name")]
    session_name: String,
    #[serde(default)]
    runtime: RuntimeConfig,
    #[serde(default)]
    launch: LaunchConfig,
}

impl Config {
    pub fn load(path_override: Option<PathBuf>) -> Result<Self> {
        let config_path = path_override.unwrap_or_else(default_config_path);

        let file_config = if config_path.exists() {
            let raw = fs::read_to_string(&config_path)
                .with_context(|| format!("failed to read config at {}", config_path.display()))?;
            toml::from_str::<ConfigFile>(&raw)
                .with_context(|| format!("failed to parse config at {}", config_path.display()))?
        } else {
            ConfigFile::default()
        };

        Ok(Self {
            session_name: file_config.session_name,
            config_path,
            runtime: file_config.runtime,
            launch: file_config.launch,
        })
    }
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            session_name: default_session_name(),
            runtime: RuntimeConfig::default(),
            launch: LaunchConfig::default(),
        }
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            target: default_target(),
            dry_run: default_dry_run(),
        }
    }
}

impl Default for LaunchConfig {
    fn default() -> Self {
        Self {
            compositor: default_compositor(),
            services: vec![
                CommandSpec {
                    name: "notification-daemon".to_string(),
                    command: "mako".to_string(),
                    args: Vec::new(),
                    optional: true,
                },
                CommandSpec {
                    name: "wallpaper".to_string(),
                    command: "swaybg".to_string(),
                    args: vec!["-c".to_string(), "#101820".to_string()],
                    optional: true,
                },
            ],
        }
    }
}

fn default_config_path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));

    base.join("quailde").join("quailde.toml")
}

fn default_session_name() -> String {
    "QuailDE".to_string()
}

fn default_target() -> String {
    "wayland-session".to_string()
}

fn default_dry_run() -> bool {
    true
}

fn default_compositor() -> CommandSpec {
    CommandSpec {
        name: "compositor".to_string(),
        command: "quail-compositor".to_string(),
        args: vec!["--session".to_string(), "QuailDE".to_string()],
        optional: false,
    }
}
