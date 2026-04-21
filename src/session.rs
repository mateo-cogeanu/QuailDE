use std::env;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use anyhow::{Context, Result, bail};

use crate::config::{CommandSpec, Config};

#[derive(Debug, Clone)]
pub struct SessionManager {
    session_name: String,
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub label: String,
    pub status: String,
}

pub struct StartSummary {
    pub dry_run: bool,
    pub launched: Vec<String>,
}

impl SessionManager {
    pub fn new(session_name: String) -> Self {
        Self { session_name }
    }

    pub fn startup_target<'a>(&self, config: &'a Config) -> &'a str {
        &config.runtime.target
    }

    pub fn startup_steps(&self, config: &Config) -> Vec<String> {
        let mut steps = vec![
            format!("validate environment for {}", self.session_name),
            format!("load config from {}", config.config_path.display()),
            format!("start {}", config.launch.compositor.name),
        ];

        for service in &config.launch.services {
            steps.push(format!("start {}", service.name));
        }

        steps.push("monitor child processes".to_string());
        steps
    }

    pub fn checks(&self, config: &Config) -> Vec<CheckResult> {
        let mut checks = vec![
            CheckResult {
                label: "target".to_string(),
                status: config.runtime.target.clone(),
            },
            CheckResult {
                label: "platform".to_string(),
                status: env::consts::OS.to_string(),
            },
            CheckResult {
                label: "dry run".to_string(),
                status: config.runtime.dry_run.to_string(),
            },
            CheckResult {
                label: "wayland display".to_string(),
                status: env_value("WAYLAND_DISPLAY"),
            },
            CheckResult {
                label: "xdg session type".to_string(),
                status: env_value("XDG_SESSION_TYPE"),
            },
        ];

        checks.push(binary_check(&config.launch.compositor));
        for service in &config.launch.services {
            checks.push(binary_check(service));
        }

        checks
    }

    pub fn start(&self, config: &Config) -> Result<StartSummary> {
        let launch_plan = std::iter::once(&config.launch.compositor)
            .chain(config.launch.services.iter())
            .collect::<Vec<_>>();

        if config.runtime.dry_run {
            return Ok(StartSummary {
                dry_run: true,
                launched: launch_plan
                    .iter()
                    .map(|spec| format_command(spec))
                    .collect(),
            });
        }

        self.validate(config)?;

        let mut children = Vec::new();
        let mut launched = Vec::new();

        for spec in launch_plan {
            match spawn_child(spec) {
                Ok(child) => {
                    launched.push(format_command(spec));
                    children.push((spec.name.clone(), child));
                }
                Err(error) if spec.optional => {
                    eprintln!("Skipping optional service {}: {error}", spec.name);
                }
                Err(error) => {
                    stop_children(&mut children);
                    return Err(error);
                }
            }
        }

        if children.is_empty() {
            bail!("no processes were started");
        }

        let (name, status) = supervise_children(children)?;
        bail!("session stopped because {name} exited with {status}");
    }

    fn validate(&self, config: &Config) -> Result<()> {
        if env::consts::OS != "linux" {
            bail!("QuailDE sessions can only be started on Linux");
        }

        ensure_command_available(&config.launch.compositor)?;

        for service in &config.launch.services {
            if service.optional && !command_exists(&service.command) {
                continue;
            }
            ensure_command_available(service)?;
        }

        Ok(())
    }
}

fn supervise_children(mut children: Vec<(String, Child)>) -> Result<(String, String)> {
    loop {
        for index in 0..children.len() {
            if let Some(status) = children[index]
                .1
                .try_wait()
                .context("failed to poll child process")?
            {
                let (name, _) = children.remove(index);
                stop_children(&mut children);
                return Ok((name, status.to_string()));
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(250));
    }
}

fn stop_children(children: &mut Vec<(String, Child)>) {
    for (_, child) in children.iter_mut() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn spawn_child(spec: &CommandSpec) -> Result<Child> {
    let program =
        resolve_command_path(&spec.command).unwrap_or_else(|| PathBuf::from(&spec.command));

    Command::new(&program)
        .args(&spec.args)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("failed to launch {}", format_command(spec)))
}

fn ensure_command_available(spec: &CommandSpec) -> Result<()> {
    if resolve_command_path(&spec.command).is_some() {
        Ok(())
    } else {
        bail!("required command '{}' was not found in PATH", spec.command)
    }
}

fn binary_check(spec: &CommandSpec) -> CheckResult {
    let resolution = resolve_command_path(&spec.command);
    let requirement = if spec.optional {
        "optional"
    } else {
        "required"
    };
    let status = if let Some(path) = resolution {
        format!("{requirement}: found {}", path.display())
    } else {
        format!("{requirement}: missing {}", spec.command)
    };

    CheckResult {
        label: spec.name.clone(),
        status,
    }
}

fn command_exists(command: &str) -> bool {
    resolve_command_path(command).is_some()
}

fn resolve_command_path(command: &str) -> Option<PathBuf> {
    if command.contains(std::path::MAIN_SEPARATOR) {
        let path = PathBuf::from(command);
        return path.exists().then_some(path);
    }

    env::var_os("PATH")
        .and_then(|paths| {
            env::split_paths(&paths)
                .map(|path| path.join(command))
                .find(|candidate| candidate.exists())
        })
        .or_else(|| {
            std::env::current_exe().ok().and_then(|exe| {
                let sibling = exe.with_file_name(command);
                sibling.exists().then_some(sibling)
            })
        })
}

fn format_command(spec: &CommandSpec) -> String {
    if spec.args.is_empty() {
        spec.command.clone()
    } else {
        format!("{} {}", spec.command, spec.args.join(" "))
    }
}

fn env_value(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| "<unset>".to_string())
}
