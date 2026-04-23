use crate::apps::DesktopApp;
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
    pub installed_apps: Vec<DesktopApp>,
    pub pending_launch: Option<usize>,
    pub startup_apps_launched: usize,
    pub last_launched_app: String,
    pub last_launch_error: String,
    pub pointer_buttons_pressed: usize,
    pub cursor_x: i32,
    pub cursor_y: i32,
    pub cursor_x_precise: f32,
    pub cursor_y_precise: f32,
    pub cursor_visible: bool,
    pub focused_surface_id: Option<u32>,
    pub dragging_surface_id: Option<u32>,
    pub drag_offset_x: i32,
    pub drag_offset_y: i32,
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
            installed_apps: Vec::new(),
            pending_launch: None,
            startup_apps_launched: 0,
            last_launched_app: "none".to_string(),
            last_launch_error: "none".to_string(),
            pointer_buttons_pressed: 0,
            cursor_x: 96,
            cursor_y: 96,
            cursor_x_precise: 96.0,
            cursor_y_precise: 96.0,
            cursor_visible: true,
            focused_surface_id: None,
            dragging_surface_id: None,
            drag_offset_x: 0,
            drag_offset_y: 0,
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
            format!("  discovered apps: {}", self.installed_apps.len()),
            format!("  startup apps launched: {}", self.startup_apps_launched),
            format!("  last launched app: {}", self.last_launched_app),
            format!("  last launch error: {}", self.last_launch_error),
            format!(
                "  pointer buttons pressed: {}",
                self.pointer_buttons_pressed
            ),
            format!("  cursor position: {},{}", self.cursor_x, self.cursor_y),
            format!("  cursor visible: {}", self.cursor_visible),
            format!(
                "  focused surface: {}",
                self.focused_surface_id
                    .map(|id| format!("surface-{id}"))
                    .unwrap_or_else(|| "none".to_string())
            ),
            format!(
                "  dragging surface: {}",
                self.dragging_surface_id
                    .map(|id| format!("surface-{id}"))
                    .unwrap_or_else(|| "none".to_string())
            ),
            format!("  presented frames: {}", self.presented_frames),
        ]
    }

    /// clamp_cursor keeps the software cursor inside the current output area.
    pub fn clamp_cursor(&mut self) {
        let max_x = self.composed_width.saturating_sub(1).max(0);
        let max_y = self.composed_height.saturating_sub(1).max(0);
        self.cursor_x_precise = self.cursor_x_precise.clamp(0.0, max_x as f32);
        self.cursor_y_precise = self.cursor_y_precise.clamp(0.0, max_y as f32);
        self.cursor_x = self.cursor_x_precise.round() as i32;
        self.cursor_y = self.cursor_y_precise.round() as i32;
    }

    /// move_cursor_relative applies high-resolution relative motion before the
    /// integer cursor position is derived for raster composition.
    pub fn move_cursor_relative(&mut self, delta_x: f32, delta_y: f32) {
        self.cursor_x_precise += delta_x;
        self.cursor_y_precise += delta_y;
        self.clamp_cursor();
    }

    /// move_cursor_absolute eases absolute-pointer devices toward their target
    /// so VM tablet input feels more like a real desktop cursor than a grid.
    pub fn move_cursor_absolute(&mut self, target_x: i32, target_y: i32) {
        let target_x = target_x as f32;
        let target_y = target_y as f32;
        self.cursor_x_precise += (target_x - self.cursor_x_precise) * 0.55;
        self.cursor_y_precise += (target_y - self.cursor_y_precise) * 0.55;
        self.clamp_cursor();
    }

    /// update_input_focus maps the cursor position onto the top-most committed
    /// surface so future keyboard and pointer routing has a live focus target.
    pub fn update_input_focus(&mut self) {
        let cursor_x = self.cursor_x;
        let cursor_y = self.cursor_y;
        self.last_input_focus_surface = self
            .top_surface_under_cursor(cursor_x, cursor_y)
            .map(|id| format!("surface-{id}"))
            .unwrap_or_else(|| "desktop-root".to_string());
    }

    /// top_surface_under_cursor returns the visible toplevel surface at the
    /// given location, preferring the focused window when overlaps occur.
    pub fn top_surface_under_cursor(&self, cursor_x: i32, cursor_y: i32) -> Option<u32> {
        let mut candidates = self
            .scene
            .surfaces
            .iter()
            .filter_map(|(id, surface)| {
                let buffer = surface.committed_buffer.as_ref()?;
                let width = buffer.width.max(0);
                let height = buffer.height.max(0);
                let left = surface.x - 6;
                let top = surface.y - 34;
                let right = surface.x.saturating_add(width).saturating_add(6);
                let bottom = surface.y.saturating_add(height).saturating_add(6);
                let inside_x = cursor_x >= left && cursor_x < right;
                let inside_y = cursor_y >= top && cursor_y < bottom;
                if inside_x && inside_y && surface.is_toplevel {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        candidates.sort_unstable();
        if let Some(focused) = self.focused_surface_id
            && candidates.contains(&focused)
        {
            return Some(focused);
        }
        candidates.pop()
    }

    /// begin_window_drag focuses the hit window and starts dragging when the
    /// cursor lands in the server-side titlebar area of a managed toplevel.
    pub fn begin_window_drag(&mut self) {
        let Some(surface_id) = self.top_surface_under_cursor(self.cursor_x, self.cursor_y) else {
            self.focused_surface_id = None;
            self.dragging_surface_id = None;
            return;
        };
        self.focused_surface_id = Some(surface_id);
        if let Some(surface) = self.scene.surfaces.get(&surface_id)
            && self.cursor_y < surface.y
        {
            self.dragging_surface_id = Some(surface_id);
            self.drag_offset_x = self.cursor_x - surface.x;
            self.drag_offset_y = self.cursor_y - surface.y;
        }
    }

    /// update_drag moves the grabbed window with the software cursor.
    pub fn update_drag(&mut self) {
        let Some(surface_id) = self.dragging_surface_id else {
            return;
        };
        let output_width = self.composed_width;
        let output_height = self.composed_height;
        if let Some(surface) = self.scene.surfaces.get_mut(&surface_id) {
            let buffer = match surface.committed_buffer.as_ref() {
                Some(buffer) => buffer,
                None => return,
            };
            let max_x = output_width
                .saturating_sub(buffer.width)
                .saturating_sub(12)
                .max(0);
            let max_y = output_height
                .saturating_sub(buffer.height)
                .saturating_sub(12)
                .max(36);
            surface.x = (self.cursor_x - self.drag_offset_x).clamp(6, max_x);
            surface.y = (self.cursor_y - self.drag_offset_y).clamp(34, max_y);
            self.last_window_geometry = format!(
                "surface-{} @ {},{} {}x{}",
                surface_id, surface.x, surface.y, buffer.width, buffer.height
            );
        }
    }

    /// end_pointer_press releases any active drag grab.
    pub fn end_pointer_press(&mut self) {
        self.dragging_surface_id = None;
    }

    /// dock_app_at_cursor resolves the dock slot under the cursor to a
    /// discovered application index so clicks can launch installed apps.
    pub fn dock_app_at_cursor(&self) -> Option<usize> {
        let height = self.composed_height.max(0) as usize;
        let dock_y = height.saturating_sub(54);
        let cursor_x = self.cursor_x.max(0) as usize;
        let cursor_y = self.cursor_y.max(0) as usize;

        if cursor_y < dock_y + 9 || cursor_y >= dock_y + 45 {
            return None;
        }

        (0..self.installed_apps.len().min(6)).find(|index| {
            let icon_x = 18 + index * 52;
            cursor_x >= icon_x && cursor_x < icon_x + 36
        })
    }
}
