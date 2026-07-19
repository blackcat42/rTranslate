use fltk::{
    app,
    dialog,
    prelude::*,
    window::DoubleWindow,
    enums,
};
use std::rc::Rc;
use std::cell::RefCell;

use crate::types::{
    BLWCoords
};

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
    let (ex, ey) = app::event_coords();
    let margin = 5; // border detection
    //.x() - inner coords
    //.x_root() - coords relative to screen
    let win_left = 0;
    let win_right = window.pixel_w();
    let win_top = 0;
    let win_bottom = window.pixel_h();
    
    match event {
        enums::Event::Push => {
            coords.x = ex;
            coords.y = ey;
            coords.x_start = app::event_x_root();
            coords.y_start = app::event_y_root();
            coords.initial_window_height = window.pixel_h();
            coords.initial_window_width = window.pixel_w();
            coords.init_on_border_left = ex < win_left + margin && ex > win_left;
            coords.init_on_border_right = ex > win_right - margin && ex < win_right;
            coords.init_on_border_top = ey < win_top + margin && ey > win_top;
            coords.init_on_border_bottom = ey > win_bottom - margin && ey < win_bottom;
            true
        }

        enums::Event::Drag => {
            if (
                (coords.x > 5) 
                && (coords.x < coords.initial_window_width - 5)) 
                && ((coords.y > 5) 
                && (coords.y < coords.initial_window_height - 5)
            ) {
                window.set_pos(app::event_x_root() - coords.x, app::event_y_root() - coords.y);
                app::redraw();
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

                if new_w < 400 { 
                    new_w = 400;
                    new_x = window.x_root();
                }
                if new_h < 150 { 
                    new_h = 150;
                    new_y = window.y_root();
                }
                window.resize(new_x, new_y, new_w, new_h);
            }
            true
        }

        enums::Event::Move | enums::Event::Enter => {
            if !(is_inner) {
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