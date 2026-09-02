//! Генерация RGBA-иконки батареи (22×22, template-ready).
//!
//! Цвет меняется по уровню: зелёный → оранжевый → красный.

use image::{Rgba, RgbaImage};

const SIZE: u32 = 22;

pub fn battery_color(percent: u8) -> Rgba<u8> {
    match percent {
        0..=20 => Rgba([255, 59, 48, 255]),
        21..=50 => Rgba([255, 149, 0, 255]),
        _ => Rgba([52, 199, 89, 255]),
    }
}

pub fn cpu_color(usage: u8) -> Rgba<u8> {
    match usage {
        0..=40 => Rgba([52, 199, 89, 255]),
        41..=75 => Rgba([255, 149, 0, 255]),
        _ => Rgba([255, 59, 48, 255]),
    }
}

pub fn make_battery_icon(fill_percent: u8, color: Rgba<u8>) -> tray_icon::Icon {
    let mut img = RgbaImage::new(SIZE, SIZE);
    let fill = fill_percent.clamp(0, 100) as f32 / 100.0;

    let body_left = 4;
    let body_right = 16;
    let body_top = 6;
    let body_bottom = 16;
    let cap_left = 17;
    let cap_right = 19;
    let cap_top = 9;
    let cap_bottom = 13;

    for y in 0..SIZE {
        for x in 0..SIZE {
            let px = x as i32;
            let py = y as i32;

            let is_outline = (px >= body_left && px <= body_right && (py == body_top || py == body_bottom))
                || (py >= body_top && py <= body_bottom && (px == body_left || px == body_right))
                || (px >= cap_left && px <= cap_right && py >= cap_top && py <= cap_bottom);

            let inner_left = body_left + 2;
            let inner_right = (body_left as f32 + (body_right - body_left - 1) as f32 * fill) as i32;
            let is_fill = px >= inner_left
                && px <= inner_right
                && py >= body_top + 2
                && py <= body_bottom - 2;

            if is_fill {
                img.put_pixel(x, y, color);
            } else if is_outline {
                img.put_pixel(x, y, Rgba([255, 255, 255, 220]));
            } else {
                img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            }
        }
    }

    let (w, h) = img.dimensions();
    let rgba = img.into_raw();
    tray_icon::Icon::from_rgba(rgba, w, h).expect("valid icon rgba")
}

const MENU_ICON_SIZE: u32 = 16;

const GREEN: Rgba<u8> = Rgba([52, 199, 89, 255]);
const RED: Rgba<u8> = Rgba([255, 59, 48, 255]);

pub fn menu_check_green() -> muda::Icon {
    menu_status_icon(true)
}

pub fn menu_cross_red() -> muda::Icon {
    menu_status_icon(false)
}

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

fn is_check_pixel(x: i32, y: i32) -> bool {
    let short = (x + y == 14 || x + y == 15) && (3..=7).contains(&x) && (7..=11).contains(&y);
    let long = (x - y == 2 || x - y == 3) && (7..=12).contains(&x) && (5..=10).contains(&y);
    short || long
}

fn is_cross_pixel(x: i32, y: i32) -> bool {
    if !(4..=11).contains(&x) || !(4..=11).contains(&y) {
        return false;
    }
    (x - y).abs() <= 1 || (x + y - 15).abs() <= 1
}


