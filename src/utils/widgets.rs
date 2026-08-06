use fltk::{
    app,
    //dialog,
    prelude::*,
    window::OverlayWindow,
    enums::Event,
    frame::Frame,
    button::Button,

};
use super::GLOBAL_SETTINGS;
use debug_print::{debug_println as dprintln};

pub trait TooltipExt {
    //Alternative tooltip implementation because native FLTK tooltips ignore z-order for always-on-top windows.
    fn with_overlay_tooltip(&mut self, win: &OverlayWindow, text_frame: &Frame, title: &str);
}

impl TooltipExt for Button {
    fn with_overlay_tooltip(&mut self, tooltip_win: &OverlayWindow, tooltip_text: &Frame, title: &str) {
        if GLOBAL_SETTINGS.no_tooltips {
            return;
        }
        let mut tooltip_win = tooltip_win.clone();
        let tooltip_text = tooltip_text.clone();
        let w = self.clone();
        let title = title.to_string();

        self.handle(move |_, ev| match ev {
        Event::Enter => {
            //let (ex, ey) = app::event_coords();
            let mut win = tooltip_win.clone();
            let mut tooltip_text_clone = tooltip_text.clone();
            let w_p = w.clone().as_widget_ptr();
            let title = title.to_string();
            app::add_timeout3(1.0, move |_handle| {
                dprintln!("tooltip timeout3");
                if let Some(current_widget) = fltk::app::belowmouse::<fltk::widget::Widget>() {
                    if current_widget.as_widget_ptr() == w_p && !win.shown() {
                        let x = app::event_x_root() + 5;
                        let y = app::event_y_root() + 10;
                        win.resize(x, y, win.w(), win.h());
                        win.show();
                        win.set_override();
                        win.set_on_top();
                        tooltip_text_clone.set_label(&title);
                    }
                }
            });
            true
        }
        Event::Leave => {
            tooltip_win.hide();
            true
        }
        Event::Push => {
            tooltip_win.hide();
            true
        }
        Event::Hide => {
            tooltip_win.hide();
            true
        }
        _ => false,
    });
    }
}

