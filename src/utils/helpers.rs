use debug_print::{debug_println as dprintln};
use fltk::{
    app,
    dialog,
    prelude::*,
    window::DoubleWindow,
    enums,
};
use std::io::Write;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{OnceLock, LazyLock};

use anyhow::{anyhow, Result};
use regex::Regex;

use crate::types::{
    BLWCoords
};
use super::GLOBAL_SETTINGS;

static WIN7_OR_GREATER: OnceLock<bool> = OnceLock::new();

pub fn is_win7_or_greater() -> bool {
    *WIN7_OR_GREATER.get_or_init(|| {
        unsafe { 
            let version = windows_sys::Win32::System::SystemInformation::GetVersion();
            let major = (version & 0xFF) as u8;
            let minor = ((version >> 8) & 0xFF) as u8;

            if major > 6 || (major == 6 && minor > 0) {
                //Windows 8: 6.2; Windows 7: 6.1
                true
            } else {
                //Windows Vista: 6.0; Windows XP Pro x64: 5.2; Windows XP: 5.1; Windows 2000: 5.0
                false
            }
        }
    })
}

pub static RE_UID: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[\w ]+$").unwrap()
});

pub fn app_message(e: &str) {
    let pos = screen_center();
    dialog::alert(pos.0 - 210, pos.1 - 40, e);
    
}

pub fn screen_center() -> (i32, i32) {
    (
        (app::screen_size().0 / 2.0) as i32,
        (app::screen_size().1 / 2.0) as i32,
    )
}

pub fn ui_scale(size: i32) -> i32 {
    (size as f32 * GLOBAL_SETTINGS.ui_scaling) as i32
}

pub fn parse_hex_to_rgb(hex: &str) -> Result<(u8, u8, u8), &'static str> {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return Err("hex.len() != 6");
    }

    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "R")?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "G")?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "B")?;
    Ok((r, g, b))
}



pub fn borderless_win_frame_handler(event: enums::Event, win_popup: &mut DoubleWindow, is_inner: &Rc<RefCell<bool>>) -> bool {
    match event {
        enums::Event::Enter => {
            win_popup.set_cursor(enums::Cursor::Default);
            *is_inner.borrow_mut() = true;
            true
        }
        enums::Event::Leave => {
            *is_inner.borrow_mut() = false;
            true
        }
        _ => false,
    }
}

pub fn borderless_win_handler(
    window: &mut DoubleWindow, 
    event: enums::Event, 
    coords: &mut BLWCoords, 
    is_inner: &Rc<RefCell<bool>>
) -> bool {

    let is_inner = *is_inner.borrow();
    let win_ppu = window.pixels_per_unit();
    let (ex, ey) = app::event_coords();
    let margin = (5_f32 * win_ppu) as i32; // border detection
    //.x() - inner coords
    //.x_root() - coords relative to screen
    let win_left = 0;
    let win_right = window.width();
    let win_top = 0;
    let win_bottom = window.height();

    match event {
        enums::Event::Push => {
            coords.x = ex;
            coords.y = ey;
            coords.x_start = app::event_x_root();
            coords.y_start = app::event_y_root();
            //println!("win_right_inner: {}", window.pixel_w());
            coords.initial_window_height = win_bottom;
            coords.initial_window_width = win_right;
            coords.init_on_border_left = ex < win_left + margin && ex > win_left;
            coords.init_on_border_right = ex > win_right - margin && ex < win_right;
            coords.init_on_border_top = ey < win_top + margin && ey > win_top;
            coords.init_on_border_bottom = ey > win_bottom - margin && ey < win_bottom;
            true
        }

        enums::Event::Drag => {
            if (
                (coords.x > (5_f32 * win_ppu) as i32) 
                && (coords.x < coords.initial_window_width - (5_f32 * win_ppu) as i32)) 
                && ((coords.y > (5_f32 * win_ppu) as i32) 
                && (coords.y < coords.initial_window_height - (5_f32 * win_ppu) as i32)
            ) {
                window.set_pos(app::event_x_root() - coords.x, app::event_y_root() - coords.y);
            } else {
                let mut new_w = coords.initial_window_width;
                let mut new_h = coords.initial_window_height;
                let mut new_x = window.x_root();
                let mut new_y = window.y_root();
                if coords.init_on_border_left {
                    new_w = coords.initial_window_width - (app::event_x_root() - coords.x_start);
                    new_x = app::event_x_root() - coords.x;
                } else if coords.init_on_border_right {
                    new_w = coords.initial_window_width + (app::event_x_root() - coords.x_start);
                }
                if coords.init_on_border_top {
                    new_h = coords.initial_window_height - (app::event_y_root() - coords.y_start);
                    new_y = app::event_y_root() - coords.y;
                } else if coords.init_on_border_bottom {
                    new_h = coords.initial_window_height + (app::event_y_root() - coords.y_start);
                }

                if new_w < (400_f32 * win_ppu) as i32 { 
                    new_w = (400_f32 * win_ppu) as i32;
                    new_x = window.x_root();
                }
                if new_h < (150_f32 * win_ppu) as i32 { 
                    new_h = (150_f32 * win_ppu) as i32;
                    new_y = window.y_root();
                }
                window.resize(new_x, new_y, new_w, new_h);
            }
            if win_ppu != 1.0 {
                window.redraw();
                app::flush();
            }
            true
        }

        enums::Event::Move | enums::Event::Enter => {
            if !(is_inner) {
                //println!("{} > {} - {} && ", ex, win_right, margin);
                let on_border_left = ex < win_left + margin && ex > win_left;
                let on_border_right = ex > win_right - margin && ex < win_right;
                let on_border_top = ey < win_top + margin && ey > win_top;
                let on_border_bottom = ey > win_bottom - margin && ey < win_bottom;

                if (on_border_left && on_border_bottom) || (on_border_right && on_border_top) {
                    window.set_cursor(enums::Cursor::NESW);
                } else if (on_border_right && on_border_bottom) || (on_border_left && on_border_top) {
                    window.set_cursor(enums::Cursor::NWSE);
                } else if on_border_left || on_border_right {
                    window.set_cursor(enums::Cursor::WE);
                } else if on_border_top || on_border_bottom {
                    window.set_cursor(enums::Cursor::NS);
                }
            }
            true
        }

        enums::Event::Leave => {
           window.set_cursor(enums::Cursor::Default);
            true
        }
        _ => false,
    }
}

fn google_tk_b(mut value: i64, pattern: &[u8]) ->  Result<i64> {
    let mut i = 0;

    while i < pattern.len().saturating_sub(2) {
        let shift = pattern[i + 2] as char;

        let shift = if shift >= 'a' {
            (shift as i64) - 87
        } else {
            shift.to_digit(10).ok_or(anyhow!("err"))? as i64
        };
        let shifted = if pattern[i + 1] as char == '+' {
            (value as u32 >> (shift % 32)) as i64
        } else {
            ((value as i32).wrapping_shl(shift as u32)) as i64
        };

        value = if pattern[i] as char == '+' {
            (value as i32).wrapping_add(shifted as i32) as i64
        } else {
            ((value as i32) ^ (shifted as i32)) as i64
        };

        i += 3;
    }

    Ok(value)
}

pub fn google_tk(input: &str, tkk: &str) -> Result<String> {
    if !GLOBAL_SETTINGS.use_google_token {
        return Err(anyhow!("err"));
    }
    let parts: Vec<&str> = tkk.split('.').collect();
    let h: i64 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let tk2: i64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

    let bytes = input.as_bytes();
    let mut a = h;

    for &b in bytes {
        a = a.wrapping_add(b as i64);
        a = google_tk_b(a, b"+-a^+6")?;
    }

    a = google_tk_b(a, b"+-3^+b+-f")?;
    a = ((a as i32) ^ (tk2 as i32)) as i64;
    a = (a as u32) as i64;
    a %= 1_000_000;

    let xor_h = (a as i32) ^ (h as i32);

    Ok(format!("{}.{}", a, xor_h))
}

pub fn extract_detected_lang(mut text: String) -> (Option<String>, String) {
    const PREFIX: &str = "<DETECTED_LANG=";

    let Some(start) = text.find(PREFIX) else {
        return (None, text);
    };

    let value_start = start + PREFIX.len();

    let Some(relative_end) = text[value_start..].find('>') else {
        return (None, text);
    };

    let end = value_start + relative_end;
    let language = text[value_start..end].to_owned();

    text.replace_range(start..=end, "");

    (Some(language), text)
}

pub fn date_from_duration(d: std::time::Duration) -> (i64, u32, u32) {        
    let days = (d.as_secs() / 86400) as i64;
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;

    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    (year, m as u32, d as u32)
}

pub fn try_copy_exclusive(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    dprintln!("{}", src.to_string_lossy());
    dprintln!("{}", dst.to_string_lossy());
    //return Ok(());
    match std::fs::OpenOptions::new().write(true).create_new(true).open(dst) {
        Ok(mut dst_file) => {
            let mut src_file = std::fs::File::open(src)?;
            std::io::copy(&mut src_file, &mut dst_file)?;
            dst_file.flush()?;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
