use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use wayland_server::protocol::{wl_compositor::WlCompositor, wl_shm::WlShm};
use wayland_server::{Display, ListeningSocket};

use crate::backend::{BackendStatus, RuntimeBackend};
use crate::protocol::{CompositorGlobal, ShmGlobal};
use crate::state::CompositorState;

/// RuntimeOptions contains the knobs that shape a single compositor process.
#[derive(Debug, Clone)]
pub struct RuntimeOptions {
    pub session_name: String,
    pub socket_prefix: String,
    pub backend: RuntimeBackend,
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
    state.advertised_globals = 2;
    let socket = ListeningSocket::bind_auto(&options.socket_prefix, 1..=32)
        .context("failed to bind a Wayland listening socket")?;
    let socket_name = socket
        .socket_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "<unnamed>".to_string());

    state.listening_socket = socket_name;

    if options.once {
        return Ok(RuntimeReport { state });
    }

    // This loop is intentionally small: it accepts clients, dispatches pending
    // protocol requests, flushes responses, and sleeps briefly.
    loop {
        accept_clients(&socket, &display, &mut state)?;
        display
            .dispatch_clients(&mut state)
            .context("failed to dispatch Wayland client requests")?;
        display
            .flush_clients()
            .context("failed to flush Wayland client buffers")?;
        thread::sleep(Duration::from_millis(16));
    }
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
