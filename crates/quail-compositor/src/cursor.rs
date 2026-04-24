use std::env;
use std::fs;
use std::sync::OnceLock;

use xcursor::{CursorTheme, parser::parse_xcursor};

/// CursorImage stores a themed software cursor frame plus its hotspot.
#[derive(Debug, Clone)]
pub struct CursorImage {
    pub width: usize,
    pub height: usize,
    pub hotspot_x: usize,
    pub hotspot_y: usize,
    pub pixels: Vec<u32>,
}

static CURSOR_IMAGE: OnceLock<Option<CursorImage>> = OnceLock::new();

/// themed_cursor loads the user's preferred XCursor theme once and reuses it
/// for every software frame so the pointer matches the surrounding Linux DE.
pub fn themed_cursor() -> Option<&'static CursorImage> {
    CURSOR_IMAGE.get_or_init(load_themed_cursor).as_ref()
}

fn load_themed_cursor() -> Option<CursorImage> {
    let theme_name = env::var("XCURSOR_THEME").unwrap_or_else(|_| "Adwaita".to_string());
    let cursor_size = env::var("XCURSOR_SIZE")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(24);
    let icon_names = [
        "left_ptr",
        "default",
        "arrow",
        "top_left_arrow",
        "left-arrow",
    ];

    for name in icon_names {
        let theme = CursorTheme::load(&theme_name);
        let Some(path) = theme.load_icon(name) else {
            continue;
        };
        let Ok(bytes) = fs::read(path) else {
            continue;
        };
        let Some(images) = parse_xcursor(&bytes) else {
            continue;
        };
        let Some(image) = choose_best_cursor_image(&images, cursor_size) else {
            continue;
        };

        let width = usize::try_from(image.width).ok()?;
        let height = usize::try_from(image.height).ok()?;
        let hotspot_x = usize::try_from(image.xhot).ok()?;
        let hotspot_y = usize::try_from(image.yhot).ok()?;
        let mut pixels = Vec::with_capacity(width.saturating_mul(height));
        for argb in image.pixels_argb.chunks_exact(4) {
            let alpha = u32::from(argb[0]);
            let red = u32::from(argb[1]);
            let green = u32::from(argb[2]);
            let blue = u32::from(argb[3]);
            pixels.push((alpha << 24) | (red << 16) | (green << 8) | blue);
        }

        return Some(CursorImage {
            width,
            height,
            hotspot_x,
            hotspot_y,
            pixels,
        });
    }

    None
}

fn choose_best_cursor_image(
    images: &[xcursor::parser::Image],
    size: u32,
) -> Option<xcursor::parser::Image> {
    images
        .iter()
        .min_by_key(|image| image.size.abs_diff(size))
        .cloned()
}
