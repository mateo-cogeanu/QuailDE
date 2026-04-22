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
/// take while we evolve the compositor. Raw QuailDE remains the default path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum RuntimeBackend {
    /// Keep QuailDE on its own hand-rolled compositor path.
    Raw,
    /// Reserved for future experiments that should not replace the raw path.
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
                display_server: "experimental backend",
                renderer: "experimental renderer path",
                input: "experimental input path",
            },
        }
    }
}
