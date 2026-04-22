use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

/// DesktopApp describes one launchable system application QuailDE can surface
/// in its early shell while a richer app model is still being built out.
#[derive(Debug, Clone)]
pub struct DesktopApp {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub category: AppCategory,
}

/// AppCategory lets the shell give a few familiar placements to common apps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppCategory {
    Terminal,
    Browser,
    Files,
    Editor,
    Utility,
}

/// discover_system_apps finds a small, practical set of installed GUI-ish apps
/// from the current PATH so QuailDE can launch what already exists.
pub fn discover_system_apps() -> Vec<DesktopApp> {
    let candidates = [
        (
            "Terminal",
            AppCategory::Terminal,
            vec![
                ("foot", vec![]),
                ("alacritty", vec![]),
                ("kitty", vec![]),
                ("wezterm", vec!["start"]),
                ("gnome-terminal", vec![]),
                ("xfce4-terminal", vec![]),
                ("konsole", vec![]),
                ("xterm", vec![]),
            ],
        ),
        (
            "Browser",
            AppCategory::Browser,
            vec![
                ("firefox", vec![]),
                ("chromium", vec![]),
                ("google-chrome", vec![]),
                ("qutebrowser", vec![]),
                ("epiphany", vec![]),
            ],
        ),
        (
            "Files",
            AppCategory::Files,
            vec![
                ("nautilus", vec![]),
                ("thunar", vec![]),
                ("pcmanfm", vec![]),
                ("dolphin", vec![]),
            ],
        ),
        (
            "Editor",
            AppCategory::Editor,
            vec![
                ("code", vec![]),
                ("codium", vec![]),
                ("gedit", vec![]),
                ("mousepad", vec![]),
                ("leafpad", vec![]),
            ],
        ),
        (
            "Utility",
            AppCategory::Utility,
            vec![
                ("pavucontrol", vec![]),
                ("galculator", vec![]),
                ("htop", vec![]),
            ],
        ),
    ];

    let mut apps = Vec::new();
    for (label, category, binaries) in candidates {
        if let Some((command, args)) = binaries
            .into_iter()
            .find(|(binary, _)| find_in_path(binary).is_some())
        {
            apps.push(DesktopApp {
                name: label.to_string(),
                command: command.to_string(),
                args: args.into_iter().map(ToString::to_string).collect(),
                category,
            });
        }
    }

    apps
}

/// spawn_app launches a discovered system app with the compositor's Wayland
/// socket exported so native clients can connect to QuailDE.
pub fn spawn_app(app: &DesktopApp, wayland_display: &str, xdg_runtime_dir: &Path) -> Result<()> {
    let mut command = Command::new(&app.command);
    command.args(&app.args);
    command.env("WAYLAND_DISPLAY", wayland_display);
    command.env("XDG_RUNTIME_DIR", xdg_runtime_dir);
    command
        .spawn()
        .with_context(|| format!("failed to launch {}", app.command))?;
    Ok(())
}

fn find_in_path(binary: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|directory| directory.join(binary))
        .find(|candidate| is_executable(candidate))
}

fn is_executable(path: &Path) -> bool {
    path.is_file()
}
