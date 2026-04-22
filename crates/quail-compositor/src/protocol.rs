use std::fs::File;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use memmap2::MmapOptions;
use wayland_protocols::xdg::shell::server::{
    xdg_popup::XdgPopup, xdg_positioner::XdgPositioner, xdg_surface::XdgSurface,
    xdg_toplevel::XdgToplevel, xdg_wm_base::XdgWmBase,
};
use wayland_server::protocol::{
    wl_buffer::WlBuffer,
    wl_callback::WlCallback,
    wl_compositor::WlCompositor,
    wl_keyboard::{KeymapFormat, WlKeyboard},
    wl_pointer::WlPointer,
    wl_region::WlRegion,
    wl_seat::{Capability as SeatCapability, WlSeat},
    wl_shm::Format as ShmFormat,
    wl_shm::WlShm,
    wl_shm_pool::WlShmPool,
    wl_surface::WlSurface,
};
use wayland_server::{
    Client, DataInit, Dispatch, DisplayHandle, GlobalDispatch, New, Resource, WEnum,
};

use crate::scene::{BufferSnapshot, SceneSurface, ShmPoolBacking, SurfaceSlots};
use crate::software::compose_scene;
use crate::state::CompositorState;

#[derive(Debug, Clone, Copy)]
pub struct CompositorGlobal;

#[derive(Debug)]
pub struct SurfaceState {
    pub slots: Mutex<SurfaceSlots>,
}

#[derive(Debug, Clone, Copy)]
pub struct RegionState;

#[derive(Debug, Clone, Copy)]
pub struct FrameCallbackState;

#[derive(Debug, Clone, Copy)]
pub struct ShmGlobal;

#[derive(Debug, Clone, Copy)]
pub struct XdgWmBaseGlobal;

#[derive(Debug, Clone, Copy)]
pub struct SeatGlobal;

#[derive(Debug, Clone, Copy)]
pub struct XdgPositionerState;

#[derive(Debug, Clone)]
pub struct XdgSurfaceState {
    pub wl_surface_id: u32,
}

#[derive(Debug, Clone, Default)]
pub struct XdgToplevelState;

#[derive(Debug, Clone, Copy)]
pub struct XdgPopupState;

#[derive(Debug, Clone, Copy)]
pub struct PointerState;

#[derive(Debug, Clone, Copy)]
pub struct KeyboardState;

#[derive(Debug)]
pub struct ShmPoolState {
    pub backing: Arc<Mutex<ShmPoolBacking>>,
}

#[derive(Debug, Clone, Copy)]
pub struct ShmPoolMetadata {
    pub size: i32,
}

#[derive(Debug)]
pub struct BufferState {
    pub offset: i32,
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub format_name: String,
    pub backing: Arc<Mutex<ShmPoolBacking>>,
}

impl GlobalDispatch<WlCompositor, CompositorGlobal> for CompositorState {
    fn bind(
        state: &mut Self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: New<WlCompositor>,
        _global_data: &CompositorGlobal,
        data_init: &mut DataInit<'_, Self>,
    ) {
        data_init.init(resource, CompositorGlobal);
        state.bound_globals += 1;
    }
}

impl Dispatch<WlCompositor, CompositorGlobal> for CompositorState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &WlCompositor,
        request: wayland_server::protocol::wl_compositor::Request,
        _data: &CompositorGlobal,
        _dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            // A compositor must at least let clients create surfaces to be
            // protocol-correct enough for higher layers to grow on top.
            wayland_server::protocol::wl_compositor::Request::CreateSurface { id } => {
                let surface = data_init.init(
                    id,
                    SurfaceState {
                        slots: Mutex::new(SurfaceSlots::default()),
                    },
                );
                let object_id = surface.id().protocol_id();
                _state.tracked_surfaces += 1;
                _state.scene.surfaces.insert(
                    object_id,
                    SceneSurface {
                        object_id,
                        ..SceneSurface::default()
                    },
                );
            }
            // Regions are part of the core surface state API, so we expose a
            // no-op implementation early instead of rejecting the request.
            wayland_server::protocol::wl_compositor::Request::CreateRegion { id } => {
                data_init.init(id, RegionState);
            }
            _ => {}
        }
    }
}

impl GlobalDispatch<WlShm, ShmGlobal> for CompositorState {
    fn bind(
        state: &mut Self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: New<WlShm>,
        _global_data: &ShmGlobal,
        data_init: &mut DataInit<'_, Self>,
    ) {
        // Clients need at least the two baseline formats to create shared
        // memory buffers that every compositor is expected to understand.
        let shm = data_init.init(resource, ShmGlobal);
        shm.format(ShmFormat::Argb8888);
        shm.format(ShmFormat::Xrgb8888);
        state.bound_globals += 1;
    }
}

impl GlobalDispatch<XdgWmBase, XdgWmBaseGlobal> for CompositorState {
    fn bind(
        state: &mut Self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: New<XdgWmBase>,
        _global_data: &XdgWmBaseGlobal,
        data_init: &mut DataInit<'_, Self>,
    ) {
        data_init.init(resource, XdgWmBaseGlobal);
        state.bound_globals += 1;
    }
}

impl GlobalDispatch<WlSeat, SeatGlobal> for CompositorState {
    fn bind(
        state: &mut Self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: New<WlSeat>,
        _global_data: &SeatGlobal,
        data_init: &mut DataInit<'_, Self>,
    ) {
        // A desktop seat needs at least pointer and keyboard capability bits so
        // clients can wire up focus, cursor, and shortcut handling later on.
        let seat = data_init.init(resource, SeatGlobal);
        seat.capabilities(SeatCapability::Pointer | SeatCapability::Keyboard);
        seat.name("seat0".to_string());
        state.bound_globals += 1;
        state.seats_bound += 1;
        state.last_seat_name = "seat0".to_string();
    }
}

impl Dispatch<WlShm, ShmGlobal> for CompositorState {
    fn request(
        state: &mut Self,
        _client: &Client,
        _resource: &WlShm,
        request: wayland_server::protocol::wl_shm::Request,
        _data: &ShmGlobal,
        _dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            // A pool holds the backing fd plus the advertised size so later
            // steps can mmap it for real rendering and damage tracking.
            wayland_server::protocol::wl_shm::Request::CreatePool { id, fd, size } => {
                match map_shm_pool(fd, size) {
                    Ok(backing) => {
                        data_init.init(
                            id,
                            ShmPoolState {
                                backing: Arc::new(Mutex::new(backing)),
                            },
                        );
                        state.shm_pools_created += 1;
                        state.last_shm_pool_size = size;
                    }
                    Err(error) => {
                        _resource.post_error(
                            wayland_server::protocol::wl_shm::Error::InvalidFd,
                            format!("failed to map shm pool: {error}"),
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<XdgWmBase, XdgWmBaseGlobal> for CompositorState {
    fn request(
        state: &mut Self,
        _client: &Client,
        resource: &XdgWmBase,
        request: wayland_protocols::xdg::shell::server::xdg_wm_base::Request,
        _data: &XdgWmBaseGlobal,
        _dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            wayland_protocols::xdg::shell::server::xdg_wm_base::Request::CreatePositioner {
                id,
            } => {
                data_init.init(id, XdgPositionerState);
            }
            wayland_protocols::xdg::shell::server::xdg_wm_base::Request::GetXdgSurface {
                id,
                surface,
            } => {
                let xdg_surface = data_init.init(
                    id,
                    XdgSurfaceState {
                        wl_surface_id: surface.id().protocol_id(),
                    },
                );
                state.xdg_surfaces_created += 1;
                state.last_xdg_surface = format!("surface-{}", surface.id().protocol_id());
                send_xdg_surface_configure(state, &xdg_surface);
            }
            wayland_protocols::xdg::shell::server::xdg_wm_base::Request::Pong { serial } => {
                state.last_xdg_pong = serial;
            }
            wayland_protocols::xdg::shell::server::xdg_wm_base::Request::Destroy => {}
            _ => {
                let serial = next_serial(state);
                resource.ping(serial);
            }
        }
    }
}

impl Dispatch<WlSeat, SeatGlobal> for CompositorState {
    fn request(
        state: &mut Self,
        _client: &Client,
        _resource: &WlSeat,
        request: wayland_server::protocol::wl_seat::Request,
        _data: &SeatGlobal,
        _dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            wayland_server::protocol::wl_seat::Request::GetPointer { id } => {
                data_init.init(id, PointerState);
                state.pointers_created += 1;
                state.pointer_enter_serial = 0;
                state.last_input_focus_surface = "pointer-awaiting-focus".to_string();
            }
            wayland_server::protocol::wl_seat::Request::GetKeyboard { id } => {
                let keyboard = data_init.init(id, KeyboardState);
                keyboard.repeat_info(25, 600);
                let _ = KeymapFormat::NoKeymap;
                state.keyboards_created += 1;
                state.keyboard_enter_serial = 0;
                state.last_input_focus_surface = "keyboard-awaiting-focus".to_string();
            }
            wayland_server::protocol::wl_seat::Request::GetTouch { id: _ } => {}
            wayland_server::protocol::wl_seat::Request::Release => {}
            _ => {}
        }
    }
}

impl Dispatch<WlSurface, SurfaceState> for CompositorState {
    fn request(
        state: &mut Self,
        _client: &Client,
        _resource: &WlSurface,
        request: wayland_server::protocol::wl_surface::Request,
        data: &SurfaceState,
        _dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            wayland_server::protocol::wl_surface::Request::Frame { callback } => {
                data_init.init(callback, FrameCallbackState);
                state.surface_frames_requested += 1;
            }
            wayland_server::protocol::wl_surface::Request::Commit => {
                let mut slots = data.slots.lock().expect("surface slots poisoned");
                slots.committed_buffer = slots.pending_buffer.clone();

                if let Some(buffer) = slots.committed_buffer.clone() {
                    state.last_committed_surface = format!(
                        "surface-{} => buffer-{} {}x{} {}",
                        _resource.id().protocol_id(),
                        buffer.object_id,
                        buffer.width,
                        buffer.height,
                        buffer.format_name
                    );
                } else {
                    state.last_committed_surface =
                        format!("surface-{} => detached", _resource.id().protocol_id());
                }
                slots.commit_count += 1;
                state.surface_commits += 1;
                update_scene_surface(state, _resource.id().protocol_id(), &slots);
                let software_frame = compose_scene(state);
                state.last_frame_checksum = software_frame.checksum;
                state.last_frame_painted_surfaces = software_frame.painted_surfaces;
                state.mapped_surfaces = state
                    .scene
                    .surfaces
                    .values()
                    .filter(|surface| surface.committed_buffer.is_some())
                    .count();
            }
            wayland_server::protocol::wl_surface::Request::Attach { buffer, .. } => {
                let has_buffer = buffer.is_some();
                data.slots
                    .lock()
                    .expect("surface slots poisoned")
                    .pending_buffer = buffer.and_then(buffer_snapshot);
                if has_buffer {
                    state.surface_buffer_attaches += 1;
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<XdgPositioner, XdgPositionerState> for CompositorState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &XdgPositioner,
        _request: wayland_protocols::xdg::shell::server::xdg_positioner::Request,
        _data: &XdgPositionerState,
        _dhandle: &DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
    }
}

impl Dispatch<XdgSurface, XdgSurfaceState> for CompositorState {
    fn request(
        state: &mut Self,
        _client: &Client,
        resource: &XdgSurface,
        request: wayland_protocols::xdg::shell::server::xdg_surface::Request,
        data: &XdgSurfaceState,
        _dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            wayland_protocols::xdg::shell::server::xdg_surface::Request::GetToplevel { id } => {
                let toplevel = data_init.init(id, XdgToplevelState);
                send_xdg_toplevel_configure(resource, &toplevel);
                state.xdg_toplevels_created += 1;
            }
            wayland_protocols::xdg::shell::server::xdg_surface::Request::GetPopup {
                id,
                parent: _,
                positioner: _,
            } => {
                data_init.init(id, XdgPopupState);
                state.xdg_popups_created += 1;
            }
            wayland_protocols::xdg::shell::server::xdg_surface::Request::SetWindowGeometry {
                x,
                y,
                width,
                height,
            } => {
                state.last_window_geometry = format!(
                    "surface-{} @ {},{} {}x{}",
                    data.wl_surface_id, x, y, width, height
                );
            }
            wayland_protocols::xdg::shell::server::xdg_surface::Request::AckConfigure {
                serial,
            } => {
                state.xdg_last_acked_serial = serial;
                state.xdg_ack_count += 1;
            }
            _ => {}
        }
    }
}

impl Dispatch<XdgToplevel, XdgToplevelState> for CompositorState {
    fn request(
        state: &mut Self,
        _client: &Client,
        _resource: &XdgToplevel,
        request: wayland_protocols::xdg::shell::server::xdg_toplevel::Request,
        _data: &XdgToplevelState,
        _dhandle: &DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            wayland_protocols::xdg::shell::server::xdg_toplevel::Request::SetTitle { title } => {
                state.last_toplevel_title = title;
            }
            wayland_protocols::xdg::shell::server::xdg_toplevel::Request::SetAppId { app_id } => {
                state.last_toplevel_app_id = app_id;
            }
            _ => {}
        }
    }
}

impl Dispatch<XdgPopup, XdgPopupState> for CompositorState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &XdgPopup,
        _request: wayland_protocols::xdg::shell::server::xdg_popup::Request,
        _data: &XdgPopupState,
        _dhandle: &DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
    }
}

impl Dispatch<WlPointer, PointerState> for CompositorState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &WlPointer,
        _request: wayland_server::protocol::wl_pointer::Request,
        _data: &PointerState,
        _dhandle: &DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
    }
}

impl Dispatch<WlKeyboard, KeyboardState> for CompositorState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &WlKeyboard,
        _request: wayland_server::protocol::wl_keyboard::Request,
        _data: &KeyboardState,
        _dhandle: &DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
    }
}

impl Dispatch<WlRegion, RegionState> for CompositorState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &WlRegion,
        _request: wayland_server::protocol::wl_region::Request,
        _data: &RegionState,
        _dhandle: &DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
    }
}

impl Dispatch<WlShmPool, ShmPoolState> for CompositorState {
    fn request(
        state: &mut Self,
        _client: &Client,
        _resource: &WlShmPool,
        request: wayland_server::protocol::wl_shm_pool::Request,
        data: &ShmPoolState,
        _dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            wayland_server::protocol::wl_shm_pool::Request::CreateBuffer {
                id,
                offset,
                width,
                height,
                stride,
                format,
            } => {
                let pool_size = data.backing.lock().expect("pool backing poisoned").size;
                data_init.init(
                    id,
                    BufferState {
                        offset,
                        width,
                        height,
                        stride,
                        format_name: shm_format_name(format),
                        backing: data.backing.clone(),
                    },
                );
                state.shm_buffers_created += 1;
                state.last_buffer_dimensions = format!("{width}x{height}");
                state.last_shm_pool_size = pool_size;
            }
            wayland_server::protocol::wl_shm_pool::Request::Resize { size } => {
                if let Ok(mut backing) = data.backing.lock() {
                    if let Ok(remapped) = remap_pool(&backing.file, size) {
                        backing.size = size;
                        backing.mmap = remapped;
                        state.last_shm_pool_size = size;
                    }
                }
                state.last_shm_pool_size = size;
            }
            _ => {}
        }
    }
}

impl Dispatch<WlBuffer, BufferState> for CompositorState {
    fn request(
        state: &mut Self,
        _client: &Client,
        _resource: &WlBuffer,
        _request: wayland_server::protocol::wl_buffer::Request,
        data: &BufferState,
        _dhandle: &DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
        // Touch the metadata so the placeholder state stays exercised while we
        // have not yet attached it to a real renderer path.
        let _ = (
            data.offset,
            data.width,
            data.height,
            data.stride,
            &data.format_name,
            data.backing.lock().ok().map(|backing| backing.size),
        );
        state.buffer_destroy_requests += 1;
    }
}

impl Dispatch<WlCallback, FrameCallbackState> for CompositorState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &WlCallback,
        _request: wayland_server::protocol::wl_callback::Request,
        _data: &FrameCallbackState,
        _dhandle: &DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
    }
}

fn shm_format_name(format: WEnum<ShmFormat>) -> String {
    match format {
        WEnum::Value(value) => format!("{value:?}"),
        // Unknown formats are still valid protocol-wise, so we preserve the raw
        // code for future debugging instead of rejecting them too early.
        WEnum::Unknown(raw) => format!("unknown-{raw}"),
    }
}

fn buffer_snapshot(buffer: WlBuffer) -> Option<BufferSnapshot> {
    let data = buffer.data::<BufferState>()?;
    Some(BufferSnapshot {
        object_id: buffer.id().protocol_id(),
        width: data.width,
        height: data.height,
        stride: data.stride,
        format_name: data.format_name.clone(),
        offset: usize::try_from(data.offset).ok()?,
        backing: data.backing.clone(),
    })
}

fn update_scene_surface(state: &mut CompositorState, object_id: u32, slots: &SurfaceSlots) {
    let scene_surface = state
        .scene
        .surfaces
        .entry(object_id)
        .or_insert(SceneSurface {
            object_id,
            ..SceneSurface::default()
        });
    scene_surface.committed_buffer = slots.committed_buffer.clone();
    scene_surface.commit_count = slots.commit_count;
}

fn map_shm_pool(fd: std::os::fd::OwnedFd, size: i32) -> anyhow::Result<ShmPoolBacking> {
    let file = File::from(fd);
    let mmap = remap_pool(&file, size)?;
    Ok(ShmPoolBacking { file, size, mmap })
}

fn remap_pool(file: &File, size: i32) -> anyhow::Result<memmap2::Mmap> {
    let len = usize::try_from(size).context("negative shm pool size")?;
    let mmap = unsafe { MmapOptions::new().len(len).map(file) }
        .context("failed to mmap shared memory pool")?;
    Ok(mmap)
}

fn next_serial(state: &mut CompositorState) -> u32 {
    state.next_serial = state.next_serial.wrapping_add(1).max(1);
    state.next_serial
}

fn send_xdg_surface_configure(state: &mut CompositorState, resource: &XdgSurface) {
    let serial = next_serial(state);
    resource.configure(serial);
    state.last_xdg_configure_serial = serial;
}

fn send_xdg_toplevel_configure(surface: &XdgSurface, toplevel: &XdgToplevel) {
    toplevel.configure(1280, 720, Vec::new());
    surface.configure(1);
}
