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


pub struct DictPopupView {
    pub txt_popup_dict: text::TextDisplay,
    pub title_frame_dict: Frame,
    pub src_dict: String,
    pub prnn_index: i32,

    pub win_popup_dict: DoubleWindow,

    pub dict_buttons: HashMap<String, fltk::button::RadioButton>,
    pub fav_button_dict: button::Button,
}

impl DictPopupView {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>, main_win: DoubleWindow, tooltip_win: fltk::window::OverlayWindow, tooltip_text: fltk::frame::Frame) -> Self {

        let working_dir = std::env::current_dir().unwrap();
        let text_bg_color_popup = enums::Color::from_hex_str(&GLOBAL_SETTINGS.text_bg_color_popup).unwrap_or(enums::Color::from_hex(0xF0F0F0));

    	////////////////////---------------BEGIN DICT POPUP WIN---------------/////////////////////
        let mut win_popup_dict = window::Window::default().with_size(450, 200);
        let mut frame_wrapper_dict = group::Flex::default().column().size_of_parent();

        frame_wrapper_dict.set_margins(3,3,3,3);
        let mut frame_dict = group::Flex::default().column();
        frame_dict.set_spacing(3);
        frame_dict.set_frame(fltk::enums::FrameType::EngravedBox);

        ////////////////////---------------TITLEBAR---------------/////////////////////
        let mut flex_titlebar_dict = group::Flex::default().row();//.below_of(&txt, 10);

        flex_titlebar_dict.set_margins(2,2,3,0);
        flex_titlebar_dict.set_pad(5);
        let mut close_button_dict = button::Button::new(5, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\close.png").to_str().unwrap_or("")) {
            close_button_dict.set_image(Some(image));
            close_button_dict.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }
        //close_button_dict.set_tooltip(t!(close));
        close_button_dict.with_overlay_tooltip(&tooltip_win, &tooltip_text, t!(close));
        
        let mut title_frame_dict = Frame::default().with_label("").with_align(fltk::enums::Align::Right);
        title_frame_dict.set_label_size(GLOBAL_SETTINGS.ui_font_size);
        let _status_s_frame = Frame::default();

        let mut fav_button_dict = button::Button::new(51, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\fav.png").to_str().unwrap_or("")) {
            fav_button_dict.set_image(Some(image));
            fav_button_dict.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }
        //fav_button_dict.set_tooltip(t!(add_to_fav));
        fav_button_dict.with_overlay_tooltip(&tooltip_win, &tooltip_text, t!(add_to_fav));

        let mut refresh_button_dict = button::Button::new(51, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\refresh.png").to_str().unwrap_or("")) {
            refresh_button_dict.set_image(Some(image));
            refresh_button_dict.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }
        //refresh_button_dict.set_tooltip(t!(refresh));
        refresh_button_dict.with_overlay_tooltip(&tooltip_win, &tooltip_text, t!(refresh));

        let mut lng_menu_button_wrapper_dict = button::Button::new(51, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\settings.png").to_str().unwrap_or("")) {
            lng_menu_button_wrapper_dict.set_image(Some(image));
            lng_menu_button_wrapper_dict.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }
        //lng_menu_button_wrapper_dict.set_tooltip(t!(lang));
        //lng_menu_button_wrapper_dict.with_overlay_tooltip(&tooltip_win, &tooltip_text, t!(lang));
        let mut lng_menu_button_dict = fltk::menu::MenuButton::default();//.with_type(fltk::menu::MenuButtonType::Popup3);

        let mut prnn_button_dict = button::Button::new(51, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\audio.png").to_str().unwrap_or("")) {
            prnn_button_dict.set_image(Some(image));
            prnn_button_dict.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }
        //prnn_button_dict.set_tooltip(t!(pronunciation));
        prnn_button_dict.with_overlay_tooltip(&tooltip_win, &tooltip_text, t!(pronunciation));

        
        let mut open_button_dict = button::Button::new(51, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\open.png").to_str().unwrap_or("")) {
            open_button_dict.set_image(Some(image));
            open_button_dict.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }
        //open_button_dict.set_tooltip(t!(open_main_win));
        open_button_dict.with_overlay_tooltip(&tooltip_win, &tooltip_text, t!(open_main_win));

        flex_titlebar_dict.fixed(&close_button_dict, 18);
        flex_titlebar_dict.fixed(&title_frame_dict, 1);
        flex_titlebar_dict.fixed(&fav_button_dict, 18);
        flex_titlebar_dict.fixed(&lng_menu_button_wrapper_dict, 18);
        flex_titlebar_dict.fixed(&prnn_button_dict, 18);
        flex_titlebar_dict.fixed(&refresh_button_dict, 18);
        flex_titlebar_dict.fixed(&open_button_dict, 18);

        frame_dict.fixed(&flex_titlebar_dict, 20);
        flex_titlebar_dict.end();
        ////////////////////---------------END TITLEBAR---------------/////////////////////

        ////////////////////---------------FLEXBOX1---------------/////////////////////
        let mut flex_dict = group::Flex::default().column();
        flex_dict.set_margins(0, 0, 0, 0);
        flex_dict.set_spacing(7);

        /////TEXTAREA
        //let src_dict_buf = text::TextBuffer::default();
        

        let mut txt_dict = text::TextDisplay::default();
        txt_dict.set_text_size(GLOBAL_SETTINGS.text_font_size);
        txt_dict.set_color(text_bg_color_popup);
        txt_dict.set_frame(fltk::enums::FrameType::FlatBox);
        //txt_dict.set_buffer(dict_buf.clone());
        txt_dict.wrap_mode(text::WrapMode::AtBounds, 0);

        /////-----BEGIN FLEX INNER (DICT BUTTONS)-----/////
        let mut flex2_dict = group::Flex::default().column();
        flex2_dict.set_spacing(5);

        let mut flex_buttons_wrapper_dict = group::Flex::default().column();
        flex_buttons_wrapper_dict.set_margins(15, 0, 15, 0);
        let flex_buttons_dict = group::Flex::default().row();

        let mut dict_buttons: HashMap<String, fltk::button::RadioButton> = HashMap::new();
        for qwe in GLOBAL_SETTINGS.dictionaries.iter() {
            let mut button = button::RadioButton::new(0, 0, 180, 25, &*qwe.name);
            let icon_path = format!(r"icons/{}.ico", &qwe.uid);
            dprintln!("{}", icon_path);
            if let Ok(image) = IcoImage::load(working_dir.join(&icon_path).to_str().unwrap_or("")) {
                button.set_image(Some(image));
                button.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageNextToText);
            }
            if qwe.uid == UICONFIG.selected_dict {
                button.set(true);
            }
            button.set_callback({
                let s = app_sender;
                move |_b| {
                    s.send(AppEvent::SetDict(qwe.uid.clone()));
                    s.send(AppEvent::RequestDictEntry(false, false, false));
                }
            });
            dict_buttons.insert(qwe.uid.clone(), button);
        }

        flex_buttons_dict.end();
        flex2_dict.fixed(&flex_buttons_wrapper_dict, 25);
        flex_buttons_wrapper_dict.end();
        
        flex_dict.fixed(&flex2_dict, 31);
        flex2_dict.end();
        /////-----END FLEX INNER (DICT BUTTONS)-----/////
        
        flex_dict.end();
        ////////////////////---------------END FLEXBOX1---------------/////////////////////
        
        frame_dict.end();
        frame_wrapper_dict.end();
        win_popup_dict.make_resizable(true);
        win_popup_dict.set_border(false);
        win_popup_dict.set_frame(fltk::enums::FrameType::UpBox);
        win_popup_dict.resizable(&win_popup_dict);
        win_popup_dict.size_range(400, 150, 0 ,0);

        win_popup_dict.end();
        ////////////////////---------------END POPUP WIN---------------/////////////////////



        lng_menu_button_dict.add(
                "From:",
                fltk::enums::Shortcut::None,
                fltk::menu::MenuFlag::Inactive,
                |_| {}
        );

        for item in &GLOBAL_SETTINGS.pinned_src_languages {
            let lng = Lang::from_str(item).unwrap_or(Lang::En);
            let name_from = format!("&{}", lng.name());
            lng_menu_button_dict.add(
                name_from.as_ref(),
                fltk::enums::Shortcut::None,
                fltk::menu::MenuFlag::Normal,
                {
                    let s = app_sender;
                    move |_b| {
                        s.send(AppEvent::SetSrcLang(lng.clone()));
                        s.send(AppEvent::RequestDictEntry(false, false, false));
                    }
                },
            );
        }
        
        for lng in Lang::iter() {
            let name_from = format!("All (source)/{}", lng.name());
            lng_menu_button_dict.add(
                name_from.as_ref(),
                fltk::enums::Shortcut::None,
                fltk::menu::MenuFlag::Normal,
                {
                    let s = app_sender;
                    move |_b| {
                        s.send(AppEvent::SetSrcLang(lng.clone()));
                        s.send(AppEvent::RequestDictEntry(false, false, false));
                    }
                },
            );
        };

        lng_menu_button_dict.add(
                "To:",
                fltk::enums::Shortcut::None,
                fltk::menu::MenuFlag::Inactive,
                |_| {}
        );


    	for item in &GLOBAL_SETTINGS.pinned_target_languages {
            let lng = Lang::from_str(item).unwrap_or(Lang::Ru);
            let name_to = format!("&{} ", lng.name());
            lng_menu_button_dict.add(
                name_to.as_ref(),
                fltk::enums::Shortcut::None,
                fltk::menu::MenuFlag::Normal,
                {
                    let s = app_sender;
                    move |_b| {
                        s.send(AppEvent::SetTargetLang(lng.clone()));
                        s.send(AppEvent::RequestDictEntry(false, false, false));
                    }
                },
            );
        }
        for lng in Lang::iter() {
            let name_to = format!("All (target)/{}", lng.name());
            lng_menu_button_dict.add(
                name_to.as_ref(),
                fltk::enums::Shortcut::None,
                fltk::menu::MenuFlag::Normal,
                {
                    let s = app_sender;
                    move |_b| {
                        s.send(AppEvent::SetTargetLang(lng.clone()));
                        s.send(AppEvent::RequestDictEntry(false, false, false));
                    }
                },
            );
        };

        lng_menu_button_dict.hide();
        lng_menu_button_wrapper_dict.set_callback(move |b| {
            if let Some(item) = lng_menu_button_dict.menu()
                && let Some(mut selected) = item.popup(b.x(), b.y() + b.h()) {
                    selected.do_callback(&lng_menu_button_dict); 
            }
        });


        win_popup_dict.hotspot(&close_button_dict);
		win_popup_dict.show();
		win_popup_dict.set_opacity(GLOBAL_SETTINGS.popup_opacity); //0.8
		win_popup_dict.hide();



		let is_inner_dict = Rc::new(RefCell::new(false));
		frame_dict.handle({
            let mut win_popup_dict = win_popup_dict.clone();
            let is_inner_dict = Rc::clone(&is_inner_dict);
            move |_t, event| {
                borderless_win_frame_handler(event, &mut win_popup_dict, &is_inner_dict)
            }
        });
		win_popup_dict.handle({
            //popup borderless window resizing and dragging
            let mut coords = BLWCoords::default();
            let is_inner_dict = Rc::clone(&is_inner_dict);

            move |window, event| {
                borderless_win_handler(window, event, &mut coords, &is_inner_dict)
            }
        });
		close_button_dict.set_callback({
            let mut win_popup_dict = win_popup_dict.clone();
            move |_| {
                win_popup_dict.hide();
            }
        });

		fav_button_dict.set_callback({
            let s = app_sender;
            move |_| {
                s.send(AppEvent::ToggleFav(true));
            }
        });
		prnn_button_dict.set_callback({
            let s = app_sender;
            move |_| {
                s.send(AppEvent::PRNNString(false));
            }
        });

        refresh_button_dict.set_callback({
            let s = app_sender;
            move |_button| {
                s.send(AppEvent::RequestDictEntry(false, true, false));
            }
        });
		open_button_dict.set_callback({
            //let s = app_sender;
            let mut main_win = main_win.clone();
            let mut win_popup_dict = win_popup_dict.clone();
            move |_| {
                main_win.show();
                let _ = main_win.take_focus();
                win_popup_dict.hide();
            }
        });
		win_popup_dict.clone().set_callback(|w| {
            // We intercept the closing of the window here
            w.hide();
        });

		win_popup_dict.resize(100, 100, UICONFIG.popup_dict_w, UICONFIG.popup_dict_h);

		DictPopupView {
			txt_popup_dict: txt_dict,
			title_frame_dict,
			src_dict: "".to_string(),
			prnn_index: 0,
			win_popup_dict,
			dict_buttons,
			fav_button_dict,
		}
    }
}
