use fltk::{
    app,
    //dialog,
    prelude::*,
    window::OverlayWindow,
    enums::Event,
    frame::Frame,
    button::Button,
    image::PngImage,
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



pub trait IconExt {
    // dpi-aware ".set_image(image); .set_align(Align::Center | Align::ImageBackdrop);"
    fn set_png_icon(&mut self, icon: &str);
}

impl IconExt for Button {
    fn set_png_icon(&mut self, icon: &str) {
        // TODO: other icon formats
        let working_dir = std::env::current_dir().unwrap();
        let screen_scale = fltk::app::screen_scale(0);
        let ui_scale = GLOBAL_SETTINGS.ui_scaling;
        let icon_scale_suffix = if screen_scale < 1.25 && ui_scale < 1.25 {
            ""
        } else if screen_scale < 1.5 || ui_scale < 1.5 {
            ".125"
        } else {
            ".150"
        };
        let mut image_path = working_dir.join(format!("icons\\{}{}.png", icon, icon_scale_suffix));
        if !image_path.exists() {
            image_path = working_dir.join(format!("icons\\{}.png", icon));
        }
        
        if let Ok(mut image) = PngImage::load(image_path.to_str().unwrap_or("")) {
            //image.scale(15, 15, true, true);
            if screen_scale == 1.0 {
                self.set_image(Some(image));
                self.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
            } else {
                let scale = screen_scale;
                self.draw(move |b| {
                    fltk::draw::draw_box(b.frame(), b.x(), b.y(), b.w(), b.h(), b.color());
                    
                    let (mut img_data, img_w, img_h, depth) = if scale >= 1.25 && scale < 1.50{
                        (image.to_rgb_image().unwrap(), image.width(), image.height(), 4)
                    } else {
                        (image.to_rgb_image().unwrap(), image.width(), image.height(), 4)
                    };
                    
                    let phys_x = ((b.x() as f32).round() * scale) as i32;
                    let phys_y = ((b.y() as f32).round() * scale) as i32;
                    let phys_w = ((b.w() as f32).round() * scale) as i32;
                    let phys_h = ((b.h() as f32).round() * scale) as i32;

                    fltk::draw::push_clip(
                        phys_x,
                        phys_y,
                        phys_w,
                        phys_h,
                    );
                    fltk::draw::override_scale();
                    let offset_x = (phys_x + ((phys_w / 2) - (img_w / 2))) as f32;
                    let offset_y = (phys_y + ((phys_h / 2) - (img_h / 2))) as f32;
                    
                    img_data.draw(offset_x as i32, offset_y as i32, 18, 18);
                    
                    fltk::draw::restore_scale(scale); 
                    fltk::draw::pop_clip();
                });
            }
        }
    }
}

