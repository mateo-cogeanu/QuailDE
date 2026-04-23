use crate::apps::{AppCategory, DesktopApp};

/// LauncherSection keeps a stable sidebar/category model for the shell UI.
#[derive(Debug, Clone)]
pub struct LauncherSection {
    pub label: String,
    pub category: Option<AppCategory>,
}

/// LauncherEntry is one visible application tile in the launcher grid.
#[derive(Debug, Clone)]
pub struct LauncherEntry {
    pub app_index: usize,
    pub label: String,
    pub subtitle: String,
    pub icon_name: String,
    pub category: AppCategory,
}

/// LauncherModel is the shell-facing view model for the dark application menu.
#[derive(Debug, Clone)]
pub struct LauncherModel {
    pub sections: Vec<LauncherSection>,
    pub entries: Vec<LauncherEntry>,
}

impl LauncherModel {
    /// from_apps converts discovered system apps into a stable launcher model
    /// the renderer and input layer can both use without duplicating layout
    /// assumptions in several places.
    pub fn from_apps(apps: &[DesktopApp]) -> Self {
        let sections = vec![
            LauncherSection {
                label: "Favorites".to_string(),
                category: None,
            },
            LauncherSection {
                label: "All Applications".to_string(),
                category: None,
            },
            LauncherSection {
                label: "Development".to_string(),
                category: Some(AppCategory::Editor),
            },
            LauncherSection {
                label: "Internet".to_string(),
                category: Some(AppCategory::Browser),
            },
            LauncherSection {
                label: "Files".to_string(),
                category: Some(AppCategory::Files),
            },
            LauncherSection {
                label: "System".to_string(),
                category: Some(AppCategory::Utility),
            },
            LauncherSection {
                label: "Terminal".to_string(),
                category: Some(AppCategory::Terminal),
            },
        ];

        let entries = apps
            .iter()
            .enumerate()
            .map(|(app_index, app)| LauncherEntry {
                app_index,
                label: app.name.clone(),
                subtitle: app.command.clone(),
                icon_name: app.icon_name.clone(),
                category: app.category,
            })
            .collect::<Vec<_>>();

        Self { sections, entries }
    }
}
