use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::scene::{BufferSnapshot, SceneSurface};
use crate::state::CompositorState;

/// SoftwareFrame summarizes the current in-memory composition result.
#[derive(Debug, Clone)]
pub struct SoftwareFrame {
    pub width: usize,
    pub height: usize,
    pub checksum: u64,
    pub painted_surfaces: usize,
    pub pixels: Vec<u32>,
}

/// compose_scene paints the committed scene into an in-memory XRGB8888 frame so
/// QuailDE can validate composition before a real output backend exists.
pub fn compose_scene(state: &mut CompositorState) -> SoftwareFrame {
    let width = usize::try_from(state.composed_width).unwrap_or(1280);
    let height = usize::try_from(state.composed_height).unwrap_or(720);

    let mut pixels = vec![0xFF101820_u32; width.saturating_mul(height)];
    let mut painted_surfaces = 0;

    paint_background(&mut pixels, width, height);
    paint_panel(&mut pixels, width, height);

    for surface in state.scene.surfaces.values() {
        if let Some(buffer) = &surface.committed_buffer {
            if paint_surface(&mut pixels, width, height, surface, buffer) {
                painted_surfaces += 1;
            }
        }
    }

    if state.cursor_visible {
        paint_cursor(&mut pixels, width, height, state.cursor_x, state.cursor_y);
    }

    let checksum = pixels.iter().fold(0_u64, |acc, pixel| {
        acc.wrapping_mul(1_099_511_628_211)
            .wrapping_add(u64::from(*pixel))
    });

    SoftwareFrame {
        width,
        height,
        checksum,
        painted_surfaces,
        pixels,
    }
}

/// write_ppm exports the current composed frame to a simple binary PPM image so
/// QuailDE can be inspected before a real output backend exists.
pub fn write_ppm(frame: &SoftwareFrame, path: &Path) -> Result<()> {
    let mut bytes = Vec::with_capacity(
        format!("P6\n{} {}\n255\n", frame.width, frame.height).len()
            + frame.pixels.len().saturating_mul(3),
    );
    bytes.extend_from_slice(format!("P6\n{} {}\n255\n", frame.width, frame.height).as_bytes());

    for pixel in &frame.pixels {
        let [blue, green, red, _alpha] = pixel.to_le_bytes();
        bytes.extend_from_slice(&[red, green, blue]);
    }

    fs::write(path, bytes).with_context(|| format!("failed to write {}", path.display()))
}

fn paint_surface(
    frame: &mut [u32],
    frame_width: usize,
    frame_height: usize,
    surface: &SceneSurface,
    buffer: &BufferSnapshot,
) -> bool {
    let width = match usize::try_from(buffer.width) {
        Ok(width) => width,
        Err(_) => return false,
    };
    let height = match usize::try_from(buffer.height) {
        Ok(height) => height,
        Err(_) => return false,
    };
    let stride = match usize::try_from(buffer.stride) {
        Ok(stride) => stride,
        Err(_) => return false,
    };
    let origin_x = surface.x.max(0) as usize;
    let origin_y = surface.y.max(0) as usize;

    let Some(()) = buffer.with_bytes(|bytes| {
        for y in 0..height {
            let row_start = y.saturating_mul(stride);
            for x in 0..width {
                let pixel_start = row_start.saturating_add(x.saturating_mul(4));
                if pixel_start.saturating_add(4) > bytes.len() {
                    return;
                }
                let pixel = u32::from_le_bytes([
                    bytes[pixel_start],
                    bytes[pixel_start + 1],
                    bytes[pixel_start + 2],
                    bytes[pixel_start + 3],
                ]);
                let dst_x = origin_x.saturating_add(x);
                let dst_y = origin_y.saturating_add(y);
                if dst_x >= frame_width || dst_y >= frame_height {
                    continue;
                }

                frame[dst_y * frame_width + dst_x] = normalize_pixel(pixel, &buffer.format_name);
            }
        }
    }) else {
        return false;
    };

    true
}

fn normalize_pixel(pixel: u32, format_name: &str) -> u32 {
    match format_name {
        "Argb8888" => pixel,
        "Xrgb8888" => pixel | 0xFF00_0000,
        _ => pixel | 0xFF00_0000,
    }
}

fn paint_background(frame: &mut [u32], width: usize, height: usize) {
    for y in 0..height {
        for x in 0..width {
            let blue = 0x30_u32.saturating_add((y as u32).saturating_mul(0x28) / height as u32);
            let green = 0x1A_u32.saturating_add((x as u32).saturating_mul(0x1A) / width as u32);
            let red = 0x08_u32.saturating_add((x as u32).saturating_mul(0x12) / width as u32);
            frame[y * width + x] = 0xFF00_0000 | (red << 16) | (green << 8) | blue;
        }
    }
}

fn paint_panel(frame: &mut [u32], width: usize, height: usize) {
    let panel_height = height.min(38);
    for y in 0..panel_height {
        for x in 0..width {
            let accent = if x % 80 < 6 { 0xFF9F_D356 } else { 0xFF13_1822 };
            frame[y * width + x] = accent;
        }
    }
}

fn paint_cursor(frame: &mut [u32], width: usize, height: usize, cursor_x: i32, cursor_y: i32) {
    let cursor_pattern = [
        "X...........",
        "XX..........",
        "XOX.........",
        "XOOX........",
        "XOOOX.......",
        "XOOOOX......",
        "XOOOOOX.....",
        "XOOOOOOX....",
        "XOOOOX......",
        "XOOXOX......",
        "XOX.XOX.....",
        "XX..XOX.....",
        "X....XOX....",
        ".....XOX....",
        "......XOX...",
        "......XOX...",
        ".......XX...",
    ];

    for (row_index, row) in cursor_pattern.iter().enumerate() {
        for (col_index, cell) in row.chars().enumerate() {
            let x = cursor_x.saturating_add(col_index as i32);
            let y = cursor_y.saturating_add(row_index as i32);
            if x < 0 || y < 0 {
                continue;
            }
            let x = x as usize;
            let y = y as usize;
            if x >= width || y >= height {
                continue;
            }
            match cell {
                'X' => frame[y * width + x] = 0xFF00_0000,
                'O' => frame[y * width + x] = 0xFFF5_F7_FA,
                _ => {}
            }
        }
    }
}
