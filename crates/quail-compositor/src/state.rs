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
    pub xdg_surfaces_created: usize,
    pub xdg_toplevels_created: usize,
    pub xdg_popups_created: usize,
    pub xdg_last_acked_serial: u32,
    pub last_xdg_configure_serial: u32,
    pub last_xdg_pong: u32,
    pub xdg_ack_count: usize,
    pub last_xdg_surface: String,
    pub last_window_geometry: String,
    pub last_toplevel_title: String,
    pub last_toplevel_app_id: String,
    pub seats_bound: usize,
    pub pointers_created: usize,
    pub keyboards_created: usize,
    pub last_seat_name: String,
    pub pointer_enter_serial: u32,
    pub keyboard_enter_serial: u32,
    pub last_input_focus_surface: String,
    pub input_events_processed: usize,
    pub last_input_event: String,
    pub pointer_buttons_pressed: usize,
    pub cursor_x: i32,
    pub cursor_y: i32,
    pub cursor_visible: bool,
    pub presented_frames: usize,
    pub quit_requested: bool,
    pub next_serial: u32,
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
            xdg_surfaces_created: 0,
            xdg_toplevels_created: 0,
            xdg_popups_created: 0,
            xdg_last_acked_serial: 0,
            last_xdg_configure_serial: 0,
            last_xdg_pong: 0,
            xdg_ack_count: 0,
            last_xdg_surface: "none".to_string(),
            last_window_geometry: "none".to_string(),
            last_toplevel_title: "none".to_string(),
            last_toplevel_app_id: "none".to_string(),
            seats_bound: 0,
            pointers_created: 0,
            keyboards_created: 0,
            last_seat_name: "seat0".to_string(),
            pointer_enter_serial: 0,
            keyboard_enter_serial: 0,
            last_input_focus_surface: "none".to_string(),
            input_events_processed: 0,
            last_input_event: "none".to_string(),
            pointer_buttons_pressed: 0,
            cursor_x: 96,
            cursor_y: 96,
            cursor_visible: true,
            presented_frames: 0,
            quit_requested: false,
            next_serial: 0,
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
            format!("  xdg surfaces created: {}", self.xdg_surfaces_created),
            format!("  xdg toplevels created: {}", self.xdg_toplevels_created),
            format!("  xdg popups created: {}", self.xdg_popups_created),
            format!(
                "  xdg last configure serial: {}",
                self.last_xdg_configure_serial
            ),
            format!("  xdg last ack serial: {}", self.xdg_last_acked_serial),
            format!("  xdg ack count: {}", self.xdg_ack_count),
            format!("  xdg last pong serial: {}", self.last_xdg_pong),
            format!("  last xdg surface: {}", self.last_xdg_surface),
            format!("  last window geometry: {}", self.last_window_geometry),
            format!("  last toplevel title: {}", self.last_toplevel_title),
            format!("  last toplevel app id: {}", self.last_toplevel_app_id),
            format!("  seats bound: {}", self.seats_bound),
            format!("  pointers created: {}", self.pointers_created),
            format!("  keyboards created: {}", self.keyboards_created),
            format!("  last seat name: {}", self.last_seat_name),
            format!("  pointer enter serial: {}", self.pointer_enter_serial),
            format!("  keyboard enter serial: {}", self.keyboard_enter_serial),
            format!(
                "  last input focus surface: {}",
                self.last_input_focus_surface
            ),
            format!("  input events processed: {}", self.input_events_processed),
            format!("  last input event: {}", self.last_input_event),
            format!(
                "  pointer buttons pressed: {}",
                self.pointer_buttons_pressed
            ),
            format!("  cursor position: {},{}", self.cursor_x, self.cursor_y),
            format!("  cursor visible: {}", self.cursor_visible),
            format!("  presented frames: {}", self.presented_frames),
        ]
    }

    /// clamp_cursor keeps the software cursor inside the current output area.
    pub fn clamp_cursor(&mut self) {
        let max_x = self.composed_width.saturating_sub(1).max(0);
        let max_y = self.composed_height.saturating_sub(1).max(0);
        self.cursor_x = self.cursor_x.clamp(0, max_x);
        self.cursor_y = self.cursor_y.clamp(0, max_y);
    }

    /// update_input_focus maps the cursor position onto the top-most committed
    /// surface so future keyboard and pointer routing has a live focus target.
    pub fn update_input_focus(&mut self) {
        let cursor_x = self.cursor_x;
        let cursor_y = self.cursor_y;
        let focused_surface = self.scene.surfaces.iter().rev().find_map(|(id, surface)| {
            let buffer = surface.committed_buffer.as_ref()?;
            let width = buffer.width.max(0);
            let height = buffer.height.max(0);
            let inside_x = cursor_x >= surface.x && cursor_x < surface.x.saturating_add(width);
            let inside_y = cursor_y >= surface.y && cursor_y < surface.y.saturating_add(height);
            if inside_x && inside_y {
                Some(format!("surface-{id}"))
            } else {
                None
            }
        });

        self.last_input_focus_surface =
            focused_surface.unwrap_or_else(|| "desktop-root".to_string());
    }
}
