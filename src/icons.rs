//! Генерация RGBA-иконок для menu bar и пунктов меню.

use image::{Rgba, RgbaImage};

const GREEN: Rgba<u8> = Rgba([52, 199, 89, 255]);
const YELLOW: Rgba<u8> = Rgba([255, 204, 0, 255]);
const RED: Rgba<u8> = Rgba([255, 59, 48, 255]);
const TEXT_COLOR: Rgba<u8> = Rgba([255, 255, 255, 255]);

const TRAY_W: u32 = 60;
const TRAY_H: u32 = 22;
const DOT_RADIUS: f32 = 4.5;
const TEXT_SCALE_NUM: i32 = 3;
const TEXT_SCALE_DEN: i32 = 2;
const GLYPH_W: i32 = 6;
const GLYPH_H: i32 = 10;
const GLYPH_SPACING: i32 = 2;

/// Иконка строки меню: «AINV» + индикатор-кружок (зелёный / жёлтый).
pub fn tray_ainv_icon(any_hosts_active: bool) -> tray_icon::Icon {
    let mut img = RgbaImage::new(TRAY_W, TRAY_H);
    let dot_color = if any_hosts_active { GREEN } else { YELLOW };

    let text_w = scaled_glyph_w() * 4 + GLYPH_SPACING * 3;
    let text_h = scaled_glyph_h();
    let text_y = ((TRAY_H as i32 - text_h) / 2).max(0);
    draw_text_ainv(&mut img, 0, text_y);

    let dot_x = text_w as f32 + DOT_RADIUS + 4.0;
    let dot_y = TRAY_H as f32 / 2.0;
    fill_circle(&mut img, dot_x, dot_y, DOT_RADIUS, dot_color);

    let (w, h) = img.dimensions();
    tray_icon::Icon::from_rgba(img.into_raw(), w, h).expect("valid tray icon rgba")
}

/// Ширина одного символа после масштабирования (×1.5).
fn scaled_glyph_w() -> i32 {
    GLYPH_W * TEXT_SCALE_NUM / TEXT_SCALE_DEN
}

/// Высота одного символа после масштабирования (×1.5).
fn scaled_glyph_h() -> i32 {
    GLYPH_H * TEXT_SCALE_NUM / TEXT_SCALE_DEN
}

/// Рисует текст «AINV» bitmap-шрифтом.
fn draw_text_ainv(img: &mut RgbaImage, x0: i32, y0: i32) {
    let glyphs = [glyph_a(), glyph_i(), glyph_n(), glyph_v()];
    let step = scaled_glyph_w() + GLYPH_SPACING;

    for (idx, glyph) in glyphs.iter().enumerate() {
        draw_glyph_scaled(img, glyph, x0 + idx as i32 * step, y0);
    }
}

/// Рисует один символ с масштабом ×1.5.
fn draw_glyph_scaled(img: &mut RgbaImage, glyph: &[[bool; 6]; 10], x0: i32, y0: i32) {
    for y in 0..GLYPH_H {
        for x in 0..GLYPH_W {
            if !glyph[y as usize][x as usize] {
                continue;
            }

            let x1 = x0 + x * TEXT_SCALE_NUM / TEXT_SCALE_DEN;
            let y1 = y0 + y * TEXT_SCALE_NUM / TEXT_SCALE_DEN;
            let x2 = x0 + (x + 1) * TEXT_SCALE_NUM / TEXT_SCALE_DEN;
            let y2 = y0 + (y + 1) * TEXT_SCALE_NUM / TEXT_SCALE_DEN;

            for py in y1..y2 {
                for px in x1..x2 {
                    put(img, px, py, TEXT_COLOR);
                }
            }
        }
    }
}

/// Ставит один пиксель, если координаты в пределах изображения.
fn put(img: &mut RgbaImage, x: i32, y: i32, color: Rgba<u8>) {
    if x >= 0 && y >= 0 {
        let (x, y) = (x as u32, y as u32);
        if x < img.width() && y < img.height() {
            img.put_pixel(x, y, color);
        }
    }
}

/// Заливает круг заданным цветом.
fn fill_circle(img: &mut RgbaImage, cx: f32, cy: f32, radius: f32, color: Rgba<u8>) {
    let r2 = radius * radius;
    for y in 0..img.height() {
        for x in 0..img.width() {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            if dx * dx + dy * dy <= r2 {
                img.put_pixel(x, y, color);
            }
        }
    }
}

/// Bitmap-глif буквы «A» (6×10).
fn glyph_a() -> [[bool; 6]; 10] {
    [
        [false, false, true, true, false, false],
        [false, true, false, false, true, false],
        [false, true, false, false, true, false],
        [false, true, true, true, true, false],
        [false, true, false, false, true, false],
        [false, true, false, false, true, false],
        [false, true, false, false, true, false],
        [false, false, false, false, false, false],
        [false, false, false, false, false, false],
        [false, false, false, false, false, false],
    ]
}

/// Bitmap-глif буквы «I» (6×10).
fn glyph_i() -> [[bool; 6]; 10] {
    [
        [false, false, true, true, false, false],
        [false, false, false, false, false, false],
        [false, false, true, true, false, false],
        [false, false, true, true, false, false],
        [false, false, true, true, false, false],
        [false, false, true, true, false, false],
        [false, false, true, true, false, false],
        [false, false, false, false, false, false],
        [false, false, false, false, false, false],
        [false, false, false, false, false, false],
    ]
}

/// Bitmap-глif буквы «N» (6×10).
fn glyph_n() -> [[bool; 6]; 10] {
    [
        [false, true, false, false, true, false],
        [false, true, true, false, true, false],
        [false, true, false, true, true, false],
        [false, true, false, false, true, false],
        [false, true, false, false, true, false],
        [false, true, false, false, true, false],
        [false, true, false, false, true, false],
        [false, false, false, false, false, false],
        [false, false, false, false, false, false],
        [false, false, false, false, false, false],
    ]
}

/// Bitmap-глif буквы «V» (6×10).
fn glyph_v() -> [[bool; 6]; 10] {
    [
        [false, true, false, false, true, false],
        [false, true, false, false, true, false],
        [false, true, false, false, true, false],
        [false, false, true, true, false, false],
        [false, false, true, true, false, false],
        [false, false, false, true, false, false],
        [false, false, false, true, false, false],
        [false, false, false, false, false, false],
        [false, false, false, false, false, false],
        [false, false, false, false, false, false],
    ]
}

// --- menu item status icons ---

const MENU_ICON_SIZE: u32 = 16;
pub fn menu_check_green() -> muda::Icon {
    menu_status_icon(true)
}

/// Красный крестик для неактивного hosts-пункта меню.
pub fn menu_cross_red() -> muda::Icon {
    menu_status_icon(false)
}

/// Строит иконку статуса (галочка или крестик) для пункта меню.
fn menu_status_icon(active: bool) -> muda::Icon {
    let mut img = RgbaImage::new(MENU_ICON_SIZE, MENU_ICON_SIZE);
    let color = if active { GREEN } else { RED };

    for y in 0..MENU_ICON_SIZE {
        for x in 0..MENU_ICON_SIZE {
            let px = x as i32;
            let py = y as i32;
            let pixel = if active {
                is_check_pixel(px, py)
            } else {
                is_cross_pixel(px, py)
            };
            img.put_pixel(
                x,
                y,
                if pixel { color } else { Rgba([0, 0, 0, 0]) },
            );
        }
    }

    let (w, h) = img.dimensions();
    muda::Icon::from_rgba(img.into_raw(), w, h).expect("valid menu icon rgba")
}

/// Определяет, входит ли пиксель в отрисовку галочки.
fn is_check_pixel(x: i32, y: i32) -> bool {
    let short = (x + y == 14 || x + y == 15) && (3..=7).contains(&x) && (7..=11).contains(&y);
    let long = (x - y == 2 || x - y == 3) && (7..=12).contains(&x) && (5..=10).contains(&y);
    short || long
}

/// Определяет, входит ли пиксель в отрисовку крестика.
fn is_cross_pixel(x: i32, y: i32) -> bool {
    if !(4..=11).contains(&x) || !(4..=11).contains(&y) {
        return false;
    }
    (x - y).abs() <= 1 || (x + y - 15).abs() <= 1
}

