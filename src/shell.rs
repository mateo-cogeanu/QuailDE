#[derive(Debug, Clone)]
pub struct ShellProfile {
    pub name: String,
    pub summary: String,
    pub traits: Vec<String>,
}

impl Default for ShellProfile {
    fn default() -> Self {
        Self {
            name: "Feather".to_string(),
            summary: "A calm, keyboard-friendly shell for lightweight Linux systems".to_string(),
            traits: vec![
                "lightweight".to_string(),
                "modern".to_string(),
                "wayland-first".to_string(),
                "cohesive".to_string(),
            ],
        }
    }
}
