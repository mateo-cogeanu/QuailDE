/// BufferSnapshot is the renderer-facing description of a client buffer once it
/// has been attached to a surface and is ready to participate in composition.
#[derive(Debug, Clone)]
pub struct BufferSnapshot {
    pub object_id: u32,
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub format_name: String,
}

/// SurfaceSlots models the Wayland pending/committed split so the compositor can
/// apply surface state atomically on commit.
#[derive(Debug, Clone, Default)]
pub struct SurfaceSlots {
    pub pending_buffer: Option<BufferSnapshot>,
    pub committed_buffer: Option<BufferSnapshot>,
    pub commit_count: usize,
}
