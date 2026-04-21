/// BackendStatus keeps the compositor honest about what runtime layer exists
/// today before we connect to a real Wayland backend.
#[derive(Debug, Clone)]
pub struct BackendStatus {
    pub display_server: &'static str,
    pub renderer: &'static str,
    pub input: &'static str,
}

impl BackendStatus {
    pub fn placeholder() -> Self {
        Self {
            display_server: "planned",
            renderer: "not connected",
            input: "not registered",
        }
    }
}
