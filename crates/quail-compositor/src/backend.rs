use clap::ValueEnum;

/// BackendStatus keeps the compositor honest about what runtime layer exists
/// today before we connect to a real Wayland backend.
#[derive(Debug, Clone)]
pub struct BackendStatus {
    pub display_server: &'static str,
    pub renderer: &'static str,
    pub input: &'static str,
}

/// RuntimeBackend selects which compositor implementation path QuailDE should
/// take while we evolve from protocol experiments toward a usable shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum RuntimeBackend {
    /// Keep the current hand-rolled protocol bootstrap for low-level testing.
    Raw,
    /// Follow the Smithay-oriented path that is realistic for a daily-ish DE.
    Smithay,
}

impl BackendStatus {
    pub fn placeholder() -> Self {
        Self {
            display_server: "planned",
            renderer: "not connected",
            input: "not registered",
        }
    }

    pub fn for_backend(backend: RuntimeBackend) -> Self {
        match backend {
            RuntimeBackend::Raw => Self {
                display_server: "wl_display bootstrap",
                renderer: "manual protocol path",
                input: "not registered",
            },
            RuntimeBackend::Smithay => Self {
                display_server: "smithay-oriented bootstrap",
                renderer: "smithay renderer path planned",
                input: "smithay seat path planned",
            },
        }
    }
}
