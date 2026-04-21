use std::sync::Mutex;

use wayland_server::protocol::{
    wl_buffer::WlBuffer, wl_callback::WlCallback, wl_compositor::WlCompositor, wl_region::WlRegion,
    wl_shm::Format as ShmFormat, wl_shm::WlShm, wl_shm_pool::WlShmPool, wl_surface::WlSurface,
};
use wayland_server::{Client, DataInit, Dispatch, DisplayHandle, GlobalDispatch, New, WEnum};

use crate::state::CompositorState;

#[derive(Debug, Clone, Copy)]
pub struct CompositorGlobal;

#[derive(Debug, Clone, Copy)]
pub struct SurfaceState;

#[derive(Debug, Clone, Copy)]
pub struct RegionState;

#[derive(Debug, Clone, Copy)]
pub struct FrameCallbackState;

#[derive(Debug, Clone, Copy)]
pub struct ShmGlobal;

#[derive(Debug)]
pub struct ShmPoolState {
    pub metadata: Mutex<ShmPoolMetadata>,
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
                data_init.init(id, SurfaceState);
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
                data_init.init(
                    id,
                    ShmPoolState {
                        metadata: Mutex::new(ShmPoolMetadata { size }),
                    },
                );
                state.shm_pools_created += 1;
                state.last_shm_pool_size = size;
                drop(fd);
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
        _data: &SurfaceState,
        _dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            wayland_server::protocol::wl_surface::Request::Frame { callback } => {
                data_init.init(callback, FrameCallbackState);
                state.surface_frames_requested += 1;
            }
            wayland_server::protocol::wl_surface::Request::Commit => {
                state.surface_commits += 1;
            }
            wayland_server::protocol::wl_surface::Request::Attach { buffer, .. } => {
                if buffer.is_some() {
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
                let pool_size = data.metadata.lock().expect("pool metadata poisoned").size;
                data_init.init(
                    id,
                    BufferState {
                        offset,
                        width,
                        height,
                        stride,
                        format_name: shm_format_name(format),
                    },
                );
                state.shm_buffers_created += 1;
                state.last_buffer_dimensions = format!("{width}x{height}");
                state.last_shm_pool_size = pool_size;
            }
            wayland_server::protocol::wl_shm_pool::Request::Resize { size } => {
                data.metadata.lock().expect("pool metadata poisoned").size = size;
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
