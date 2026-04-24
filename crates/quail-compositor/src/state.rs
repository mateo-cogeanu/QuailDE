use crate::apps::DesktopApp;
use crate::backend::BackendStatus;
use crate::launcher::LauncherModel;
use crate::output::OutputState;
use crate::scene::SceneGraph;
use crate::shell::ShellSurfaceState;
use crate::terminal::BuiltinTerminalState;
use std::time::{Duration, Instant};
use wayland_server::Resource;
use wayland_server::protocol::{
    wl_keyboard::{KeyState as KeyboardKeyState, WlKeyboard},
    wl_pointer::{ButtonState as PointerButtonState, WlPointer},
    wl_surface::WlSurface,
};

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
    pub keyboard_focus_surface_id: Option<u32>,
    pub last_input_focus_surface: String,
    pub input_events_processed: usize,
    pub last_input_event: String,
    pub installed_apps: Vec<DesktopApp>,
    pub launcher: LauncherModel,
    pub launcher_selected_section: usize,
    pub launcher_search_query: String,
    pub pending_launch: Option<usize>,
    pub startup_apps_launched: usize,
    pub last_launched_app: String,
    pub last_launch_error: String,
    pub pointer_buttons_pressed: usize,
    pub shell_click_active: bool,
    pub cursor_x: i32,
    pub cursor_y: i32,
    pub cursor_x_precise: f32,
    pub cursor_y_precise: f32,
    pub cursor_target_x: f32,
    pub cursor_target_y: f32,
    pub cursor_visible: bool,
    pub launcher_open: bool,
    pub quick_settings_open: bool,
    pub power_menu_open: bool,
    pub active_workspace: usize,
    pub workspace_count: usize,
    pub notifications: Vec<NotificationEntry>,
    pub wifi_enabled: bool,
    pub bluetooth_enabled: bool,
    pub night_light_enabled: bool,
    pub brightness_level: u8,
    pub volume_level: u8,
    pub terminal: BuiltinTerminalState,
    pub focused_surface_id: Option<u32>,
    pub pointer_focus_surface_id: Option<u32>,
    pub dragging_surface_id: Option<u32>,
    pub drag_offset_x: i32,
    pub drag_offset_y: i32,
    pub presented_frames: usize,
    pub quit_requested: bool,
    pub next_serial: u32,
    pub pointer_resources: Vec<WlPointer>,
    pub keyboard_resources: Vec<WlKeyboard>,
}

/// NotificationEntry stores shell toasts together with their birth time so the
/// compositor can expire them automatically instead of leaving stale messages.
#[derive(Debug, Clone)]
pub struct NotificationEntry {
    pub message: String,
    pub created_at: Instant,
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
            keyboard_focus_surface_id: None,
            last_input_focus_surface: "none".to_string(),
            input_events_processed: 0,
            last_input_event: "none".to_string(),
            installed_apps: Vec::new(),
            launcher: LauncherModel {
                sections: Vec::new(),
                entries: Vec::new(),
            },
            launcher_selected_section: 1,
            launcher_search_query: String::new(),
            pending_launch: None,
            startup_apps_launched: 0,
            last_launched_app: "none".to_string(),
            last_launch_error: "none".to_string(),
            pointer_buttons_pressed: 0,
            shell_click_active: false,
            cursor_x: 96,
            cursor_y: 96,
            cursor_x_precise: 96.0,
            cursor_y_precise: 96.0,
            cursor_target_x: 96.0,
            cursor_target_y: 96.0,
            cursor_visible: true,
            launcher_open: false,
            quick_settings_open: false,
            power_menu_open: false,
            active_workspace: 0,
            workspace_count: 4,
            notifications: vec![NotificationEntry {
                message: "Welcome to QuailDE".to_string(),
                created_at: Instant::now(),
            }],
            wifi_enabled: true,
            bluetooth_enabled: false,
            night_light_enabled: false,
            brightness_level: 72,
            volume_level: 48,
            terminal: BuiltinTerminalState::new(),
            focused_surface_id: None,
            pointer_focus_surface_id: None,
            dragging_surface_id: None,
            drag_offset_x: 0,
            drag_offset_y: 0,
            presented_frames: 0,
            quit_requested: false,
            next_serial: 0,
            pointer_resources: Vec::new(),
            keyboard_resources: Vec::new(),
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
                "  keyboard focus surface: {}",
                self.keyboard_focus_surface_id
                    .map(|id| format!("surface-{id}"))
                    .unwrap_or_else(|| "none".to_string())
            ),
            format!(
                "  last input focus surface: {}",
                self.last_input_focus_surface
            ),
            format!("  input events processed: {}", self.input_events_processed),
            format!("  last input event: {}", self.last_input_event),
            format!("  discovered apps: {}", self.installed_apps.len()),
            format!("  launcher entries: {}", self.launcher.entries.len()),
            format!(
                "  launcher selected section: {}",
                self.launcher_selected_section
            ),
            format!("  startup apps launched: {}", self.startup_apps_launched),
            format!("  last launched app: {}", self.last_launched_app),
            format!("  last launch error: {}", self.last_launch_error),
            format!(
                "  pointer buttons pressed: {}",
                self.pointer_buttons_pressed
            ),
            format!("  shell click active: {}", self.shell_click_active),
            format!("  cursor position: {},{}", self.cursor_x, self.cursor_y),
            format!("  cursor visible: {}", self.cursor_visible),
            format!("  launcher open: {}", self.launcher_open),
            format!("  quick settings open: {}", self.quick_settings_open),
            format!("  power menu open: {}", self.power_menu_open),
            format!("  active workspace: {}", self.active_workspace + 1),
            format!("  notifications: {}", self.notifications.len()),
            format!("  terminal visible: {}", self.terminal.snapshot().visible),
            format!(
                "  focused surface: {}",
                self.focused_surface_id
                    .map(|id| format!("surface-{id}"))
                    .unwrap_or_else(|| "none".to_string())
            ),
            format!(
                "  pointer focus surface: {}",
                self.pointer_focus_surface_id
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
        self.cursor_target_x = self.cursor_target_x.clamp(0.0, max_x as f32);
        self.cursor_target_y = self.cursor_target_y.clamp(0.0, max_y as f32);
        self.cursor_x = self.cursor_x_precise.round() as i32;
        self.cursor_y = self.cursor_y_precise.round() as i32;
    }

    /// move_cursor_relative applies high-resolution relative motion before the
    /// integer cursor position is derived for raster composition.
    pub fn move_cursor_relative(&mut self, delta_x: f32, delta_y: f32) {
        self.cursor_target_x += delta_x;
        self.cursor_target_y += delta_y;
        self.cursor_x_precise = self.cursor_target_x;
        self.cursor_y_precise = self.cursor_target_y;
        self.clamp_cursor();
    }

    /// move_cursor_absolute maps absolute-pointer devices directly onto the
    /// output space so VM tablets follow the hand without added cursor lag.
    pub fn move_cursor_absolute(&mut self, target_x: f32, target_y: f32) {
        self.cursor_target_x = target_x;
        self.cursor_target_y = target_y;
        self.cursor_x_precise = target_x;
        self.cursor_y_precise = target_y;
        self.clamp_cursor();
    }

    /// update_input_focus maps the cursor position onto the top-most committed
    /// surface so future keyboard and pointer routing has a live focus target.
    pub fn update_input_focus(&mut self) {
        let cursor_x = self.cursor_x;
        let cursor_y = self.cursor_y;
        let focus = self.top_surface_under_cursor(cursor_x, cursor_y);
        self.pointer_focus_surface_id = focus;
        self.last_input_focus_surface = focus
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
                if inside_x
                    && inside_y
                    && surface.is_toplevel
                    && surface.workspace == self.active_workspace
                {
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
            let min_x = 6.min(max_x);
            let max_y = output_height
                .saturating_sub(buffer.height)
                .saturating_sub(12)
                .max(36);
            surface.x = (self.cursor_x - self.drag_offset_x).clamp(min_x, max_x);
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

    /// launcher_app_at_cursor resolves the launcher grid tile under the cursor
    /// to a discovered application index so the shell can start real apps.
    pub fn launcher_app_at_cursor(&self) -> Option<usize> {
        if !self.launcher_open {
            return None;
        }
        let height = self.composed_height.max(0) as usize;
        let panel_height = height.min(620);
        let panel_x = 18;
        let panel_y = height.saturating_sub(panel_height + 78);
        let sidebar_width = 256;
        let cursor_x = self.cursor_x.max(0) as usize;
        let cursor_y = self.cursor_y.max(0) as usize;

        self.visible_launcher_entries()
            .iter()
            .take(12)
            .enumerate()
            .find_map(|(index, entry)| {
                let col = index % 4;
                let row = index / 4;
                let tile_x = panel_x + sidebar_width + 28 + col * 116;
                let tile_y = panel_y + 86 + row * 128;
                let inside_x = cursor_x >= tile_x && cursor_x < tile_x + 96;
                let inside_y = cursor_y >= tile_y && cursor_y < tile_y + 102;
                (inside_x && inside_y).then_some(entry.app_index)
            })
    }

    /// launcher_section_at_cursor resolves the launcher sidebar row under the
    /// cursor so the menu can switch categories like a normal DE launcher.
    pub fn launcher_section_at_cursor(&self) -> Option<usize> {
        if !self.launcher_open {
            return None;
        }
        let height = self.composed_height.max(0) as usize;
        let panel_height = height.min(620);
        let panel_y = height.saturating_sub(panel_height + 78);
        let cursor_x = self.cursor_x.max(0) as usize;
        let cursor_y = self.cursor_y.max(0) as usize;
        if cursor_x < 12 || cursor_x >= 244 {
            return None;
        }
        self.launcher
            .sections
            .iter()
            .enumerate()
            .find_map(|(index, _section)| {
                let item_y = panel_y + 74 + index * 52;
                (cursor_y >= item_y && cursor_y < item_y + 44).then_some(index)
            })
    }

    /// launcher_bounds_contains returns whether the cursor currently sits inside
    /// the open launcher surface so outside clicks can dismiss it.
    pub fn launcher_bounds_contains(&self) -> bool {
        if !self.launcher_open {
            return false;
        }
        let width = self.composed_width.max(0) as usize;
        let height = self.composed_height.max(0) as usize;
        let panel_width = width.min(780);
        let panel_height = height.min(620);
        let panel_x = 18;
        let panel_y = height.saturating_sub(panel_height + 78);
        let cursor_x = self.cursor_x.max(0) as usize;
        let cursor_y = self.cursor_y.max(0) as usize;
        cursor_x >= panel_x
            && cursor_x < panel_x + panel_width
            && cursor_y >= panel_y
            && cursor_y < panel_y + panel_height
    }

    /// menu_button_at_cursor resolves the start-style menu button in the bottom
    /// panel so QuailDE can toggle the launcher like a normal DE shell.
    pub fn menu_button_at_cursor(&self) -> bool {
        let height = self.composed_height.max(0) as usize;
        let panel_y = height.saturating_sub(54);
        let cursor_x = self.cursor_x.max(0) as usize;
        let cursor_y = self.cursor_y.max(0) as usize;
        cursor_x >= 12 && cursor_x < 52 && cursor_y >= panel_y + 7 && cursor_y < panel_y + 47
    }

    /// workspace_at_cursor resolves the workspace pill under the pointer in the
    /// bottom panel so QuailDE can switch desktops without a keyboard shortcut.
    pub fn workspace_at_cursor(&self) -> Option<usize> {
        let height = self.composed_height.max(0) as usize;
        let panel_y = height.saturating_sub(54);
        let start_x = 392;
        let cursor_x = self.cursor_x.max(0) as usize;
        let cursor_y = self.cursor_y.max(0) as usize;
        if cursor_y < panel_y + 11 || cursor_y >= panel_y + 43 {
            return None;
        }
        (0..self.workspace_count).find(|workspace| {
            let pill_x = start_x + workspace * 42;
            cursor_x >= pill_x && cursor_x < pill_x + 34
        })
    }

    /// quick_settings_button_at_cursor resolves the panel control that opens
    /// QuailDE's daily-driver toggles for connectivity, brightness, and volume.
    pub fn quick_settings_button_at_cursor(&self) -> bool {
        let height = self.composed_height.max(0) as usize;
        let panel_y = height.saturating_sub(54);
        let cursor_x = self.cursor_x.max(0) as usize;
        let cursor_y = self.cursor_y.max(0) as usize;
        let button_x = self.composed_width.max(0) as usize - 206;
        cursor_x >= button_x
            && cursor_x < button_x + 74
            && cursor_y >= panel_y + 10
            && cursor_y < panel_y + 42
    }

    /// power_button_at_cursor resolves the panel control that opens the power
    /// menu with lock, log-out, restart, and shutdown-style actions.
    pub fn power_button_at_cursor(&self) -> bool {
        let height = self.composed_height.max(0) as usize;
        let panel_y = height.saturating_sub(54);
        let cursor_x = self.cursor_x.max(0) as usize;
        let cursor_y = self.cursor_y.max(0) as usize;
        let button_x = self.composed_width.max(0) as usize - 116;
        cursor_x >= button_x
            && cursor_x < button_x + 44
            && cursor_y >= panel_y + 10
            && cursor_y < panel_y + 42
    }

    /// quick_settings_action_at_cursor resolves the toggle row under the
    /// pointer so the menu can flip common desktop settings quickly.
    pub fn quick_settings_action_at_cursor(&self) -> Option<usize> {
        if !self.quick_settings_open {
            return None;
        }
        let panel_x = self.composed_width.max(0) as usize - 286;
        let panel_y = self.composed_height.max(0) as usize - 270;
        let cursor_x = self.cursor_x.max(0) as usize;
        let cursor_y = self.cursor_y.max(0) as usize;
        if cursor_x < panel_x + 18 || cursor_x >= panel_x + 250 {
            return None;
        }
        (0..5).find(|index| {
            let item_y = panel_y + 52 + index * 34;
            cursor_y >= item_y && cursor_y < item_y + 26
        })
    }

    /// power_action_at_cursor resolves the selected item inside the power menu.
    pub fn power_action_at_cursor(&self) -> Option<usize> {
        if !self.power_menu_open {
            return None;
        }
        let panel_x = self.composed_width.max(0) as usize - 224;
        let panel_y = self.composed_height.max(0) as usize - 246;
        let cursor_x = self.cursor_x.max(0) as usize;
        let cursor_y = self.cursor_y.max(0) as usize;
        if cursor_x < panel_x + 18 || cursor_x >= panel_x + 186 {
            return None;
        }
        (0..4).find(|index| {
            let item_y = panel_y + 48 + index * 38;
            cursor_y >= item_y && cursor_y < item_y + 28
        })
    }

    /// quick_settings_bounds_contains lets outside clicks dismiss the quick
    /// settings popover like a normal desktop panel menu.
    pub fn quick_settings_bounds_contains(&self) -> bool {
        if !self.quick_settings_open {
            return false;
        }
        let panel_x = self.composed_width.max(0) as usize - 286;
        let panel_y = self.composed_height.max(0) as usize - 270;
        let cursor_x = self.cursor_x.max(0) as usize;
        let cursor_y = self.cursor_y.max(0) as usize;
        cursor_x >= panel_x
            && cursor_x < panel_x + 268
            && cursor_y >= panel_y
            && cursor_y < panel_y + 208
    }

    /// power_menu_bounds_contains lets outside clicks dismiss the power menu.
    pub fn power_menu_bounds_contains(&self) -> bool {
        if !self.power_menu_open {
            return false;
        }
        let panel_x = self.composed_width.max(0) as usize - 224;
        let panel_y = self.composed_height.max(0) as usize - 246;
        let cursor_x = self.cursor_x.max(0) as usize;
        let cursor_y = self.cursor_y.max(0) as usize;
        cursor_x >= panel_x
            && cursor_x < panel_x + 196
            && cursor_y >= panel_y
            && cursor_y < panel_y + 212
    }

    /// panel_app_at_cursor resolves the bottom-panel launcher slot under the
    /// cursor to a discovered application index.
    pub fn panel_app_at_cursor(&self) -> Option<usize> {
        let height = self.composed_height.max(0) as usize;
        let dock_y = height.saturating_sub(54);
        let cursor_x = self.cursor_x.max(0) as usize;
        let cursor_y = self.cursor_y.max(0) as usize;

        if cursor_y < dock_y + 9 || cursor_y >= dock_y + 45 {
            return None;
        }

        self.launcher
            .entries
            .iter()
            .take(6)
            .enumerate()
            .find_map(|(index, entry)| {
                let icon_x = 68 + index * 52;
                (cursor_x >= icon_x && cursor_x < icon_x + 36).then_some(entry.app_index)
            })
    }

    /// handle_shell_click resolves launcher toggles and app launches before the
    /// click is forwarded to client windows.
    pub fn handle_shell_click(&mut self) -> bool {
        if self.menu_button_at_cursor() {
            self.launcher_open = !self.launcher_open;
            self.quick_settings_open = false;
            self.power_menu_open = false;
            self.terminal.unfocus();
            return true;
        }
        if let Some(workspace) = self.workspace_at_cursor() {
            self.switch_workspace(workspace);
            return true;
        }
        if self.quick_settings_button_at_cursor() {
            self.quick_settings_open = !self.quick_settings_open;
            self.launcher_open = false;
            self.power_menu_open = false;
            self.terminal.unfocus();
            return true;
        }
        if self.power_button_at_cursor() {
            self.power_menu_open = !self.power_menu_open;
            self.launcher_open = false;
            self.quick_settings_open = false;
            self.terminal.unfocus();
            return true;
        }
        if let Some(action) = self.quick_settings_action_at_cursor() {
            self.apply_quick_settings_action(action);
            return true;
        }
        if let Some(action) = self.power_action_at_cursor() {
            self.apply_power_action(action);
            return true;
        }
        if let Some(index) = self.launcher_section_at_cursor() {
            self.launcher_selected_section = index;
            return true;
        }
        if self.terminal.close_button_hit(self.cursor_x, self.cursor_y) {
            self.terminal.hide();
            return true;
        }
        if self
            .terminal
            .focus_if_contains(self.cursor_x, self.cursor_y)
        {
            self.focused_surface_id = None;
            return true;
        }
        if let Some(index) = self.launcher_app_at_cursor() {
            self.pending_launch = Some(index);
            self.launcher_open = false;
            self.quick_settings_open = false;
            self.power_menu_open = false;
            self.terminal.unfocus();
            return true;
        }
        if let Some(index) = self.panel_app_at_cursor() {
            self.pending_launch = Some(index);
            self.quick_settings_open = false;
            self.power_menu_open = false;
            self.terminal.unfocus();
            return true;
        }
        if self.launcher_open && !self.launcher_bounds_contains() {
            self.launcher_open = false;
            return true;
        }
        if self.quick_settings_open && !self.quick_settings_bounds_contains() {
            self.quick_settings_open = false;
            return true;
        }
        if self.power_menu_open && !self.power_menu_bounds_contains() {
            self.power_menu_open = false;
            return true;
        }
        self.terminal.unfocus();
        false
    }

    /// visible_launcher_entries returns the menu grid after section/search
    /// filtering so the shell stops treating the launcher as a fixed mock list.
    pub fn visible_launcher_entries(&self) -> Vec<&crate::launcher::LauncherEntry> {
        let selected_category = self
            .launcher
            .sections
            .get(self.launcher_selected_section)
            .and_then(|section| section.category);
        let query = self.launcher_search_query.to_ascii_lowercase();

        self.launcher
            .entries
            .iter()
            .filter(|entry| {
                selected_category.is_none_or(|category| entry.category == category)
                    && (query.is_empty()
                        || entry.label.to_ascii_lowercase().contains(&query)
                        || entry.subtitle.to_ascii_lowercase().contains(&query))
            })
            .collect()
    }

    /// route_pointer_motion emits wl_pointer focus and motion events to the
    /// focused client when the shell cursor moves across the scene.
    pub fn route_pointer_motion(&mut self) {
        let serial = self.next_serial();
        let focus_surface = self.focused_surface();

        if self.pointer_focus_surface_id != self.focused_surface_id {
            if let Some(previous) = self.pointer_focus_surface_resource() {
                for pointer in &self.pointer_resources {
                    pointer.leave(serial, &previous);
                }
            }
        }

        if let Some(surface) = focus_surface {
            let local_x = self.surface_local_x(surface.id().protocol_id());
            let local_y = self.surface_local_y(surface.id().protocol_id());
            if self.pointer_focus_surface_id != Some(surface.id().protocol_id()) {
                for pointer in &self.pointer_resources {
                    pointer.enter(serial, &surface, local_x, local_y);
                    pointer.frame();
                }
                self.pointer_enter_serial = serial;
                self.pointer_focus_surface_id = Some(surface.id().protocol_id());
            } else {
                for pointer in &self.pointer_resources {
                    pointer.motion(0, local_x, local_y);
                    pointer.frame();
                }
            }
        } else {
            self.pointer_focus_surface_id = None;
        }
    }

    /// route_pointer_button forwards left-button clicks to the focused client
    /// once shell-level launch and drag handling have already run.
    pub fn route_pointer_button(&mut self, pressed: bool) {
        let serial = self.next_serial();
        let state = if pressed {
            PointerButtonState::Pressed
        } else {
            PointerButtonState::Released
        };
        for pointer in &self.pointer_resources {
            pointer.button(serial, 0, 0x110, state);
            pointer.frame();
        }
    }

    /// route_keyboard_key forwards a raw Linux key code to the focused client.
    pub fn route_keyboard_key(&mut self, linux_key_code: u32, pressed: bool) {
        let Some(surface) = self.focused_surface() else {
            if let Some(previous) = self.keyboard_focus_surface_resource() {
                let serial = self.next_serial();
                for keyboard in &self.keyboard_resources {
                    keyboard.leave(serial, &previous);
                }
                self.keyboard_focus_surface_id = None;
                self.keyboard_enter_serial = 0;
            }
            return;
        };
        let Some(surface_id) = self.focused_surface_id else {
            return;
        };
        let serial = self.next_serial();
        if self.keyboard_focus_surface_id != Some(surface_id) {
            if let Some(previous) = self.keyboard_focus_surface_resource() {
                for keyboard in &self.keyboard_resources {
                    keyboard.leave(serial, &previous);
                }
            }
            for keyboard in &self.keyboard_resources {
                keyboard.enter(serial, &surface, Vec::new());
                keyboard.modifiers(serial, 0, 0, 0, 0);
            }
            self.keyboard_enter_serial = serial;
            self.keyboard_focus_surface_id = Some(surface_id);
        }
        let key_state = if pressed {
            KeyboardKeyState::Pressed
        } else {
            KeyboardKeyState::Released
        };
        let evdev_key = linux_key_code.saturating_sub(8);
        for keyboard in &self.keyboard_resources {
            keyboard.key(serial, 0, evdev_key, key_state);
        }
    }

    fn focused_surface(&self) -> Option<WlSurface> {
        let id = self.focused_surface_id?;
        self.scene.surfaces.get(&id)?.resource.clone()
    }

    fn pointer_focus_surface_resource(&self) -> Option<WlSurface> {
        let id = self.pointer_focus_surface_id?;
        self.scene.surfaces.get(&id)?.resource.clone()
    }

    fn keyboard_focus_surface_resource(&self) -> Option<WlSurface> {
        let id = self.keyboard_focus_surface_id?;
        self.scene.surfaces.get(&id)?.resource.clone()
    }

    fn surface_local_x(&self, surface_id: u32) -> f64 {
        let Some(surface) = self.scene.surfaces.get(&surface_id) else {
            return 0.0;
        };
        f64::from(self.cursor_x.saturating_sub(surface.x))
    }

    fn surface_local_y(&self, surface_id: u32) -> f64 {
        let Some(surface) = self.scene.surfaces.get(&surface_id) else {
            return 0.0;
        };
        f64::from(self.cursor_y.saturating_sub(surface.y))
    }

    fn next_serial(&mut self) -> u32 {
        self.next_serial = self.next_serial.wrapping_add(1).max(1);
        self.next_serial
    }

    /// push_notification records a short shell message so launches, workspace
    /// switches, and power actions have visible user feedback in the desktop.
    pub fn push_notification(&mut self, message: impl Into<String>) {
        self.notifications.push(NotificationEntry {
            message: message.into(),
            created_at: Instant::now(),
        });
        while self.notifications.len() > 4 {
            self.notifications.remove(0);
        }
    }

    /// expire_notifications keeps shell toasts short-lived so they behave like
    /// transient desktop notifications instead of becoming permanent clutter.
    pub fn expire_notifications(&mut self) {
        let ttl = Duration::from_secs(1);
        self.notifications
            .retain(|notification| notification.created_at.elapsed() < ttl);
    }

    /// switch_workspace moves the shell to a different desktop and clears any
    /// focus that would point at windows no longer visible on the new desktop.
    pub fn switch_workspace(&mut self, workspace: usize) {
        let clamped = workspace.min(self.workspace_count.saturating_sub(1));
        self.active_workspace = clamped;
        self.focused_surface_id = None;
        self.dragging_surface_id = None;
        self.terminal.unfocus();
        self.push_notification(format!("Switched to workspace {}", clamped + 1));
    }

    /// route_shell_key lets launcher search and shell overlays react to keys
    /// before input is offered to the terminal or focused client surface.
    pub fn route_shell_key(&mut self, linux_key_code: u32, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        match linux_key_code {
            1 if self.launcher_open || self.quick_settings_open || self.power_menu_open => {
                self.launcher_open = false;
                self.quick_settings_open = false;
                self.power_menu_open = false;
                return true;
            }
            14 if self.launcher_open => {
                self.launcher_search_query.pop();
                return true;
            }
            _ => {}
        }

        if self.launcher_open
            && let Some(ch) = shell_search_char(linux_key_code)
        {
            self.launcher_search_query.push(ch);
            return true;
        }

        if self.launcher_open && linux_key_code == 28 {
            if let Some(entry) = self.visible_launcher_entries().first() {
                self.pending_launch = Some(entry.app_index);
                self.launcher_open = false;
                return true;
            }
        }

        false
    }

    /// route_terminal_key forwards raw Linux key codes to the built-in shell
    /// terminal before client routing, giving QuailDE a fallback terminal app
    /// even when no external graphical terminal is present yet.
    pub fn route_terminal_key(&mut self, linux_key_code: u32, pressed: bool) -> bool {
        self.terminal.handle_key_event(linux_key_code, pressed)
    }

    fn apply_quick_settings_action(&mut self, action: usize) {
        match action {
            0 => {
                self.wifi_enabled = !self.wifi_enabled;
                self.push_notification(if self.wifi_enabled {
                    "Wi-Fi enabled"
                } else {
                    "Wi-Fi disabled"
                });
            }
            1 => {
                self.bluetooth_enabled = !self.bluetooth_enabled;
                self.push_notification(if self.bluetooth_enabled {
                    "Bluetooth enabled"
                } else {
                    "Bluetooth disabled"
                });
            }
            2 => {
                self.night_light_enabled = !self.night_light_enabled;
                self.push_notification(if self.night_light_enabled {
                    "Night light enabled"
                } else {
                    "Night light disabled"
                });
            }
            3 => {
                self.brightness_level = (self.brightness_level.saturating_add(10)).min(100);
                self.push_notification(format!("Brightness {}%", self.brightness_level));
            }
            4 => {
                self.volume_level = (self.volume_level.saturating_add(10)).min(100);
                self.push_notification(format!("Volume {}%", self.volume_level));
            }
            _ => {}
        }
    }

    fn apply_power_action(&mut self, action: usize) {
        match action {
            0 => self.push_notification("Screen lock is not wired yet"),
            1 => self.push_notification("Session log out is not wired yet"),
            2 => {
                self.push_notification("Restart requested");
                self.quit_requested = true;
            }
            3 => {
                self.push_notification("Powering down QuailDE session");
                self.quit_requested = true;
            }
            _ => {}
        }
        self.power_menu_open = false;
    }
}

fn shell_search_char(linux_key_code: u32) -> Option<char> {
    match linux_key_code {
        2 => Some('1'),
        3 => Some('2'),
        4 => Some('3'),
        5 => Some('4'),
        6 => Some('5'),
        7 => Some('6'),
        8 => Some('7'),
        9 => Some('8'),
        10 => Some('9'),
        11 => Some('0'),
        16 => Some('q'),
        17 => Some('w'),
        18 => Some('e'),
        19 => Some('r'),
        20 => Some('t'),
        21 => Some('y'),
        22 => Some('u'),
        23 => Some('i'),
        24 => Some('o'),
        25 => Some('p'),
        30 => Some('a'),
        31 => Some('s'),
        32 => Some('d'),
        33 => Some('f'),
        34 => Some('g'),
        35 => Some('h'),
        36 => Some('j'),
        37 => Some('k'),
        38 => Some('l'),
        44 => Some('z'),
        45 => Some('x'),
        46 => Some('c'),
        47 => Some('v'),
        48 => Some('b'),
        49 => Some('n'),
        50 => Some('m'),
        57 => Some(' '),
        _ => None,
    }
}
