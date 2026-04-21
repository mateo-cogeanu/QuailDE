use wayland_server::protocol::{
    wl_callback::WlCallback, wl_compositor::WlCompositor, wl_region::WlRegion,
    wl_surface::WlSurface,
};
use wayland_server::{Client, DataInit, Dispatch, DisplayHandle, GlobalDispatch, New};

use crate::state::CompositorState;

#[derive(Debug, Clone, Copy)]
pub struct CompositorGlobal;

#[derive(Debug, Clone, Copy)]
pub struct SurfaceState;

#[derive(Debug, Clone, Copy)]
pub struct RegionState;

#[derive(Debug, Clone, Copy)]
pub struct FrameCallbackState;

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
