use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::apps::AppCategory;
use crate::cursor::themed_cursor;
use crate::render::Canvas;
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

/// compose_scene paints the committed scene into an in-memory XRGB8888 frame.
pub fn compose_scene(state: &mut CompositorState) -> SoftwareFrame {
    let width = usize::try_from(state.composed_width).unwrap_or(1280);
    let height = usize::try_from(state.composed_height).unwrap_or(720);
    let mut pixels = vec![0xFF101319_u32; width.saturating_mul(height)];
    let mut painted_surfaces = 0;

    {
        let mut canvas = Canvas {
            pixels: &mut pixels,
            width,
            height,
        };
        paint_background(&mut canvas);
        paint_bottom_panel(&mut canvas, state);
        if state.launcher_open {
            paint_launcher_surface(&mut canvas, state);
        }
    }

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
        let mut canvas = Canvas {
            pixels: &mut pixels,
            width,
            height,
        };
        paint_cursor(&mut canvas, state.cursor_x_precise, state.cursor_y_precise);
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

/// write_ppm exports the current composed frame to a simple binary PPM image.
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

fn paint_background(canvas: &mut Canvas<'_>) {
    for y in 0..canvas.height {
        for x in 0..canvas.width {
            let blue =
                0x18_u32.saturating_add((y as u32).saturating_mul(0x28) / canvas.height as u32);
            let green =
                0x14_u32.saturating_add((x as u32).saturating_mul(0x14) / canvas.width as u32);
            let red =
                0x0F_u32.saturating_add((x as u32).saturating_mul(0x0E) / canvas.width as u32);
            canvas.pixels[y * canvas.width + x] = 0xFF00_0000 | (red << 16) | (green << 8) | blue;
        }
    }

    paint_cloud(
        canvas,
        canvas.width as i32 / 3,
        canvas.height as i32 / 3,
        (canvas.width.min(canvas.height) as i32) / 4,
        0x441A233A,
    );
    paint_cloud(
        canvas,
        canvas.width as i32 * 3 / 4,
        canvas.height as i32 * 2 / 5,
        (canvas.width.min(canvas.height) as i32) / 5,
        0x332D1C3B,
    );
    paint_cloud(
        canvas,
        canvas.width as i32 * 4 / 5,
        canvas.height as i32 * 3 / 4,
        (canvas.width.min(canvas.height) as i32) / 4,
        0x3B261C38,
    );
    canvas.glow(canvas.width as i32 - 180, 120, 170, 0x22E8A15B);
    canvas.glow(220, 150, 220, 0x1A59B6FF);
    canvas.glow(
        canvas.width as i32 / 2,
        canvas.height as i32 - 100,
        260,
        0x1C8A5BFF,
    );
    canvas.light_streak(canvas.width as i32 - 440, 140, 320, 0x15FFFFFF);
    canvas.light_streak(
        canvas.width as i32 - 620,
        canvas.height as i32 - 240,
        280,
        0x0EFFFFFF,
    );
}

fn paint_launcher_surface(canvas: &mut Canvas<'_>, state: &CompositorState) {
    let panel_width = canvas.width.min(780);
    let panel_height = canvas.height.min(620);
    let panel_x = 18;
    let panel_y = canvas.height.saturating_sub(panel_height + 78);
    let sidebar_width = 256;
    let visible_entries = state.visible_launcher_entries();

    canvas.fill_rounded_rect(
        panel_x + 8,
        panel_y + 10,
        panel_width,
        panel_height,
        24,
        0x4410161E,
    );
    canvas.fill_rounded_rect(panel_x, panel_y, panel_width, panel_height, 20, 0xE5161A21);
    canvas.fill_rounded_rect(
        panel_x + 1,
        panel_y + 1,
        panel_width.saturating_sub(2),
        58,
        20,
        0xF01C222C,
    );
    canvas.fill_rect(panel_x, panel_y + 58, panel_width, 1, 0xFF2B3240);
    canvas.fill_rounded_rect(
        panel_x + 278,
        panel_y + 14,
        panel_width.saturating_sub(334),
        34,
        11,
        0xFF262D37,
    );
    canvas.fill_rounded_rect(panel_x + 28, panel_y + 16, 26, 26, 13, 0xFF293545);
    canvas.text(
        (panel_x + 60) as f32,
        (panel_y + 34) as f32,
        22.0,
        0xFFD7DFEA,
        "QuailDE",
    );
    canvas.text(
        (panel_x + 302) as f32,
        (panel_y + 35) as f32,
        18.0,
        0xAA9AA8B8,
        "Search applications...",
    );
    canvas.fill_rounded_rect(
        panel_x + panel_width - 62,
        panel_y + 20,
        18,
        18,
        9,
        0xFF2A3340,
    );
    canvas.fill_rounded_rect(
        panel_x + panel_width - 34,
        panel_y + 20,
        18,
        18,
        9,
        0xFF2A3340,
    );
    canvas.fill_rect(
        panel_x + sidebar_width,
        panel_y + 60,
        1,
        panel_height.saturating_sub(118),
        0xFF2A313C,
    );

    for (index, section) in state.launcher.sections.iter().enumerate() {
        let item_y = panel_y + 74 + index * 52;
        if index == state.launcher_selected_section {
            canvas.fill_rounded_rect(panel_x + 12, item_y, sidebar_width - 24, 44, 10, 0xFF20384D);
        }
        canvas.fill_rounded_rect(panel_x + 24, item_y + 12, 18, 18, 7, 0xFF4C79A6);
        canvas.text(
            (panel_x + 54) as f32,
            (item_y + 28) as f32,
            18.0,
            if index == state.launcher_selected_section {
                0xFFF4F7FB
            } else {
                0xFFC8D2DE
            },
            &section.label,
        );
    }

    canvas.fill_rounded_rect(
        panel_x + 18,
        panel_y + panel_height - 54,
        122,
        36,
        10,
        0xFF1E2430,
    );
    canvas.fill_rounded_rect(
        panel_x + 148,
        panel_y + panel_height - 54,
        96,
        36,
        10,
        0xFF1A202B,
    );
    canvas.fill_rounded_rect(
        panel_x + panel_width - 300,
        panel_y + panel_height - 54,
        86,
        36,
        10,
        0xFF1A202B,
    );
    canvas.fill_rounded_rect(
        panel_x + panel_width - 204,
        panel_y + panel_height - 54,
        86,
        36,
        10,
        0xFF1A202B,
    );
    canvas.fill_rounded_rect(
        panel_x + panel_width - 108,
        panel_y + panel_height - 54,
        90,
        36,
        10,
        0xFF251E22,
    );
    canvas.text(
        (panel_x + 32) as f32,
        (panel_y + panel_height - 31) as f32,
        16.0,
        0xFFD9E0EA,
        "Applications",
    );
    canvas.text(
        (panel_x + 164) as f32,
        (panel_y + panel_height - 31) as f32,
        16.0,
        0xFFAEB9C6,
        "Places",
    );
    canvas.text(
        (panel_x + panel_width - 284) as f32,
        (panel_y + panel_height - 31) as f32,
        15.0,
        0xFFAEB9C6,
        "Sleep",
    );
    canvas.text(
        (panel_x + panel_width - 188) as f32,
        (panel_y + panel_height - 31) as f32,
        15.0,
        0xFFAEB9C6,
        "Restart",
    );
    canvas.text(
        (panel_x + panel_width - 90) as f32,
        (panel_y + panel_height - 31) as f32,
        15.0,
        0xFFE6CACE,
        "Shut Down",
    );

    for (index, entry) in visible_entries.into_iter().take(8).enumerate() {
        let col = index % 4;
        let row = index / 4;
        let tile_x = panel_x + sidebar_width + 28 + col * 116;
        let tile_y = panel_y + 86 + row * 128;
        let color = match entry.category {
            AppCategory::Terminal => 0xFF4C6FFF,
            AppCategory::Browser => 0xFFFFA64D,
            AppCategory::Files => 0xFF4CBF8A,
            AppCategory::Editor => 0xFFB36DFF,
            AppCategory::Utility => 0xFF8FA3BA,
        };
        canvas.fill_rounded_rect(
            tile_x,
            tile_y,
            96,
            102,
            14,
            if index == 0 { 0xFF24394E } else { 0xFF161B24 },
        );
        canvas.fill_rounded_rect(tile_x + 1, tile_y + 1, 94, 100, 13, 0x14FF_FFFF);
        canvas.fill_rounded_rect(tile_x + 22, tile_y + 14, 50, 50, 16, color);
        canvas.icon(&entry.icon_name, tile_x + 28, tile_y + 20, 38, 38);
        canvas.text(
            (tile_x + 10) as f32,
            (tile_y + 83) as f32,
            15.0,
            0xFFD8E0EA,
            &entry.label,
        );
        canvas.text(
            (tile_x + 10) as f32,
            (tile_y + 97) as f32,
            12.0,
            0x887E8E9F,
            &entry.subtitle,
        );
    }
}

fn paint_bottom_panel(canvas: &mut Canvas<'_>, state: &CompositorState) {
    let panel_height = 54;
    let panel_y = canvas.height.saturating_sub(panel_height);
    canvas.fill_rect(0, panel_y, canvas.width, panel_height, 0xEE131821);
    canvas.fill_rect(0, panel_y, canvas.width, 1, 0xFF2B3240);
    canvas.fill_rounded_rect(
        12,
        panel_y + 7,
        40,
        40,
        12,
        if state.launcher_open {
            0xFF20384D
        } else {
            0xFF202631
        },
    );
    canvas.fill_rounded_rect(22, panel_y + 17, 8, 8, 4, 0xFF59B6FF);
    canvas.fill_rounded_rect(34, panel_y + 17, 8, 8, 4, 0xFFFFA64D);
    canvas.fill_rounded_rect(22, panel_y + 29, 8, 8, 4, 0xFF4CBF8A);
    canvas.fill_rounded_rect(34, panel_y + 29, 8, 8, 4, 0xFFB36DFF);

    for (index, entry) in state.launcher.entries.iter().take(6).enumerate() {
        let icon_x = 68 + index * 52;
        let color = match entry.category {
            AppCategory::Terminal => 0xFF4C6FFF,
            AppCategory::Browser => 0xFFFFA64D,
            AppCategory::Files => 0xFF4CBF8A,
            AppCategory::Editor => 0xFFB36DFF,
            AppCategory::Utility => 0xFF8FA3BA,
        };
        canvas.fill_rounded_rect(icon_x, panel_y + 9, 36, 36, 10, 0xFF202631);
        canvas.fill_rounded_rect(icon_x + 7, panel_y + 16, 22, 22, 7, color);
        canvas.icon(&entry.icon_name, icon_x + 5, panel_y + 14, 26, 26);
    }

    let mut indicator_x = canvas.width.saturating_sub(250);
    for _ in 0..8 {
        canvas.fill_rounded_rect(indicator_x, panel_y + 18, 16, 16, 5, 0xFF697789);
        indicator_x += 26;
    }
    canvas.text(
        (canvas.width.saturating_sub(92)) as f32,
        (panel_y + 34) as f32,
        18.0,
        0xFFD8E0EA,
        "10:42",
    );
}

fn paint_cursor(canvas: &mut Canvas<'_>, cursor_x: f32, cursor_y: f32) {
    if let Some(cursor) = themed_cursor() {
        let draw_x = cursor_x - cursor.hotspot_x as f32;
        let draw_y = cursor_y - cursor.hotspot_y as f32;
        canvas.image(&cursor.pixels, cursor.width, cursor.height, draw_x, draw_y);
        return;
    }

    let cursor_pattern = [
        "SS................",
        "SXX...............",
        "SXXX..............",
        "SXOOX.............",
        "SXOOOX............",
        "SXOOOOX...........",
        "SXOOOOOX..........",
        "SXOOOOOOX.........",
        "SXOOOOOOOX........",
        "SXOOOOXOOX........",
        "SXOOOX.XOOX.......",
        "SXOOX..XOOX.......",
        "SXXX....XOOX......",
        "SX.......XOOX.....",
        ".........XOOX.....",
        "..........XX......",
    ];

    for (row_index, row) in cursor_pattern.iter().enumerate() {
        for (col_index, cell) in row.chars().enumerate() {
            let x = cursor_x.round() as i32 + col_index as i32;
            let y = cursor_y.round() as i32 + row_index as i32;
            if x < 0 || y < 0 {
                continue;
            }
            let x = x as usize;
            let y = y as usize;
            if x >= canvas.width || y >= canvas.height {
                continue;
            }
            match cell {
                'S' => canvas.blend_pixel(x, y, 0x44000000),
                'X' => canvas.pixels[y * canvas.width + x] = 0xFF0E1218,
                'O' => canvas.pixels[y * canvas.width + x] = 0xFFF7FAFD,
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
    let body_color = if focused { 0xFF131922 } else { 0xFF171D27 };
    let title_color = if focused { 0xFF1F2D40 } else { 0xFF202733 };

    let mut canvas = Canvas {
        pixels: frame,
        width,
        height,
    };
    canvas.fill_rounded_rect(x, y, window_width, window_height, 16, 0x6611161D);
    canvas.fill_rounded_rect(
        x + 3,
        y + 3,
        window_width.saturating_sub(6),
        window_height.saturating_sub(6),
        14,
        body_color,
    );
    canvas.fill_rounded_rect(
        x + 3,
        y + 3,
        window_width.saturating_sub(6),
        34,
        14,
        title_color,
    );
    canvas.fill_rounded_rect(x + 16, y + 14, 12, 12, 6, 0xFFDC5B64);
    canvas.fill_rounded_rect(x + 34, y + 14, 12, 12, 6, 0xFFD6A448);
    canvas.fill_rounded_rect(x + 52, y + 14, 12, 12, 6, 0xFF5FBC8D);
    canvas.text(
        (x + 84) as f32,
        (y + 24) as f32,
        14.0,
        if focused { 0xFFDFE6F1 } else { 0xAAB5C0CD },
        &surface.window_title,
    );
}

fn paint_cloud(canvas: &mut Canvas<'_>, center_x: i32, center_y: i32, radius: i32, color: u32) {
    canvas.glow(center_x - radius / 2, center_y, radius, color);
    canvas.glow(center_x + radius / 3, center_y - radius / 5, radius, color);
    canvas.glow(center_x, center_y + radius / 4, radius, color);
}
