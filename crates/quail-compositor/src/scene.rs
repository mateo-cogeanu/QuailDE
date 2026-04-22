use std::collections::BTreeMap;
use std::fs::File;
use std::sync::{Arc, Mutex};

use memmap2::Mmap;

/// ShmPoolBacking owns the live mmap for one wl_shm_pool so buffers can keep
/// reading pixels even after the pool object itself is destroyed.
#[derive(Debug)]
pub struct ShmPoolBacking {
    pub file: File,
    pub size: i32,
    pub mmap: Mmap,
}

/// BufferSnapshot is the renderer-facing description of a client buffer once it
/// has been attached to a surface and is ready to participate in composition.
#[derive(Clone)]
pub struct BufferSnapshot {
    pub object_id: u32,
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub format_name: String,
    pub offset: usize,
    pub backing: Arc<Mutex<ShmPoolBacking>>,
}

impl BufferSnapshot {
    /// byte_len returns the linear memory span this buffer needs inside the pool.
    pub fn byte_len(&self) -> Option<usize> {
        let height = usize::try_from(self.height).ok()?;
        let stride = usize::try_from(self.stride).ok()?;
        height.checked_mul(stride)
    }

    /// with_bytes provides safe shared access to the mapped buffer slice.
    pub fn with_bytes<R>(&self, f: impl FnOnce(&[u8]) -> R) -> Option<R> {
        let backing = self.backing.lock().ok()?;
        let len = self.byte_len()?;
        let end = self.offset.checked_add(len)?;
        if end > backing.mmap.len() {
            return None;
        }
        Some(f(&backing.mmap[self.offset..end]))
    }
}

impl std::fmt::Debug for BufferSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BufferSnapshot")
            .field("object_id", &self.object_id)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("stride", &self.stride)
            .field("format_name", &self.format_name)
            .field("offset", &self.offset)
            .finish()
    }
}

/// SurfaceSlots models the Wayland pending/committed split so the compositor can
/// apply surface state atomically on commit.
#[derive(Debug, Clone, Default)]
pub struct SurfaceSlots {
    pub pending_buffer: Option<BufferSnapshot>,
    pub committed_buffer: Option<BufferSnapshot>,
    pub commit_count: usize,
}

/// SceneSurface is the committed state the software compositor can draw from.
#[derive(Debug, Clone, Default)]
pub struct SceneSurface {
    pub object_id: u32,
    pub x: i32,
    pub y: i32,
    pub committed_buffer: Option<BufferSnapshot>,
    pub commit_count: usize,
}

/// SceneGraph stores all known surfaces in a stable order for composition.
#[derive(Debug, Clone, Default)]
pub struct SceneGraph {
    pub surfaces: BTreeMap<u32, SceneSurface>,
}
