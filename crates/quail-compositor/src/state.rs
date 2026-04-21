use crate::backend::BackendStatus;
use crate::output::OutputState;
use crate::shell::ShellSurfaceState;

/// CompositorState collects the runtime pieces we need before a real desktop
/// session can render windows and shell UI.
#[derive(Debug, Clone)]
pub struct CompositorState {
    pub session_name: String,
    pub stage: &'static str,
    pub backend: BackendStatus,
    pub outputs: OutputState,
    pub shell: ShellSurfaceState,
}

impl CompositorState {
    pub fn bootstrap(session_name: String) -> Self {
        Self {
            session_name,
            stage: "bootstrap",
            backend: BackendStatus::placeholder(),
            outputs: OutputState::placeholder(),
            shell: ShellSurfaceState::placeholder(),
        }
    }

    pub fn startup_phases(&self) -> [&'static str; 4] {
        [
            "initialize Wayland display",
            "create renderer backend",
            "register input and output state",
            "attach the first shell surface",
        ]
    }

    pub fn summary_lines(&self) -> [String; 6] {
        [
            format!("  session: {}", self.session_name),
            format!("  stage: {}", self.stage),
            format!("  display server: {}", self.backend.display_server),
            format!("  renderer: {}", self.backend.renderer),
            format!("  outputs: {}", self.outputs.layout),
            format!(
                "  shell surface: {} ({})",
                self.shell.primary_surface, self.shell.layer_shell
            ),
        ]
    }
}
