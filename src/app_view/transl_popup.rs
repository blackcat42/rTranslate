use debug_print::{debug_println as dprintln};
use fltk::{
    prelude::*,
    window,
    window::DoubleWindow,
    text,
    enums,
    button,
    group,
    image::PngImage,
    image::IcoImage,
    frame::Frame,
};

use std::rc::Rc;
use std::cell::RefCell;
use std::str::FromStr;
use std::convert::AsRef;
use std::collections::HashMap;

use strum::IntoEnumIterator;
//use mouse_position::mouse_position::{Mouse};
use crate::types::{
    AppEvent, Lang, BLWCoords
};

use crate::utils::helpers::{
    borderless_win_handler, 
    borderless_win_frame_handler
};
use crate::utils::widgets::TooltipExt;

use super::GLOBAL_SETTINGS;
use super::UICONFIG;
use super::t;


pub struct TranslPopupView {
    pub translator_buttons: HashMap<String, fltk::button::RadioButton>,
    pub fav_button: button::Button,
    pub txt_popup: text::TextDisplay,
    pub title_frame: Frame,
    pub src: String, //todo: use src_buf?
    pub win_popup: DoubleWindow,
}

impl TranslPopupView {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>, main_win: DoubleWindow, win_popup_dict: DoubleWindow, tooltip_win: fltk::window::OverlayWindow, tooltip_text: fltk::frame::Frame) -> Self {

        let working_dir = std::env::current_dir().unwrap();
        let text_bg_color_popup = enums::Color::from_hex_str(&GLOBAL_SETTINGS.text_bg_color_popup).unwrap_or(enums::Color::from_hex(0xF0F0F0));

    	////////////////////---------------BEGIN POPUP WIN---------------/////////////////////
        let mut win_popup = window::Window::default().with_size(550, 200);
        let mut frame_wrapper = group::Flex::default().column().size_of_parent();
        frame_wrapper.set_margins(3,3,3,3);
        let mut frame = group::Flex::default().column();
        frame.set_spacing(3);
        frame.set_frame(fltk::enums::FrameType::EngravedBox);

        ////////////////////---------------TITLEBAR---------------/////////////////////
        let mut flex_titlebar = group::Flex::default().row();
        flex_titlebar.set_margins(2,2,3,0);
        flex_titlebar.set_pad(5);

        let mut close_button = button::Button::new(5, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\close.png").to_str().unwrap_or("")) {
            //image.scale(20, 20, true, true);
            close_button.set_image(Some(image));
            close_button.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }
        //close_button.set_tooltip(t!(close));
        close_button.with_overlay_tooltip(&tooltip_win, &tooltip_text, t!(close));

        
        let mut title_frame = Frame::default().with_label("").with_align(fltk::enums::Align::Right);
        title_frame.set_label_size(GLOBAL_SETTINGS.ui_font_size);
        let _status_s_frame = Frame::default();

        let mut fav_button = button::Button::new(51, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\fav.png").to_str().unwrap_or("")) {
            fav_button.set_image(Some(image));
            fav_button.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }
        //fav_button.set_tooltip(t!(add_to_fav));
        fav_button.with_overlay_tooltip(&tooltip_win, &tooltip_text, t!(add_to_fav));

        let mut refresh_button = button::Button::new(51, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\refresh.png").to_str().unwrap_or("")) {
            refresh_button.set_image(Some(image));
            refresh_button.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }
        //refresh_button.set_tooltip(t!(refresh));
        refresh_button.with_overlay_tooltip(&tooltip_win, &tooltip_text, t!(refresh));

        let mut lng_menu_button_wrapper = button::Button::new(51, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\settings.png").to_str().unwrap_or("")) {
            lng_menu_button_wrapper.set_image(Some(image));
            lng_menu_button_wrapper.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }
        //lng_menu_button_wrapper.set_tooltip(t!(lang));
        //lng_menu_button_wrapper.with_overlay_tooltip(&tooltip_win, &tooltip_text, t!(lang));
        let mut lng_menu_button = fltk::menu::MenuButton::default();//.with_type(fltk::menu::MenuButtonType::Popup3);

        let mut tts_button = button::Button::new(28, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\audio.png").to_str().unwrap_or("")) {
            tts_button.set_image(Some(image));
            tts_button.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }
        //tts_button.set_tooltip(t!(tts));
        tts_button.with_overlay_tooltip(&tooltip_win, &tooltip_text, t!(tts));
        /*let mut qsettings_button = button::Button::new(51, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\settings.png").to_str().unwrap_or("")) {
            qsettings_button.set_image(Some(image));
            qsettings_button.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }*/
        let mut dict_button = button::Button::new(51, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\dict.png").to_str().unwrap_or("")) {
            dict_button.set_image(Some(image));
            dict_button.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }
        //dict_button.set_tooltip(t!(send_to_dict));
        dict_button.with_overlay_tooltip(&tooltip_win, &tooltip_text, t!(send_to_dict));

        let mut open_button = button::Button::new(51, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\open.png").to_str().unwrap_or("")) {
            open_button.set_image(Some(image));
            open_button.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }
        //open_button.set_tooltip(t!(open_main_win));
        open_button.with_overlay_tooltip(&tooltip_win, &tooltip_text, t!(open_main_win));

        flex_titlebar.fixed(&close_button, 18);
        flex_titlebar.fixed(&title_frame, 1);
        flex_titlebar.fixed(&fav_button, 18);
        flex_titlebar.fixed(&refresh_button, 18);
        flex_titlebar.fixed(&lng_menu_button_wrapper, 18);
        flex_titlebar.fixed(&tts_button, 18);
        flex_titlebar.fixed(&dict_button, 18);
        //flex_titlebar.fixed(&qsettings_button, 18);
        flex_titlebar.fixed(&open_button, 18);

        frame.fixed(&flex_titlebar, 20);
        flex_titlebar.end();
        ////////////////////---------------END TITLEBAR---------------/////////////////////

        ////////////////////---------------FLEXBOX1---------------/////////////////////
        let mut flex = group::Flex::default().column();
        flex.set_margins(0, 0, 0, 0);
        flex.set_spacing(7);

        /////TEXTAREA
        

        let mut txt = text::TextDisplay::default();
        txt.set_text_size(GLOBAL_SETTINGS.text_font_size);
        txt.set_color(text_bg_color_popup);
        txt.set_frame(fltk::enums::FrameType::FlatBox);
        //txt.set_buffer(translation_buf.clone());
        txt.wrap_mode(text::WrapMode::AtBounds, 0);

        /////-----BEGIN FLEX INNER (TRANSLATION BUTTONS)-----/////
        let mut flex2 = group::Flex::default().column();
        flex2.set_spacing(5);

        let mut flex_buttons_wrapper = group::Flex::default().column();
        flex_buttons_wrapper.set_margins(15, 0, 15, 0);
        let flex_buttons = group::Flex::default().row();

        let mut translator_buttons: HashMap<String, fltk::button::RadioButton> = HashMap::new();
        let btn_n = GLOBAL_SETTINGS.translators.len();
        for qwe in GLOBAL_SETTINGS.translators.iter() {
            let mut button = button::RadioButton::new(0, 0, 180, 25, &*qwe.name);
            let icon_path = if let Some(cmd) = &qwe.command && cmd == "QTRANSLATE" {
                format!(r"extensions/qtranslate/Services/{}/Service.ico", &qwe.uid)
            } else {
                format!(r"icons/{}.ico", &qwe.uid)
            };
            dprintln!("{}", icon_path);
            if let Ok(image) = IcoImage::load(working_dir.join(&icon_path).to_str().unwrap_or("")) {
                button.set_image(Some(image));
                if btn_n > 5 {
                    button.set_align(fltk::enums::Align::Left | fltk::enums::Align::Inside | fltk::enums::Align::ImageNextToText | fltk::enums::Align::Clip);
                } else {
                    button.set_align(fltk::enums::Align::Inside | fltk::enums::Align::ImageNextToText | fltk::enums::Align::Clip);
                }
                
            }
            if qwe.uid == UICONFIG.selected_translator {
                button.set(true);
            }
            button.set_callback({
                let s = app_sender;
                move |_b| {
                    s.send(AppEvent::SetTranslator(qwe.uid.clone()));
                    s.send(AppEvent::Translate(false, false, false));
                }
            });
            translator_buttons.insert(qwe.uid.clone(), button);
        }

        flex_buttons.end();
        flex2.fixed(&flex_buttons_wrapper, 25);
        flex_buttons_wrapper.end();
        
        flex.fixed(&flex2, 31);
        flex2.end();
        /////-----END FLEX INNER (TRANSLATION BUTTONS)-----/////
        //flex2.auto_layout();
        
        flex.end();
        ////////////////////---------------END FLEXBOX1---------------/////////////////////
        
        frame.end();
        frame_wrapper.end();
        win_popup.make_resizable(true);
        win_popup.set_border(false);
        win_popup.set_frame(fltk::enums::FrameType::UpBox);
        win_popup.resizable(&win_popup);
        win_popup.size_range(400, 150, 0 ,0);
        win_popup.end();        
        ////////////////////---------------END POPUP WIN---------------/////////////////////



        lng_menu_button.add(
                "From:",
                fltk::enums::Shortcut::None,
                fltk::menu::MenuFlag::Inactive,
                |_| {}
        );
        
        for item in &GLOBAL_SETTINGS.pinned_src_languages {
            let lng = Lang::from_str(item).unwrap_or(Lang::En);
            let name_from = format!("&{}", lng.name());
            lng_menu_button.add(
                name_from.as_ref(),
                fltk::enums::Shortcut::None,
                fltk::menu::MenuFlag::Normal,
                {
                    let s = app_sender;
                    let lng = lng.clone();
                    move |_b| {
                        s.send(AppEvent::SetSrcLang(lng.clone()));
                        s.send(AppEvent::Translate(false, false, false));
                    }
                },
            );
        }
        
        for lng in Lang::iter() {
            let name_from = format!("All (source)/{}", lng.name());
            lng_menu_button.add(
                name_from.as_ref(),
                fltk::enums::Shortcut::None,
                fltk::menu::MenuFlag::Normal,
                {
                    let s = app_sender;
                    let lng = lng.clone();
                    move |_b| {
                        s.send(AppEvent::SetSrcLang(lng.clone()));
                        s.send(AppEvent::Translate(false, false, false));
                    }
                },
            );
        };
        //lng_menu_button.add("", fltk::enums::Shortcut::None, fltk::menu::MenuFlag::MenuDivider, |_| {});
        lng_menu_button.add(
                "To:",
                fltk::enums::Shortcut::None,
                fltk::menu::MenuFlag::Inactive,
                |_| {}
        );
        

        for item in &GLOBAL_SETTINGS.pinned_target_languages {
            let lng = Lang::from_str(item).unwrap_or(Lang::Ru);
            let name_to = format!("&{} ", lng.name());
            lng_menu_button.add(
                name_to.as_ref(),
                fltk::enums::Shortcut::None,
                fltk::menu::MenuFlag::Normal,
                {
                    let s = app_sender;
                    let lng = lng.clone();
                    move |_b| {
                        s.send(AppEvent::SetTargetLang(lng.clone()));
                        s.send(AppEvent::Translate(false, false, false));
                    }
                },
            );
        }
        for lng in Lang::iter() {
            let name_to = format!("All (target)/{}", lng.name());
            lng_menu_button.add(
                name_to.as_ref(),
                fltk::enums::Shortcut::None,
                fltk::menu::MenuFlag::Normal,
                {
                    let s = app_sender;
                    let lng = lng.clone();
                    move |_b| {
                        s.send(AppEvent::SetTargetLang(lng.clone()));
                        s.send(AppEvent::Translate(false, false, false));
                    }
                },
            );
        };

        lng_menu_button.hide();
        
        lng_menu_button_wrapper.set_callback(move |b| {
            if let Some(item) = lng_menu_button.menu()
                && let Some(mut selected) = item.popup(b.x(), b.y() + b.h()) {
                    selected.do_callback(&lng_menu_button); 
            }
        });
        

        win_popup.hotspot(&close_button);
        
        //trying to hide popup window at startup...
        win_popup.show();
        
        win_popup.set_opacity(GLOBAL_SETTINGS.popup_opacity); //This should be called on a shown window
        


        win_popup.hide();


        //IMPL RESIZING/DRAGGING BHVR FOR BORDELESS WINDOW
        let is_inner = Rc::new(RefCell::new(false));
        
        frame.handle({
            let mut win_popup = win_popup.clone();
            let is_inner = Rc::clone(&is_inner);
            move |_t, event| {
                borderless_win_frame_handler(event, &mut win_popup, &is_inner)
            }
        });
        
    
        win_popup.handle({
            //popup borderless window resizing and dragging
            let mut coords = BLWCoords::default();
            let is_inner = Rc::clone(&is_inner);

            move |window, event| {
                borderless_win_handler(window, event, &mut coords, &is_inner)
            }
        });


        //WIDGET CALLBACKS
        close_button.set_callback({
            let mut win_popup = win_popup.clone();
            move |_| {
                win_popup.hide();
            }
        });
        
        fav_button.set_callback({
            let s = app_sender;
            move |_| {
                s.send(AppEvent::ToggleFav(false));
            }
        });
        
        
        tts_button.set_callback({
            let s = app_sender;
            move |_| {
                s.send(AppEvent::TTString());
            }
        });
        
        
        dict_button.set_callback({
            let win_popup = win_popup.clone();
            let mut win_popup_dict = win_popup_dict.clone();
            let s = app_sender;
            move |_button| {
                win_popup_dict.show();
                win_popup_dict.set_pos(win_popup.x_root() + 50, win_popup.y_root() + 50);
                win_popup_dict.set_on_top();
                
                s.send(AppEvent::SendToDict());
            }
        });

        refresh_button.set_callback({
            let s = app_sender;
            move |_button| {
                s.send(AppEvent::Translate(false, true, false));
            }
        });
        
        /*refresh_button_main.set_callback({
            let s = app_sender;
            move |_button| {
                s.send(AppEvent::Translate(true, true, false));
                s.send(AppEvent::RequestDictEntry(true, true, false));
            }
        });*/
        open_button.set_callback({
            //let s = app_sender;
            let mut main_win = main_win.clone();
            let mut win_popup = win_popup.clone();
            move |_| {
                main_win.show();
                let _ = main_win.take_focus();
                win_popup.hide();
            }
        });
        
        
        win_popup.clone().set_callback(|w| {
            // We intercept the closing of the window here
            w.hide();
        });

        win_popup.resize(100, 100, UICONFIG.popup_w, UICONFIG.popup_h);



		TranslPopupView {
			txt_popup: txt,
			title_frame,
            win_popup,
            translator_buttons,
            fav_button,
            src: "".to_string(),
		}
    }
}
