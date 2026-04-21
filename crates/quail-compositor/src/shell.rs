/// ShellSurfaceState describes the first desktop-facing surface contract that
/// the compositor will eventually host, starting with a panel.
#[derive(Debug, Clone)]
pub struct ShellSurfaceState {
    pub primary_surface: &'static str,
    pub layer_shell: &'static str,
}

impl ShellSurfaceState {
    pub fn placeholder() -> Self {
        Self {
            primary_surface: "panel",
            layer_shell: "not attached",
        }
    }
}
