use crate::backend::BackendStatus;
use crate::output::OutputState;
use crate::scene::SceneGraph;
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
    pub listening_socket: String,
    pub connected_clients: usize,
    pub advertised_globals: usize,
    pub bound_globals: usize,
    pub surface_commits: usize,
    pub surface_frames_requested: usize,
    pub surface_buffer_attaches: usize,
    pub shm_pools_created: usize,
    pub shm_buffers_created: usize,
    pub buffer_destroy_requests: usize,
    pub last_shm_pool_size: i32,
    pub last_buffer_dimensions: String,
    pub tracked_surfaces: usize,
    pub mapped_surfaces: usize,
    pub last_committed_surface: String,
    pub scene: SceneGraph,
    pub composed_width: i32,
    pub composed_height: i32,
    pub last_frame_checksum: u64,
    pub last_frame_painted_surfaces: usize,
}

impl CompositorState {
    pub fn bootstrap(session_name: String) -> Self {
        Self {
            session_name,
            stage: "bootstrap",
            backend: BackendStatus::placeholder(),
            outputs: OutputState::placeholder(),
            shell: ShellSurfaceState::placeholder(),
            listening_socket: "not bound".to_string(),
            connected_clients: 0,
            advertised_globals: 0,
            bound_globals: 0,
            surface_commits: 0,
            surface_frames_requested: 0,
            surface_buffer_attaches: 0,
            shm_pools_created: 0,
            shm_buffers_created: 0,
            buffer_destroy_requests: 0,
            last_shm_pool_size: 0,
            last_buffer_dimensions: "none".to_string(),
            tracked_surfaces: 0,
            mapped_surfaces: 0,
            last_committed_surface: "none".to_string(),
            scene: SceneGraph::default(),
            composed_width: 1280,
            composed_height: 720,
            last_frame_checksum: 0,
            last_frame_painted_surfaces: 0,
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

    pub fn summary_lines(&self) -> Vec<String> {
        vec![
            format!("  session: {}", self.session_name),
            format!("  stage: {}", self.stage),
            format!("  display server: {}", self.backend.display_server),
            format!("  renderer: {}", self.backend.renderer),
            format!("  outputs: {}", self.outputs.layout),
            format!(
                "  shell surface: {} ({})",
                self.shell.primary_surface, self.shell.layer_shell
            ),
            format!("  wayland socket: {}", self.listening_socket),
            format!("  connected clients: {}", self.connected_clients),
            format!("  advertised globals: {}", self.advertised_globals),
            format!("  bound globals: {}", self.bound_globals),
            format!("  surface commits: {}", self.surface_commits),
            format!(
                "  surface buffer attaches: {}",
                self.surface_buffer_attaches
            ),
            format!(
                "  frame callbacks requested: {}",
                self.surface_frames_requested
            ),
            format!("  shm pools created: {}", self.shm_pools_created),
            format!("  shm buffers created: {}", self.shm_buffers_created),
            format!(
                "  buffer destroy requests: {}",
                self.buffer_destroy_requests
            ),
            format!("  last shm pool size: {}", self.last_shm_pool_size),
            format!("  last buffer dimensions: {}", self.last_buffer_dimensions),
            format!("  tracked surfaces: {}", self.tracked_surfaces),
            format!("  mapped surfaces: {}", self.mapped_surfaces),
            format!("  last committed surface: {}", self.last_committed_surface),
            format!(
                "  software output: {}x{}",
                self.composed_width, self.composed_height
            ),
            format!("  frame checksum: {}", self.last_frame_checksum),
            format!(
                "  painted surfaces in last frame: {}",
                self.last_frame_painted_surfaces
            ),
        ]
    }
}
