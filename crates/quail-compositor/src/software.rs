use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::apps::AppCategory;
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
    paint_launcher_surface(&mut pixels, width, height, state);
    paint_bottom_panel(&mut pixels, width, height, state);

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
            let blue = 0x18_u32.saturating_add((y as u32).saturating_mul(0x28) / height as u32);
            let green = 0x14_u32.saturating_add((x as u32).saturating_mul(0x14) / width as u32);
            let red = 0x0F_u32.saturating_add((x as u32).saturating_mul(0x0E) / width as u32);
            frame[y * width + x] = 0xFF00_0000 | (red << 16) | (green << 8) | blue;
        }
    }

    paint_cloud(
        frame,
        width,
        height,
        (width as i32) / 3,
        (height as i32) / 3,
        (width.min(height) as i32) / 4,
        0x441A233A,
    );
    paint_cloud(
        frame,
        width,
        height,
        (width as i32 * 3) / 4,
        (height as i32 * 2) / 5,
        (width.min(height) as i32) / 5,
        0x332D1C3B,
    );
    paint_cloud(
        frame,
        width,
        height,
        (width as i32 * 4) / 5,
        (height as i32 * 3) / 4,
        (width.min(height) as i32) / 4,
        0x3B261C38,
    );
    paint_glow(
        frame,
        width,
        height,
        width as i32 - 180,
        120,
        170,
        0x22E8A15B,
    );
    paint_glow(frame, width, height, 220, 150, 220, 0x1A59B6FF);
    paint_glow(
        frame,
        width,
        height,
        width as i32 / 2,
        height as i32 - 100,
        260,
        0x1C8A5BFF,
    );
    paint_light_streak(
        frame,
        width,
        height,
        width as i32 - 440,
        140,
        320,
        0x15FFFFFF,
    );
    paint_light_streak(
        frame,
        width,
        height,
        width as i32 - 620,
        height as i32 - 240,
        280,
        0x0EFFFFFF,
    );
}

fn paint_launcher_surface(frame: &mut [u32], width: usize, height: usize, state: &CompositorState) {
    let panel_width = width.min(780);
    let panel_height = height.min(620);
    let panel_x = 18;
    let panel_y = height.saturating_sub(panel_height + 78);

    fill_rounded_rect(
        frame,
        width,
        height,
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        20,
        0xE5161A21,
    );
    fill_rounded_rect(
        frame,
        width,
        height,
        panel_x + 1,
        panel_y + 1,
        panel_width.saturating_sub(2),
        58,
        20,
        0xF01C222C,
    );
    fill_rounded_rect(
        frame,
        width,
        height,
        panel_x + 278,
        panel_y + 14,
        panel_width.saturating_sub(334),
        34,
        11,
        0xFF262D37,
    );
    fill_rounded_rect(
        frame,
        width,
        height,
        panel_x + 28,
        panel_y + 16,
        26,
        26,
        13,
        0xFF293545,
    );
    fill_rounded_rect(
        frame,
        width,
        height,
        panel_x + 60,
        panel_y + 23,
        100,
        10,
        5,
        0xFFD7DFEA,
    );
    fill_rounded_rect(
        frame,
        width,
        height,
        panel_x + 302,
        panel_y + 26,
        134,
        8,
        4,
        0x889AA8B8,
    );
    fill_rounded_rect(
        frame,
        width,
        height,
        panel_x + panel_width - 62,
        panel_y + 20,
        18,
        18,
        9,
        0xFF2A3340,
    );
    fill_rounded_rect(
        frame,
        width,
        height,
        panel_x + panel_width - 34,
        panel_y + 20,
        18,
        18,
        9,
        0xFF2A3340,
    );

    let sidebar_width = 256;
    fill_rect(
        frame,
        width,
        height,
        panel_x + sidebar_width,
        panel_y + 60,
        1,
        panel_height.saturating_sub(118),
        0xFF2A313C,
    );
    fill_rounded_rect(
        frame,
        width,
        height,
        panel_x + 12,
        panel_y + 74,
        sidebar_width - 24,
        44,
        10,
        0xFF213040,
    );

    let sidebar_items = [
        ("Favorites", true),
        ("All Applications", false),
        ("Development", false),
        ("Graphics", false),
        ("Internet", false),
        ("Multimedia", false),
        ("Office", false),
        ("Settings", false),
    ];
    for (index, (_label, selected)) in sidebar_items.iter().enumerate() {
        let item_y = panel_y + 74 + index * 52;
        if *selected {
            fill_rounded_rect(
                frame,
                width,
                height,
                panel_x + 12,
                item_y,
                sidebar_width - 24,
                44,
                10,
                0xFF20384D,
            );
        }
        fill_rounded_rect(
            frame,
            width,
            height,
            panel_x + 24,
            item_y + 12,
            18,
            18,
            7,
            0xFF4C79A6,
        );
        let bar_width = if index == 1 { 142 } else { 110 };
        fill_rounded_rect(
            frame,
            width,
            height,
            panel_x + 54,
            item_y + 16,
            bar_width,
            9,
            4,
            0xFFC8D2DE,
        );
    }

    fill_rounded_rect(
        frame,
        width,
        height,
        panel_x + 18,
        panel_y + panel_height - 54,
        122,
        36,
        10,
        0xFF1E2430,
    );
    fill_rounded_rect(
        frame,
        width,
        height,
        panel_x + 148,
        panel_y + panel_height - 54,
        96,
        36,
        10,
        0xFF1A202B,
    );
    fill_rounded_rect(
        frame,
        width,
        height,
        panel_x + panel_width - 300,
        panel_y + panel_height - 54,
        86,
        36,
        10,
        0xFF1A202B,
    );
    fill_rounded_rect(
        frame,
        width,
        height,
        panel_x + panel_width - 204,
        panel_y + panel_height - 54,
        86,
        36,
        10,
        0xFF1A202B,
    );
    fill_rounded_rect(
        frame,
        width,
        height,
        panel_x + panel_width - 108,
        panel_y + panel_height - 54,
        90,
        36,
        10,
        0xFF251E22,
    );

    for (index, app) in state.installed_apps.iter().take(8).enumerate() {
        let col = index % 4;
        let row = index / 4;
        let tile_x = panel_x + sidebar_width + 28 + col * 116;
        let tile_y = panel_y + 86 + row * 128;
        let color = match app.category {
            AppCategory::Terminal => 0xFF4C6FFF,
            AppCategory::Browser => 0xFFFFA64D,
            AppCategory::Files => 0xFF4CBF8A,
            AppCategory::Editor => 0xFFB36DFF,
            AppCategory::Utility => 0xFF8FA3BA,
        };
        fill_rounded_rect(
            frame,
            width,
            height,
            tile_x,
            tile_y,
            96,
            102,
            14,
            if index == 0 { 0xFF24394E } else { 0xFF161B24 },
        );
        fill_rounded_rect(
            frame,
            width,
            height,
            tile_x + 22,
            tile_y + 14,
            50,
            50,
            16,
            color,
        );
        fill_rounded_rect(
            frame,
            width,
            height,
            tile_x + 34,
            tile_y + 26,
            26,
            26,
            9,
            0xFFF7FAFD,
        );
        let label_width = app.name.len().min(10).saturating_mul(6).max(46);
        fill_rounded_rect(
            frame,
            width,
            height,
            tile_x + 12,
            tile_y + 74,
            label_width,
            8,
            4,
            0xFFD8E0EA,
        );
        let label_width_secondary = app.command.len().min(9).saturating_mul(5).max(28);
        fill_rounded_rect(
            frame,
            width,
            height,
            tile_x + 12,
            tile_y + 88,
            label_width_secondary,
            6,
            3,
            0x667E8E9F,
        );
    }
}

fn paint_bottom_panel(frame: &mut [u32], width: usize, height: usize, state: &CompositorState) {
    let panel_height = 54;
    let panel_y = height.saturating_sub(panel_height);
    fill_rect(
        frame,
        width,
        height,
        0,
        panel_y,
        width,
        panel_height,
        0xEE131821,
    );
    fill_rect(frame, width, height, 0, panel_y, width, 1, 0xFF2B3240);

    for (index, app) in state.installed_apps.iter().take(6).enumerate() {
        let icon_x = 18 + index * 52;
        let color = match app.category {
            AppCategory::Terminal => 0xFF4C6FFF,
            AppCategory::Browser => 0xFFFFA64D,
            AppCategory::Files => 0xFF4CBF8A,
            AppCategory::Editor => 0xFFB36DFF,
            AppCategory::Utility => 0xFF8FA3BA,
        };
        fill_rounded_rect(
            frame,
            width,
            height,
            icon_x,
            panel_y + 9,
            36,
            36,
            10,
            0xFF202631,
        );
        fill_rounded_rect(
            frame,
            width,
            height,
            icon_x + 7,
            panel_y + 16,
            22,
            22,
            7,
            color,
        );
    }

    let mut indicator_x = width.saturating_sub(250);
    for _ in 0..8 {
        fill_rounded_rect(
            frame,
            width,
            height,
            indicator_x,
            panel_y + 18,
            16,
            16,
            5,
            0xFF697789,
        );
        indicator_x += 26;
    }
    fill_rounded_rect(
        frame,
        width,
        height,
        width.saturating_sub(80),
        panel_y + 16,
        54,
        20,
        7,
        0xFFD8E0EA,
    );
}

fn paint_cursor(frame: &mut [u32], width: usize, height: usize, cursor_x: i32, cursor_y: i32) {
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
                'S' => blend_pixel(&mut frame[y * width + x], 0x44000000),
                'X' => frame[y * width + x] = 0xFF0E1218,
                'O' => frame[y * width + x] = 0xFFF7FAFD,
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
    fill_rounded_rect(
        frame,
        width,
        height,
        x,
        y,
        window_width,
        window_height,
        16,
        0x6611161D,
    );
    fill_rounded_rect(
        frame,
        width,
        height,
        x + 3,
        y + 3,
        window_width.saturating_sub(6),
        window_height.saturating_sub(6),
        14,
        body_color,
    );
    fill_rounded_rect(
        frame,
        width,
        height,
        x + 3,
        y + 3,
        window_width.saturating_sub(6),
        34,
        14,
        title_color,
    );
    fill_rounded_rect(frame, width, height, x + 16, y + 14, 12, 12, 6, 0xFFDC5B64);
    fill_rounded_rect(frame, width, height, x + 34, y + 14, 12, 12, 6, 0xFFD6A448);
    fill_rounded_rect(frame, width, height, x + 52, y + 14, 12, 12, 6, 0xFF5FBC8D);
    let title_width = surface.window_title.len().min(24).saturating_mul(7).max(42);
    fill_rounded_rect(
        frame,
        width,
        height,
        x + 84,
        y + 14,
        title_width,
        10,
        5,
        if focused { 0xFFDFE6F1 } else { 0xAAB5C0CD },
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

fn fill_rounded_rect(
    frame: &mut [u32],
    frame_width: usize,
    frame_height: usize,
    x: usize,
    y: usize,
    rect_width: usize,
    rect_height: usize,
    radius: usize,
    color: u32,
) {
    let max_x = x.saturating_add(rect_width).min(frame_width);
    let max_y = y.saturating_add(rect_height).min(frame_height);
    let radius = radius.min(rect_width / 2).min(rect_height / 2);
    let radius_i32 = radius as i32;

    for draw_y in y.min(frame_height)..max_y {
        for draw_x in x.min(frame_width)..max_x {
            let local_x = draw_x.saturating_sub(x);
            let local_y = draw_y.saturating_sub(y);
            let inside = local_x >= radius
                || local_x + radius >= rect_width
                || local_y >= radius
                || local_y + radius >= rect_height
                || corner_distance(local_x, local_y, rect_width, rect_height, radius_i32);
            if inside {
                blend_pixel(&mut frame[draw_y * frame_width + draw_x], color);
            }
        }
    }
}

fn corner_distance(
    local_x: usize,
    local_y: usize,
    rect_width: usize,
    rect_height: usize,
    radius: i32,
) -> bool {
    let center_x = if local_x < radius as usize {
        radius
    } else {
        rect_width as i32 - radius - 1
    };
    let center_y = if local_y < radius as usize {
        radius
    } else {
        rect_height as i32 - radius - 1
    };
    let dx = local_x as i32 - center_x;
    let dy = local_y as i32 - center_y;
    dx.saturating_mul(dx) + dy.saturating_mul(dy) <= radius.saturating_mul(radius)
}

fn blend_pixel(destination: &mut u32, source: u32) {
    let alpha = ((source >> 24) & 0xFF) as u32;
    if alpha == 0 {
        return;
    }
    if alpha == 0xFF {
        *destination = source;
        return;
    }

    let inv_alpha = 0xFF_u32.saturating_sub(alpha);
    let dst = *destination;
    let src_r = (source >> 16) & 0xFF;
    let src_g = (source >> 8) & 0xFF;
    let src_b = source & 0xFF;
    let dst_r = (dst >> 16) & 0xFF;
    let dst_g = (dst >> 8) & 0xFF;
    let dst_b = dst & 0xFF;
    let red = (src_r * alpha + dst_r * inv_alpha) / 0xFF;
    let green = (src_g * alpha + dst_g * inv_alpha) / 0xFF;
    let blue = (src_b * alpha + dst_b * inv_alpha) / 0xFF;
    *destination = 0xFF00_0000 | (red << 16) | (green << 8) | blue;
}

fn paint_glow(
    frame: &mut [u32],
    frame_width: usize,
    frame_height: usize,
    center_x: i32,
    center_y: i32,
    radius: i32,
    color: u32,
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

            let alpha = ((radius_squared - distance) * ((color >> 24) & 0xFF) as i32
                / radius_squared.max(1)) as u32;
            let source = (alpha << 24) | (color & 0x00FF_FFFF);
            blend_pixel(&mut frame[y * frame_width + x], source);
        }
    }
}

fn paint_cloud(
    frame: &mut [u32],
    width: usize,
    height: usize,
    center_x: i32,
    center_y: i32,
    radius: i32,
    color: u32,
) {
    paint_glow(
        frame,
        width,
        height,
        center_x - radius / 2,
        center_y,
        radius,
        color,
    );
    paint_glow(
        frame,
        width,
        height,
        center_x + radius / 3,
        center_y - radius / 5,
        radius,
        color,
    );
    paint_glow(
        frame,
        width,
        height,
        center_x,
        center_y + radius / 4,
        radius,
        color,
    );
}

fn paint_light_streak(
    frame: &mut [u32],
    frame_width: usize,
    frame_height: usize,
    start_x: i32,
    start_y: i32,
    length: i32,
    color: u32,
) {
    for step in 0..length.max(0) {
        let x = start_x + step;
        let y = start_y + step / 3;
        if x < 0 || y < 0 {
            continue;
        }
        let x = x as usize;
        let y = y as usize;
        if x >= frame_width || y >= frame_height {
            continue;
        }
        for spread in 0..4 {
            let draw_y = y.saturating_add(spread);
            if draw_y >= frame_height {
                continue;
            }
            blend_pixel(&mut frame[draw_y * frame_width + x], color);
        }
    }
}
