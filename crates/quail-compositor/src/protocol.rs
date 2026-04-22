use std::fs::File;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use memmap2::MmapOptions;
use wayland_server::protocol::{
    wl_buffer::WlBuffer, wl_callback::WlCallback, wl_compositor::WlCompositor, wl_region::WlRegion,
    wl_shm::Format as ShmFormat, wl_shm::WlShm, wl_shm_pool::WlShmPool, wl_surface::WlSurface,
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
