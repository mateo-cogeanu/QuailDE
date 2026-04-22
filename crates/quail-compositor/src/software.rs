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

    let mut pixels = vec![0xFF1D365D_u32; width.saturating_mul(height)];
    let mut painted_surfaces = 0;

    paint_background(&mut pixels, width, height);
    paint_panel(&mut pixels, width, height);
    paint_status_area(&mut pixels, width, height);
    paint_dock(&mut pixels, width, height);
    paint_desktop_icons(&mut pixels, width, height);

    let mut ordered_surfaces = state.scene.surfaces.values().collect::<Vec<_>>();
    ordered_surfaces.sort_by_key(|surface| {
        let focused_rank = if state.focused_surface_id == Some(surface.object_id) {
            1
        } else {
            0
        };
        (focused_rank, surface.object_id)
    });

    for surface in ordered_surfaces {
        if let Some(buffer) = &surface.committed_buffer {
            paint_window_frame(
                &mut pixels,
                width,
                height,
                surface,
                state.focused_surface_id == Some(surface.object_id),
            );
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
            let blue = 0x72_u32.saturating_add((y as u32).saturating_mul(0x3A) / height as u32);
            let green = 0x55_u32.saturating_add((x as u32).saturating_mul(0x46) / width as u32);
            let red = 0x1D_u32.saturating_add((x as u32).saturating_mul(0x22) / width as u32);
            frame[y * width + x] = 0xFF00_0000 | (red << 16) | (green << 8) | blue;
        }
    }

    let orb_radius = (width.min(height) / 7).max(40) as i32;
    paint_glow(
        frame,
        width,
        height,
        width as i32 - orb_radius * 2,
        orb_radius * 2,
        orb_radius,
    );
    paint_glow(
        frame,
        width,
        height,
        orb_radius * 2,
        height as i32 - orb_radius * 2,
        orb_radius,
    );
}

fn paint_panel(frame: &mut [u32], width: usize, height: usize) {
    let panel_height = height.min(38);
    fill_rect(frame, width, height, 0, 0, width, panel_height, 0xEE1A2030);
    fill_rect(frame, width, height, 18, 10, 118, 18, 0xFF2B90D9);
    fill_rect(frame, width, height, 24, 14, 28, 10, 0xFFF4F7FB);
    fill_rect(frame, width, height, 62, 14, 46, 10, 0xFFBFE2FF);
}

fn paint_status_area(frame: &mut [u32], width: usize, height: usize) {
    let right = width.saturating_sub(18);
    fill_rect(
        frame,
        width,
        height,
        right.saturating_sub(172),
        10,
        154,
        18,
        0x99374259,
    );
    fill_rect(
        frame,
        width,
        height,
        right.saturating_sub(160),
        14,
        18,
        10,
        0xFFF2C14E,
    );
    fill_rect(
        frame,
        width,
        height,
        right.saturating_sub(132),
        14,
        18,
        10,
        0xFF67D5B5,
    );
    fill_rect(
        frame,
        width,
        height,
        right.saturating_sub(104),
        14,
        18,
        10,
        0xFFF26B6B,
    );
    fill_rect(
        frame,
        width,
        height,
        right.saturating_sub(70),
        14,
        44,
        10,
        0xFFDCE8F5,
    );
}

fn paint_dock(frame: &mut [u32], width: usize, height: usize) {
    let dock_width = width.min(340);
    let dock_height = 72;
    let dock_x = (width.saturating_sub(dock_width)) / 2;
    let dock_y = height.saturating_sub(dock_height + 18);

    fill_rect(
        frame,
        width,
        height,
        dock_x,
        dock_y,
        dock_width,
        dock_height,
        0xCC182132,
    );

    for index in 0..5 {
        let icon_x = dock_x + 24 + index * 62;
        fill_rect(
            frame,
            width,
            height,
            icon_x,
            dock_y + 14,
            44,
            44,
            match index {
                0 => 0xFF4F8EF7,
                1 => 0xFFFFB347,
                2 => 0xFF7CD992,
                3 => 0xFFF279A2,
                _ => 0xFFC2D3E8,
            },
        );
        fill_rect(
            frame,
            width,
            height,
            icon_x + 10,
            dock_y + 24,
            24,
            24,
            0xFFF6FAFF,
        );
    }
}

fn paint_desktop_icons(frame: &mut [u32], width: usize, height: usize) {
    for index in 0..3 {
        let icon_y = 82 + index * 96;
        fill_rect(frame, width, height, 28, icon_y, 56, 56, 0xAA223248);
        fill_rect(frame, width, height, 40, icon_y + 12, 32, 32, 0xFFE9F2FB);
        fill_rect(frame, width, height, 24, icon_y + 64, 64, 12, 0xAA1A2030);
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

fn paint_window_frame(
    frame: &mut [u32],
    width: usize,
    height: usize,
    surface: &SceneSurface,
    focused: bool,
) {
    let Some(buffer) = surface.committed_buffer.as_ref() else {
        return;
    };
    let x = surface.x.saturating_sub(6).max(0) as usize;
    let y = surface.y.saturating_sub(34).max(0) as usize;
    let window_width = usize::try_from(buffer.width.max(0))
        .unwrap_or(0)
        .saturating_add(12);
    let window_height = usize::try_from(buffer.height.max(0))
        .unwrap_or(0)
        .saturating_add(40);
    let body_color = if focused { 0xFFF5F7FB } else { 0xFFE6EBF1 };
    let title_color = if focused { 0xFF29538A } else { 0xFF46566E };
    fill_rect(
        frame,
        width,
        height,
        x,
        y,
        window_width,
        window_height,
        0x88202A38,
    );
    fill_rect(
        frame,
        width,
        height,
        x + 3,
        y + 3,
        window_width.saturating_sub(6),
        window_height.saturating_sub(6),
        body_color,
    );
    fill_rect(
        frame,
        width,
        height,
        x + 3,
        y + 3,
        window_width.saturating_sub(6),
        34,
        title_color,
    );
    fill_rect(frame, width, height, x + 16, y + 14, 12, 12, 0xFFFF6B6B);
    fill_rect(frame, width, height, x + 34, y + 14, 12, 12, 0xFFF2C14E);
    fill_rect(frame, width, height, x + 52, y + 14, 12, 12, 0xFF67D5B5);
    let title_width = surface.window_title.len().min(24).saturating_mul(7).max(42);
    fill_rect(
        frame,
        width,
        height,
        x + 84,
        y + 14,
        title_width,
        10,
        if focused { 0xFFDDEBFF } else { 0xFFD2DCE8 },
    );
}

fn fill_rect(
    frame: &mut [u32],
    frame_width: usize,
    frame_height: usize,
    x: usize,
    y: usize,
    rect_width: usize,
    rect_height: usize,
    color: u32,
) {
    let max_x = x.saturating_add(rect_width).min(frame_width);
    let max_y = y.saturating_add(rect_height).min(frame_height);
    for draw_y in y.min(frame_height)..max_y {
        for draw_x in x.min(frame_width)..max_x {
            frame[draw_y * frame_width + draw_x] = color;
        }
    }
}

fn paint_glow(
    frame: &mut [u32],
    frame_width: usize,
    frame_height: usize,
    center_x: i32,
    center_y: i32,
    radius: i32,
) {
    let min_x = (center_x - radius).max(0) as usize;
    let max_x = (center_x + radius).max(0) as usize;
    let min_y = (center_y - radius).max(0) as usize;
    let max_y = (center_y + radius).max(0) as usize;
    let radius_squared = radius.saturating_mul(radius);

    for y in min_y.min(frame_height)..max_y.min(frame_height) {
        for x in min_x.min(frame_width)..max_x.min(frame_width) {
            let dx = x as i32 - center_x;
            let dy = y as i32 - center_y;
            let distance = dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy));
            if distance >= radius_squared {
                continue;
            }

            let intensity = ((radius_squared - distance) * 90 / radius_squared.max(1)) as u32;
            let blue = 0x90_u32.saturating_add(intensity / 2).min(0xFF);
            let green = 0x7A_u32.saturating_add(intensity / 3).min(0xFF);
            let red = 0x38_u32.saturating_add(intensity / 5).min(0xFF);
            frame[y * frame_width + x] = 0xFF00_0000 | (red << 16) | (green << 8) | blue;
        }
    }
}
