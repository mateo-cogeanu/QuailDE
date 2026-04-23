use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AppCategory {
    Terminal,
    Browser,
    Files,
    Editor,
    Utility,
}

/// discover_system_apps combines desktop-entry discovery with PATH fallbacks so
/// QuailDE can launch real installed apps instead of a hard-coded shortlist.
pub fn discover_system_apps() -> Vec<DesktopApp> {
    let mut discovered = Vec::<DesktopApp>::new();
    let mut seen = BTreeSet::<(String, String)>::new();

    for app in discover_desktop_entries() {
        let key = (app.name.clone(), app.command.clone());
        if seen.insert(key) {
            discovered.push(app);
        }
    }
    for app in discover_path_fallbacks() {
        let key = (app.name.clone(), app.command.clone());
        if seen.insert(key) {
            discovered.push(app);
        }
    }

    discovered.sort_by_key(|app| (category_rank(app.category), app.name.to_ascii_lowercase()));
    discovered
}

/// spawn_app launches a discovered system app with the compositor's Wayland
/// socket exported so native clients can connect to QuailDE.
pub fn spawn_app(app: &DesktopApp, wayland_display: &str, xdg_runtime_dir: &Path) -> Result<()> {
    let mut command = Command::new(&app.command);
    command.args(&app.args);
    command.env("WAYLAND_DISPLAY", wayland_display);
    command.env("XDG_RUNTIME_DIR", xdg_runtime_dir);
    // These environment variables nudge common toolkits toward Wayland-native
    // backends so QuailDE can use installed apps without relying on X11 first.
    command.env("XDG_CURRENT_DESKTOP", "QuailDE");
    command.env("DESKTOP_SESSION", "QuailDE");
    command.env("XDG_SESSION_TYPE", "wayland");
    command.env("GDK_BACKEND", "wayland");
    command.env("QT_QPA_PLATFORM", "wayland");
    command.env("CLUTTER_BACKEND", "wayland");
    command.env("SDL_VIDEODRIVER", "wayland");
    command.env("MOZ_ENABLE_WAYLAND", "1");
    command
        .spawn()
        .with_context(|| format!("failed to launch {}", app.command))?;
    Ok(())
}

fn discover_desktop_entries() -> Vec<DesktopApp> {
    let mut entries = Vec::<DesktopApp>::new();
    for directory in desktop_entry_dirs() {
        let Ok(iter) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in iter.flatten() {
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("desktop") {
                continue;
            }
            let Some(app) = parse_desktop_entry(&path) else {
                continue;
            };
            entries.push(app);
        }
    }

    entries.sort_by_key(|app| (category_rank(app.category), app.name.to_ascii_lowercase()));
    entries
}

fn discover_path_fallbacks() -> Vec<DesktopApp> {
    let candidates = [
        (
            "Terminal",
            AppCategory::Terminal,
            vec![
                ("kgx", vec![]),
                ("gnome-console", vec![]),
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
                ("firefox-esr", vec![]),
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
        for (command, args) in binaries {
            if find_in_path(command).is_some() {
                apps.push(DesktopApp {
                    name: label.to_string(),
                    command: command.to_string(),
                    args: args.into_iter().map(ToString::to_string).collect(),
                    category,
                });
            }
        }
    }

    apps
}

fn find_in_path(binary: &str) -> Option<PathBuf> {
    if binary.contains('/') {
        let path = PathBuf::from(binary);
        return is_executable(&path).then_some(path);
    }
    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|directory| directory.join(binary))
        .find(|candidate| is_executable(candidate))
}

fn is_executable(path: &Path) -> bool {
    path.is_file()
        && path
            .metadata()
            .ok()
            .is_some_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
}

fn desktop_entry_dirs() -> Vec<PathBuf> {
    let mut directories = vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
    ];
    if let Some(home) = env::var_os("HOME") {
        directories.push(PathBuf::from(home).join(".local/share/applications"));
    }
    directories
}

fn parse_desktop_entry(path: &Path) -> Option<DesktopApp> {
    let contents = fs::read_to_string(path).ok()?;
    let mut name = None;
    let mut exec = None;
    let mut categories = None;
    let mut no_display = false;
    let mut terminal = false;
    let mut in_desktop_entry = false;

    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            in_desktop_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_desktop_entry {
            continue;
        }
        let (key, value) = line.split_once('=')?;
        match key {
            "Name" => name = Some(value.trim().to_string()),
            "Exec" => exec = Some(value.trim().to_string()),
            "Categories" => categories = Some(value.trim().to_string()),
            "NoDisplay" => no_display = value.trim().eq_ignore_ascii_case("true"),
            "Terminal" => terminal = value.trim().eq_ignore_ascii_case("true"),
            _ => {}
        }
    }

    if no_display {
        return None;
    }

    let name = name?;
    let exec = exec?;
    let category = classify_desktop_entry(&name, categories.as_deref(), terminal)?;
    let (command, args) = sanitize_exec(&exec)?;
    if find_in_path(&command).is_none() {
        return None;
    }

    Some(DesktopApp {
        name,
        command,
        args,
        category,
    })
}

fn classify_desktop_entry(
    name: &str,
    categories: Option<&str>,
    terminal: bool,
) -> Option<AppCategory> {
    let categories = categories.unwrap_or_default();
    let lower_name = name.to_ascii_lowercase();
    if terminal || categories.contains("TerminalEmulator") || lower_name.contains("terminal") {
        return Some(AppCategory::Terminal);
    }
    if categories.contains("WebBrowser")
        || lower_name.contains("browser")
        || lower_name.contains("firefox")
        || lower_name.contains("chrom")
    {
        return Some(AppCategory::Browser);
    }
    if categories.contains("FileManager")
        || lower_name.contains("files")
        || lower_name.contains("nautilus")
        || lower_name.contains("dolphin")
    {
        return Some(AppCategory::Files);
    }
    if categories.contains("TextEditor")
        || categories.contains("Development")
        || lower_name.contains("editor")
        || lower_name.contains("code")
    {
        return Some(AppCategory::Editor);
    }
    if categories.contains("Utility") || categories.contains("Settings") {
        return Some(AppCategory::Utility);
    }
    None
}

fn sanitize_exec(exec: &str) -> Option<(String, Vec<String>)> {
    let mut parts = exec
        .split_whitespace()
        .filter(|part| !part.starts_with('%'))
        .map(|part| part.trim_matches('"').to_string())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        return None;
    }

    // Desktop entries often wrap the real command in `env VAR=... app`, so the
    // launcher strips that preamble and resolves the actual executable.
    if parts.first().map(String::as_str) == Some("env") {
        parts.remove(0);
        while parts.first().is_some_and(|part| part.contains('=')) {
            parts.remove(0);
        }
    }
    if parts.is_empty() {
        return None;
    }

    let command = parts.remove(0);
    Some((command, parts))
}

fn category_rank(category: AppCategory) -> u8 {
    match category {
        AppCategory::Terminal => 0,
        AppCategory::Browser => 1,
        AppCategory::Files => 2,
        AppCategory::Editor => 3,
        AppCategory::Utility => 4,
    }
}
