use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use ab_glyph::{Font, FontArc, Glyph, PxScale, ScaleFont, point};

/// Canvas owns the immediate-mode software drawing helpers the shell uses.
pub struct Canvas<'a> {
    pub pixels: &'a mut [u32],
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone)]
struct IconBitmap {
    width: usize,
    height: usize,
    pixels: Vec<u32>,
}

static SYSTEM_FONT: OnceLock<Option<FontArc>> = OnceLock::new();
static ICON_CACHE: OnceLock<Mutex<HashMap<String, Option<IconBitmap>>>> = OnceLock::new();

impl<'a> Canvas<'a> {
    /// fill_rect paints an opaque or alpha-blended rectangle.
    pub fn fill_rect(
        &mut self,
        x: usize,
        y: usize,
        rect_width: usize,
        rect_height: usize,
        color: u32,
    ) {
        let max_x = x.saturating_add(rect_width).min(self.width);
        let max_y = y.saturating_add(rect_height).min(self.height);
        for draw_y in y.min(self.height)..max_y {
            for draw_x in x.min(self.width)..max_x {
                self.blend_pixel(draw_x, draw_y, color);
            }
        }
    }

    /// fill_rounded_rect gives the shell softer geometry than raw hard-edged boxes.
    pub fn fill_rounded_rect(
        &mut self,
        x: usize,
        y: usize,
        rect_width: usize,
        rect_height: usize,
        radius: usize,
        color: u32,
    ) {
        let max_x = x.saturating_add(rect_width).min(self.width);
        let max_y = y.saturating_add(rect_height).min(self.height);
        let radius = radius.min(rect_width / 2).min(rect_height / 2);
        let radius_i32 = radius as i32;

        for draw_y in y.min(self.height)..max_y {
            for draw_x in x.min(self.width)..max_x {
                let local_x = draw_x.saturating_sub(x);
                let local_y = draw_y.saturating_sub(y);
                let inside = local_x >= radius
                    || local_x + radius >= rect_width
                    || local_y >= radius
                    || local_y + radius >= rect_height
                    || corner_distance(local_x, local_y, rect_width, rect_height, radius_i32);
                if inside {
                    self.blend_pixel(draw_x, draw_y, color);
                }
            }
        }
    }

    /// text draws anti-aliased glyphs from a real system font when available.
    pub fn text(&mut self, x: f32, baseline_y: f32, size: f32, color: u32, content: &str) {
        let Some(font) = system_font() else {
            return;
        };
        let scale = PxScale::from(size);
        let scaled = font.as_scaled(scale);
        let mut caret_x = x;

        for ch in content.chars() {
            let glyph_id = scaled.glyph_id(ch);
            let glyph = Glyph {
                id: glyph_id,
                scale,
                position: point(caret_x, baseline_y),
            };
            if let Some(outlined) = font.outline_glyph(glyph.clone()) {
                let bounds = outlined.px_bounds();
                outlined.draw(|gx, gy, coverage| {
                    let draw_x = bounds.min.x as i32 + gx as i32;
                    let draw_y = bounds.min.y as i32 + gy as i32;
                    if draw_x < 0 || draw_y < 0 {
                        return;
                    }
                    let draw_x = draw_x as usize;
                    let draw_y = draw_y as usize;
                    if draw_x >= self.width || draw_y >= self.height {
                        return;
                    }
                    let alpha = (((color >> 24) & 0xFF) as f32 * coverage).round() as u32;
                    let shaded = (alpha << 24) | (color & 0x00FF_FFFF);
                    self.blend_pixel(draw_x, draw_y, shaded);
                });
            }
            caret_x += scaled.h_advance(glyph_id);
        }
    }

    /// icon resolves and paints a system icon theme bitmap when one exists.
    pub fn icon(&mut self, icon_name: &str, x: usize, y: usize, width: usize, height: usize) {
        let Some(bitmap) = load_icon(icon_name) else {
            return;
        };
        if bitmap.width == 0 || bitmap.height == 0 || width == 0 || height == 0 {
            return;
        }

        for draw_y in 0..height {
            let src_y = draw_y.saturating_mul(bitmap.height) / height;
            for draw_x in 0..width {
                let src_x = draw_x.saturating_mul(bitmap.width) / width;
                let pixel = bitmap.pixels[src_y * bitmap.width + src_x];
                let dst_x = x.saturating_add(draw_x);
                let dst_y = y.saturating_add(draw_y);
                if dst_x < self.width && dst_y < self.height {
                    self.blend_pixel(dst_x, dst_y, pixel);
                }
            }
        }
    }

    /// image paints an ARGB bitmap into the frame, optionally with fractional
    /// positioning so the cursor can move smoothly even on coarse VM tablets.
    pub fn image(
        &mut self,
        pixels: &[u32],
        image_width: usize,
        image_height: usize,
        x: f32,
        y: f32,
    ) {
        if image_width == 0 || image_height == 0 {
            return;
        }
        let base_x = x.floor() as i32;
        let base_y = y.floor() as i32;
        let frac_x = x - base_x as f32;
        let frac_y = y - base_y as f32;

        for src_y in 0..image_height {
            for src_x in 0..image_width {
                let pixel = pixels[src_y * image_width + src_x];
                if pixel >> 24 == 0 {
                    continue;
                }
                let draw_x = base_x + src_x as i32;
                let draw_y = base_y + src_y as i32;
                self.blend_pixel_fractional(draw_x, draw_y, frac_x, frac_y, pixel);
            }
        }
    }

    /// glow adds a soft radial highlight to the wallpaper or shell chrome.
    pub fn glow(&mut self, center_x: i32, center_y: i32, radius: i32, color: u32) {
        let min_x = (center_x - radius).max(0) as usize;
        let max_x = (center_x + radius).max(0) as usize;
        let min_y = (center_y - radius).max(0) as usize;
        let max_y = (center_y + radius).max(0) as usize;
        let radius_squared = radius.saturating_mul(radius);

        for draw_y in min_y.min(self.height)..max_y.min(self.height) {
            for draw_x in min_x.min(self.width)..max_x.min(self.width) {
                let dx = draw_x as i32 - center_x;
                let dy = draw_y as i32 - center_y;
                let distance = dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy));
                if distance >= radius_squared {
                    continue;
                }
                let alpha = ((radius_squared - distance) * ((color >> 24) & 0xFF) as i32
                    / radius_squared.max(1)) as u32;
                let shaded = (alpha << 24) | (color & 0x00FF_FFFF);
                self.blend_pixel(draw_x, draw_y, shaded);
            }
        }
    }

    /// light_streak draws a soft diagonal streak for the wallpaper.
    pub fn light_streak(&mut self, start_x: i32, start_y: i32, length: i32, color: u32) {
        for step in 0..length.max(0) {
            let x = start_x + step;
            let y = start_y + step / 3;
            if x < 0 || y < 0 {
                continue;
            }
            let x = x as usize;
            let y = y as usize;
            if x >= self.width || y >= self.height {
                continue;
            }
            for spread in 0..4 {
                let draw_y = y.saturating_add(spread);
                if draw_y < self.height {
                    self.blend_pixel(x, draw_y, color);
                }
            }
        }
    }

    /// blend_pixel alpha-blends a single source pixel into the software frame.
    pub fn blend_pixel(&mut self, x: usize, y: usize, source: u32) {
        let destination = &mut self.pixels[y * self.width + x];
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

    fn blend_pixel_fractional(
        &mut self,
        draw_x: i32,
        draw_y: i32,
        frac_x: f32,
        frac_y: f32,
        source: u32,
    ) {
        let weights = [
            ((1.0 - frac_x) * (1.0 - frac_y), 0_i32, 0_i32),
            (frac_x * (1.0 - frac_y), 1_i32, 0_i32),
            ((1.0 - frac_x) * frac_y, 0_i32, 1_i32),
            (frac_x * frac_y, 1_i32, 1_i32),
        ];

        for (weight, offset_x, offset_y) in weights {
            if weight <= 0.0 {
                continue;
            }
            let target_x = draw_x + offset_x;
            let target_y = draw_y + offset_y;
            if target_x < 0 || target_y < 0 {
                continue;
            }
            let target_x = target_x as usize;
            let target_y = target_y as usize;
            if target_x >= self.width || target_y >= self.height {
                continue;
            }
            let scaled_alpha = ((((source >> 24) & 0xFF) as f32) * weight).round() as u32;
            let shaded = (scaled_alpha << 24) | (source & 0x00FF_FFFF);
            self.blend_pixel(target_x, target_y, shaded);
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

fn system_font() -> Option<&'static FontArc> {
    SYSTEM_FONT
        .get_or_init(|| {
            let candidates = [
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
                "/usr/share/fonts/liberation2/LiberationSans-Regular.ttf",
                "/usr/share/fonts/TTF/DejaVuSans.ttf",
            ];
            for path in candidates {
                if let Ok(bytes) = fs::read(path)
                    && let Ok(font) = FontArc::try_from_vec(bytes)
                {
                    return Some(font);
                }
            }
            None
        })
        .as_ref()
}

fn load_icon(icon_name: &str) -> Option<IconBitmap> {
    let cache = ICON_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(cache) = cache.lock()
        && let Some(bitmap) = cache.get(icon_name)
    {
        return bitmap.clone();
    }

    let bitmap = resolve_icon(icon_name).and_then(|path| load_icon_file(&path));
    if let Ok(mut cache) = cache.lock() {
        cache.insert(icon_name.to_string(), bitmap.clone());
    }
    bitmap
}

fn resolve_icon(icon_name: &str) -> Option<PathBuf> {
    let path = PathBuf::from(icon_name);
    if path.is_file() {
        return Some(path);
    }

    let icon_dirs = [
        "/usr/share/icons/Adwaita",
        "/usr/share/icons/hicolor",
        "/usr/share/pixmaps",
        "/usr/local/share/icons/hicolor",
    ];
    let sizes = [
        "256x256", "128x128", "96x96", "64x64", "48x48", "32x32", "24x24", "22x22", "16x16",
    ];
    let categories = [
        "apps",
        "categories",
        "places",
        "devices",
        "mimetypes",
        "actions",
    ];

    let mut candidates = Vec::new();
    if icon_name.contains('.') {
        candidates.push(icon_name.to_string());
    } else {
        candidates.push(format!("{icon_name}.png"));
        candidates.push(format!("{icon_name}.xpm"));
        candidates.push(icon_name.to_string());
    }

    for root in icon_dirs {
        for candidate in &candidates {
            let direct = Path::new(root).join(candidate);
            if direct.is_file() {
                return Some(direct);
            }
        }
        for size in sizes {
            for category in categories {
                for candidate in &candidates {
                    let themed = Path::new(root).join(size).join(category).join(candidate);
                    if themed.is_file() {
                        return Some(themed);
                    }
                }
            }
        }
    }

    None
}

fn load_icon_file(path: &Path) -> Option<IconBitmap> {
    let image = image::ImageReader::open(path)
        .ok()?
        .decode()
        .ok()?
        .to_rgba8();
    let width = usize::try_from(image.width()).ok()?;
    let height = usize::try_from(image.height()).ok()?;
    let mut pixels = Vec::with_capacity(width.saturating_mul(height));
    for pixel in image.pixels() {
        let [red, green, blue, alpha] = pixel.0;
        pixels.push(
            (u32::from(alpha) << 24)
                | (u32::from(red) << 16)
                | (u32::from(green) << 8)
                | u32::from(blue),
        );
    }
    Some(IconBitmap {
        width,
        height,
        pixels,
    })
}
