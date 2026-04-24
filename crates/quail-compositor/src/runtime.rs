use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use wayland_protocols::xdg::shell::server::xdg_wm_base::XdgWmBase;
use wayland_server::protocol::{
    wl_compositor::WlCompositor, wl_output::WlOutput, wl_seat::WlSeat, wl_shm::WlShm,
};
use wayland_server::{Display, ListeningSocket};

use crate::apps::{AppCategory, discover_system_apps, spawn_app};
use crate::backend::{BackendStatus, RuntimeBackend};
use crate::launcher::LauncherModel;
use crate::linux::create_linux_platform;
use crate::protocol::{CompositorGlobal, OutputGlobal, SeatGlobal, ShmGlobal, XdgWmBaseGlobal};
use crate::software::{compose_scene, write_ppm};
use crate::state::CompositorState;

/// RuntimeOptions contains the knobs that shape a single compositor process.
#[derive(Debug, Clone)]
pub struct RuntimeOptions {
    pub session_name: String,
    pub socket_prefix: String,
    pub backend: RuntimeBackend,
    pub dump_frame: Option<PathBuf>,
    pub drm_device: PathBuf,
    pub framebuffer: PathBuf,
    pub input_dir: PathBuf,
    pub use_tty_graphics: bool,
    pub once: bool,
}

/// RuntimeReport is the bootstrap result we print for diagnostics and tests.
#[derive(Debug, Clone)]
pub struct RuntimeReport {
    pub state: CompositorState,
}

/// run_runtime creates a real Wayland display and listening socket, then either
/// exits after bootstrap or keeps a tiny dispatch loop alive.
pub fn run_runtime(options: RuntimeOptions) -> Result<RuntimeReport> {
    let mut state = CompositorState::bootstrap(options.session_name.clone());
    state.backend = BackendStatus::for_backend(options.backend);
    state.stage = match options.backend {
        RuntimeBackend::Raw => "wayland-bootstrap",
        RuntimeBackend::Smithay => "experimental-backend",
    };

    let mut display = Display::<CompositorState>::new().context("failed to create wl_display")?;
    state.backend.display_server = "wl_display created";
    display
        .handle()
        .create_global::<CompositorState, WlCompositor, _>(6, CompositorGlobal);
    display
        .handle()
        .create_global::<CompositorState, WlShm, _>(2, ShmGlobal);
    display
        .handle()
        .create_global::<CompositorState, XdgWmBase, _>(7, XdgWmBaseGlobal);
    display
        .handle()
        .create_global::<CompositorState, WlSeat, _>(9, SeatGlobal);
    display
        .handle()
        .create_global::<CompositorState, WlOutput, _>(4, OutputGlobal);
    state.advertised_globals = 5;
    let socket = ListeningSocket::bind_auto(&options.socket_prefix, 1..=32)
        .context("failed to bind a Wayland listening socket")?;
    let socket_name = socket
        .socket_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "<unnamed>".to_string());

    state.listening_socket = socket_name;
    state.installed_apps = discover_system_apps();
    state.launcher = LauncherModel::from_apps(&state.installed_apps);

    if let Some(path) = &options.dump_frame {
        let frame = compose_scene(&mut state);
        write_ppm(&frame, path)?;
        state.last_frame_checksum = frame.checksum;
        state.last_frame_painted_surfaces = frame.painted_surfaces;
    }

    launch_startup_apps(&mut state);

    if options.once {
        return Ok(RuntimeReport { state });
    }

    let mut linux_platform = if options.backend == RuntimeBackend::Raw {
        Some(create_linux_platform(
            &mut state,
            &options.drm_device,
            &options.framebuffer,
            &options.input_dir,
            options.use_tty_graphics,
        )?)
    } else {
        None
    };

    // This loop keeps the raw compositor alive: it accepts clients, dispatches
    // protocol requests, polls Linux input, renders a software frame, and
    // flushes client responses on a simple fixed cadence.
    loop {
        accept_clients(&socket, &display, &mut state)?;
        display
            .dispatch_clients(&mut state)
            .context("failed to dispatch Wayland client requests")?;
        if let Some(platform) = linux_platform.as_mut() {
            platform.tick(&mut state)?;
        }
        launch_pending_app(&mut state);
        display
            .flush_clients()
            .context("failed to flush Wayland client buffers")?;
        if state.quit_requested {
            break;
        }
        thread::sleep(Duration::from_millis(16));
    }

    Ok(RuntimeReport { state })
}

fn launch_startup_apps(state: &mut CompositorState) {
    let Some(runtime_dir) = std::env::var_os("XDG_RUNTIME_DIR") else {
        return;
    };
    let runtime_dir = PathBuf::from(runtime_dir);
    let Some(app) = state
        .installed_apps
        .iter()
        .find(|app| app.category == AppCategory::Terminal)
        .or_else(|| state.installed_apps.first())
        .cloned()
    else {
        return;
    };

    if let Err(error) = launch_app(state, &app, &runtime_dir) {
        state.last_launch_error = format!("{}: {error}", app.name);
        return;
    }

    state.startup_apps_launched += 1;
    state.last_launched_app = app.name;
    state.last_launch_error = "none".to_string();
}

fn launch_pending_app(state: &mut CompositorState) {
    let Some(index) = state.pending_launch.take() else {
        return;
    };
    let Some(runtime_dir) = std::env::var_os("XDG_RUNTIME_DIR") else {
        return;
    };
    let runtime_dir = PathBuf::from(runtime_dir);
    if let Some(app) = state.installed_apps.get(index).cloned() {
        if let Err(error) = launch_app(state, &app, &runtime_dir) {
            state.last_launch_error = format!("{}: {error}", app.name);
        } else {
            state.last_launched_app = app.name;
            state.last_launch_error = "none".to_string();
        }
    }
}

/// launch_app keeps QuailDE's built-in terminal on the same launch path as
/// normal installed apps so the shell can offer a usable terminal everywhere.
fn launch_app(
    state: &mut CompositorState,
    app: &crate::apps::DesktopApp,
    runtime_dir: &PathBuf,
) -> Result<()> {
    if crate::terminal::BuiltinTerminalState::is_builtin_terminal_command(&app.command) {
        state.terminal.ensure_started()?;
        state.terminal.show();
        return Ok(());
    }

    spawn_app(app, &state.listening_socket, runtime_dir)
}

fn accept_clients(
    socket: &ListeningSocket,
    display: &Display<CompositorState>,
    state: &mut CompositorState,
) -> Result<()> {
    while let Some(stream) = socket
        .accept()
        .context("failed to accept client connection")?
    {
        // `()` is enough client metadata for now; later we can replace it with
        // richer per-client bookkeeping once the compositor owns real objects.
        display
            .handle()
            .insert_client(stream, Arc::new(()))
            .context("failed to register Wayland client")?;
        state.connected_clients += 1;
    }

    Ok(())
}
