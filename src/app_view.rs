use fltk::{
    app,
    prelude::*,
    window,
    window::DoubleWindow,
    text,
    enums,
    browser,
    button,
    group,
    image::PngImage,
    image::IcoImage,
    frame::Frame,
};

use std::rc::Rc;
use std::cell::RefCell;
use std::time::Duration;
use std::thread;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::str::FromStr;
use std::convert::AsRef;
use std::collections::HashMap;

use strum::IntoEnumIterator;
use mouse_position::mouse_position::{Mouse};
use crate::types::{AppEvent, Lang, LangNames, TTSource, PRNNSource, TranslSource, UIState, UIStateDict };

use crate::bbcode::{dsl_parse};

use tray_icon::{
    menu::{
        MenuEvent,
    },
    TrayIconEvent,
};

use super::GLOBAL_SETTINGS;


pub struct AppView {
    //app_sender: fltk::app::Sender<AppEvent>,

    pub src_buf: text::TextBuffer,
    pub src_dict_buf: text::TextBuffer,
    translation_buf: text::TextBuffer,
    dict_buf: text::TextBuffer,
    waiting_buf: text::TextBuffer,
    error_buf: text::TextBuffer,
    is_processing: Arc<AtomicBool>,

    txt_popup: text::TextDisplay,
    txt_popup_dict: text::TextDisplay,
    txt_main: text::TextDisplay,
    txt_dict_main: text::TextDisplay,
    title_frame: Frame,
    title_frame_dict: Frame,
    //status_frame: Frame,
    status_frame_main: Frame,
    //status_frame_dict: Frame,
    pub src: String, //todo: use src_buf?
    pub src_dict: String,

    transl_browser: fltk::browser::HoldBrowser,
    fav_browser: fltk::browser::HoldBrowser,
    tts_browser: fltk::browser::HoldBrowser,
    dict_assets_browser: fltk::browser::HoldBrowser,

    win_popup: DoubleWindow,
    win_popup_dict: DoubleWindow,
    pub main_win: DoubleWindow,

    translator_buttons: HashMap<String, fltk::button::RadioButton>,
    dict_buttons: HashMap<String, fltk::button::RadioButton>,
    fav_button: button::Button,
    fav_button_dict: button::Button,
    fav_button_main: button::Button,


    lang_choice_from: fltk::menu::Choice,
    lang_choice_to: fltk::menu::Choice,
    dict_choice: fltk::menu::Choice,
    transl_choice: fltk::menu::Choice,
    tts_choice: fltk::menu::Choice,
    prnn_choice: fltk::menu::Choice,
}

pub struct BLWCoords {
    x: i32,
    y: i32,
    x_start: i32,
    y_start: i32,
    initial_window_height: i32,
    initial_window_width: i32,
    init_on_border_left: bool,
    init_on_border_right: bool,
    init_on_border_top: bool,
    init_on_border_bottom: bool,
}

impl AppView {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>) -> Self {

        let working_dir = std::env::current_dir().unwrap();

        ////////////////////---------------BEGIN UI---------------/////////////////////
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

        
        let mut title_frame = Frame::default().with_label("").with_align(fltk::enums::Align::Right);
        title_frame.set_label_size(GLOBAL_SETTINGS.ui_font_size);
        let _status_s_frame = Frame::default();

        let mut fav_button = button::Button::new(51, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\fav.png").to_str().unwrap_or("")) {
            fav_button.set_image(Some(image));
            fav_button.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }

        let mut refresh_button = button::Button::new(51, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\refresh.png").to_str().unwrap_or("")) {
            refresh_button.set_image(Some(image));
            refresh_button.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }

        let mut tts_button = button::Button::new(28, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\audio.png").to_str().unwrap_or("")) {
            tts_button.set_image(Some(image));
            tts_button.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }
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
        let mut open_button = button::Button::new(51, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\open.png").to_str().unwrap_or("")) {
            open_button.set_image(Some(image));
            open_button.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }

        flex_titlebar.fixed(&close_button, 18);
        flex_titlebar.fixed(&title_frame, 1);
        flex_titlebar.fixed(&fav_button, 18);
        flex_titlebar.fixed(&refresh_button, 18);
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
        let mut src_buf = text::TextBuffer::default();
        let translation_buf = text::TextBuffer::default();
        let waiting_buf = text::TextBuffer::default();
        let error_buf = text::TextBuffer::default();

        let mut txt = text::TextDisplay::default();
        txt.set_color(enums::Color::from_hex_str(&GLOBAL_SETTINGS.text_bg_color).unwrap_or(enums::Color::from_hex(0xF0F0F0)));
        txt.set_frame(fltk::enums::FrameType::FlatBox);
        txt.set_buffer(translation_buf.clone());
        txt.wrap_mode(text::WrapMode::AtBounds, 0);

        /////-----BEGIN FLEX INNER (TRANSLATION BUTTONS)-----/////
        let mut flex2 = group::Flex::default().column();
        flex2.set_spacing(5);

        let mut flex_buttons_wrapper = group::Flex::default().column();
        flex_buttons_wrapper.set_margins(15, 0, 15, 0);
        let flex_buttons = group::Flex::default().row();

        let mut translator_buttons: HashMap<String, fltk::button::RadioButton> = HashMap::new();
        for qwe in GLOBAL_SETTINGS.translators.iter() {
            let mut button = button::RadioButton::new(0, 0, 180, 25, &*qwe.name);
            let icon_path = format!(r"icons/{}.ico", &qwe.uid);
            println!("{}", icon_path);
            if let Ok(image) = IcoImage::load(working_dir.join(&icon_path).to_str().unwrap_or("")) {
                button.set_image(Some(image));
                button.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageNextToText);
            }
            if qwe.uid == GLOBAL_SETTINGS.default_translator {
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
        
        let mut title_frame_dict = Frame::default().with_label("").with_align(fltk::enums::Align::Right);
        title_frame_dict.set_label_size(GLOBAL_SETTINGS.ui_font_size);
        let _status_s_frame = Frame::default();

        let mut fav_button_dict = button::Button::new(51, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\fav.png").to_str().unwrap_or("")) {
            fav_button_dict.set_image(Some(image));
            fav_button_dict.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }

        let mut refresh_button_dict = button::Button::new(51, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\refresh.png").to_str().unwrap_or("")) {
            refresh_button_dict.set_image(Some(image));
            refresh_button_dict.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }

        let mut prnn_button_dict = button::Button::new(51, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\audio.png").to_str().unwrap_or("")) {
            prnn_button_dict.set_image(Some(image));
            prnn_button_dict.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }

        
        let mut open_button_dict = button::Button::new(51, 5, 18, 18, "");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\open.png").to_str().unwrap_or("")) {
            open_button_dict.set_image(Some(image));
            open_button_dict.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }

        flex_titlebar_dict.fixed(&close_button_dict, 18);
        flex_titlebar_dict.fixed(&fav_button_dict, 18);
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
        let src_dict_buf = text::TextBuffer::default();
        let dict_buf = text::TextBuffer::default();

        let mut txt_dict = text::TextDisplay::default();
        txt_dict.set_color(enums::Color::from_hex_str(&GLOBAL_SETTINGS.text_bg_color).unwrap_or(enums::Color::from_hex(0xF0F0F0)));
        txt_dict.set_frame(fltk::enums::FrameType::FlatBox);
        txt_dict.set_buffer(dict_buf.clone());
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
            println!("{}", icon_path);
            if let Ok(image) = IcoImage::load(working_dir.join(&icon_path).to_str().unwrap_or("")) {
                button.set_image(Some(image));
                button.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageNextToText);
            }
            if qwe.uid == GLOBAL_SETTINGS.default_dict {
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



        ////////////////////---------------BEGIN MAIN WIN---------------/////////////////////        
        let mut main_win = window::Window::default().with_size(800, 600).with_label("rTranslate");
        let mut main_flex_wrapper = group::Flex::new(0,0,800,600,None);
        main_flex_wrapper.set_type(group::FlexType::Column);
        let mut main_flex_wrapper_inner = group::Flex::default().row();

        let mut main_flex_left = group::Flex::new(0,0,400,585,None);
        main_flex_left.set_type(group::FlexType::Column);

        let mut main_src_txt_wrapper = group::Flex::default().column();
        let mut main_src_txt = text::TextEditor::default().with_label("Source:").with_align(fltk::enums::Align::TopLeft);
        main_src_txt.set_buffer(src_buf.clone());
        main_src_txt.wrap_mode(text::WrapMode::AtBounds, 0);
        /*src_buf.add_modify_callback(|pos, inserted, deleted, restyled, text| {
            
        });*/
        main_src_txt_wrapper.set_pad(5);
        main_src_txt_wrapper.set_margins(5,25,5,5);
        main_src_txt_wrapper.end();

        let mut main_controls_left = group::Flex::default().column();
        let mut col_left_row_lng = group::Flex::default().row();
        let mut lang_choice_from = fltk::menu::Choice::default().with_size(30, 10).with_label("From:").with_align(fltk::enums::Align::TopLeft);

        for lng in Lang::iter() {
            let name = LangNames::from_str(lng.as_ref()).unwrap();
            lang_choice_from.add(
                name.as_ref(),
                fltk::enums::Shortcut::None,
                fltk::menu::MenuFlag::Normal,
                {
                    let s = app_sender;
                    move |_b| {
                        s.send(AppEvent::SetSrcLang(lng.clone()));
                        s.send(AppEvent::Translate(true, false, false));
                    }
                },
            ); 
        }
        //let col_left_row1_tr_but1 = button::Button::default().with_label("@<->").with_size(20, 20);
        let mut lang_choice_to = fltk::menu::Choice::default().with_size(30, 10).with_label("To:").with_align(fltk::enums::Align::TopLeft);
        for lng in Lang::iter() {
            let name = LangNames::from_str(lng.as_ref()).unwrap();
            lang_choice_to.add(
                name.as_ref(),
                fltk::enums::Shortcut::None,
                fltk::menu::MenuFlag::Normal,
                {
                    let s = app_sender;
                    move |_b| {
                        s.send(AppEvent::SetTargetLang(lng.clone()));
                        s.send(AppEvent::Translate(true, false, false));
                    }
                },
            );
        }
        
        col_left_row_lng.set_margins(0,15,0,15);
        main_controls_left.fixed(&col_left_row_lng, 55);
        col_left_row_lng.end();

        let mut col_left_row_tr = group::Flex::default().row();

        let mut transl_choice = fltk::menu::Choice::default().with_size(30, 10).with_label("Translate with:").with_align(fltk::enums::Align::TopLeft);

        for transl_ch in GLOBAL_SETTINGS.translators.iter() {
            transl_choice.add(
                &*transl_ch.name,
                fltk::enums::Shortcut::None,
                fltk::menu::MenuFlag::Normal,
                {
                    let s = app_sender;
                    move |_b| {
                        s.send(AppEvent::SetTranslator(transl_ch.uid.clone()));
                        s.send(AppEvent::Translate(true, false, false));
                    }
                },
            );
        }

        let mut run_transl_btn_main = button::Button::default().with_label("Translate").with_size(20, 20);
        run_transl_btn_main.set_callback({
                let s = app_sender;
                move |_b| {
                    s.send(AppEvent::Translate(false, false, true));
                }
        });
        col_left_row_tr.fixed(&run_transl_btn_main, 150);

        col_left_row_tr.set_margins(0,15,0,0);
        main_controls_left.fixed(&col_left_row_tr, 40);
        col_left_row_tr.end();

        let mut col_left_row_dict = group::Flex::default().row();
        let mut dict_choice = fltk::menu::Choice::default().with_size(30, 10).with_label("Dictionary:").with_align(fltk::enums::Align::TopLeft);

        for dict_ch in GLOBAL_SETTINGS.dictionaries.iter() {
            dict_choice.add(
                &*dict_ch.name,
                fltk::enums::Shortcut::None,
                fltk::menu::MenuFlag::Normal,
                {
                    let s = app_sender;
                    move |_b| {
                        s.send(AppEvent::SetDict(dict_ch.uid.clone()));
                        s.send(AppEvent::RequestDictEntry(true, false, false));
                    }
                },
            );
        }
        let mut run_dict_btn_main = button::Button::default().with_label("Send to Dictionary").with_size(20, 20);
        run_dict_btn_main.set_callback({
                let s = app_sender;
                move |_b| {
                    s.send(AppEvent::SendToDict());
                }
        });
        col_left_row_dict.fixed(&run_dict_btn_main, 150);

        col_left_row_dict.set_margins(0,15,0,0);
        main_controls_left.fixed(&col_left_row_dict, 40);
        col_left_row_dict.end();

        let mut col_left_row_tts = group::Flex::default().row();
        let mut tts_choice = fltk::menu::Choice::default().with_size(50, 10).with_label("TTS engine/voice:").with_align(fltk::enums::Align::TopLeft);

        for qwe in GLOBAL_SETTINGS.tts_engines.iter() {
            for tts_voice in qwe.voices.iter() {
                let name = format!("{}-{}", &*qwe.name, tts_voice);
                tts_choice.add(
                    &name,
                    fltk::enums::Shortcut::None,
                    fltk::menu::MenuFlag::Normal,
                    {
                        let s = app_sender;
                        move |_b| {
                            s.send(AppEvent::SetTTSEngine(qwe.uid.clone(), tts_voice.clone()));
                        }
                    },
                );
            }
        }

        let mut _col1_row1_tts_but = button::Button::default().with_size(10, 10).with_label("Play");
        /*if let Ok(image) = PngImage::load(working_dir.join(r"icons\play.png").to_str().unwrap_or("")) {
            _col1_row1_tts_but.set_image(Some(image));
            _col1_row1_tts_but.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }*/
        _col1_row1_tts_but.set_callback({
                let s = app_sender;
                move |_b| {
                    s.send(AppEvent::TTString());
                }
        });
    

        let mut prnn_choice = fltk::menu::Choice::default().with_size(50, 10).with_label("Pronunciation:").with_align(fltk::enums::Align::TopLeft);

        for qwe in GLOBAL_SETTINGS.prnn_sources.iter() {
            let name = format!("{}", &*qwe.name);
            prnn_choice.add(
                &name,
                fltk::enums::Shortcut::None,
                fltk::menu::MenuFlag::Normal,
                {
                    let s = app_sender;
                    move |_b| {
                        s.send(AppEvent::SetPRNNEngine(qwe.uid.clone()));
                    }
                },
            );
        }

        let mut _col1_row2_prnn_but = button::Button::default().with_size(10, 10).with_label("Play");
        /*if let Ok(image) = PngImage::load(working_dir.join(r"icons\play.png").to_str().unwrap_or("")) {
            _col1_row2_prnn_but.set_image(Some(image));
            _col1_row2_prnn_but.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }*/
        _col1_row2_prnn_but.set_callback({
                let s = app_sender;
                move |_b| {
                    s.send(AppEvent::PRNNString());
                }
        });
        
        col_left_row_tts.fixed(&_col1_row1_tts_but, 55); //25
        col_left_row_tts.fixed(&_col1_row2_prnn_but, 55);

        col_left_row_tts.set_margins(0,15,0,0);
        main_controls_left.fixed(&col_left_row_tts, 40);
        col_left_row_tts.end();


        let mut col_left_row_fav = group::Flex::default().row();
        let mut fav_button_main = button::Button::new(51, 5, 18, 18, "").with_label("Add to favorites");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\fav.png").to_str().unwrap_or("")) {
            fav_button_main.set_image(Some(image));
            fav_button_main.set_align(fltk::enums::Align::Inside | fltk::enums::Align::Left | fltk::enums::Align::ImageNextToText);
        }
        let mut refresh_button_main = button::Button::new(51, 5, 18, 18, "").with_label("Refresh");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\refresh.png").to_str().unwrap_or("")) {
            refresh_button_main.set_image(Some(image));
            refresh_button_main.set_align(fltk::enums::Align::Inside | fltk::enums::Align::Left | fltk::enums::Align::ImageNextToText);
        }
        col_left_row_fav.fixed(&fav_button_main, 135);
        col_left_row_fav.fixed(&refresh_button_main, 100);
        col_left_row_fav.set_margins(0,15,0,0);
        main_controls_left.fixed(&col_left_row_fav, 40);
        col_left_row_fav.end();


        main_controls_left.set_margins(5,15,5,5);
        main_flex_left.fixed(&main_controls_left, 240);
        main_controls_left.end();

        //TABS
        let mut col_main_tabs = group::Flex::default().row().with_pos(5, 0);
        let mut tab = group::Tabs::default().with_size(100, 50); //::default_fill not working in debug mode

        let history_tab = group::Flex::default_fill().with_label("Recent history\t").column();
        let mut history_browser_wrapper = group::Flex::default().row();
        let mut browser = browser::HoldBrowser::new(0, 20, 200, 200, None);
        browser.set_column_widths(&[100, 100]);
        browser.set_column_char('\t');
        history_browser_wrapper.set_pad(5);
        history_browser_wrapper.set_margin(5);
        history_browser_wrapper.end();
        history_tab.end();

        let fav_tab = group::Flex::default_fill().with_label("Favorites\t").column();
        let mut fav_browser_wrapper = group::Flex::default().row();
        let mut fav_browser = browser::HoldBrowser::new(0, 20, 200, 200, None);
        fav_browser.set_column_widths(&[100, 100]);
        fav_browser.set_column_char('\t');
        fav_browser_wrapper.set_pad(5);
        fav_browser_wrapper.set_margin(5);
        fav_browser_wrapper.end();
        fav_tab.end();

        tab.end();
        tab.auto_layout();

        main_flex_left.fixed(&col_main_tabs, 200);
        col_main_tabs.set_pad(5);
        col_main_tabs.set_margins(5,25,5,10);
        col_main_tabs.end();
        //TABS END

        main_flex_left.end();

        //SECOND COLUMN
        let mut main_flex_right = group::Flex::new(400,0,400,585,None);
        main_flex_right.set_type(group::FlexType::Column);
        
        let mut main_transl_txt = text::TextDisplay::default().with_label("Translation:").with_align(fltk::enums::Align::TopLeft);
        main_transl_txt.set_buffer(translation_buf.clone());
        main_transl_txt.wrap_mode(text::WrapMode::AtBounds, 0);
        //main_transl_txt.set_color(enums::Color::from_hex_str(&GLOBAL_SETTINGS.text_bg_color).unwrap_or(enums::Color::from_hex(0xF0F0F0)));

        let mut main_dict_txt = text::TextDisplay::default().with_label("Dictionary entry:").with_align(fltk::enums::Align::TopLeft);
        main_dict_txt.set_buffer(dict_buf.clone());
        main_dict_txt.wrap_mode(text::WrapMode::AtBounds, 0);
        //main_dict_txt.above_of(&dict_assets_browser, 20);
        //main_dict_txt.set_color(enums::Color::from_hex_str(&GLOBAL_SETTINGS.text_bg_color).unwrap_or(enums::Color::from_hex(0xF0F0F0)));
        main_flex_right.fixed(&main_dict_txt, 125);

        let mut dict_assets_browser = browser::HoldBrowser::new(0, 0, 200, 200, None).with_label("Pronunciations (cached), click to play:").with_align(fltk::enums::Align::TopLeft);
        dict_assets_browser.set_column_widths(&[100, 100]);
        dict_assets_browser.set_column_char('\t');
        main_flex_right.fixed(&dict_assets_browser, 125);

        let mut tts_browser = browser::HoldBrowser::new(0, 0, 200, 200, None).with_label("TTS (cached), click to play:").with_align(fltk::enums::Align::TopLeft);
        tts_browser.set_column_widths(&[100, 100]);
        tts_browser.set_column_char('\t');
        main_flex_right.fixed(&tts_browser, 125);

        main_flex_right.set_pad(20);
        main_flex_right.set_margins(5,25,5,10);

        main_flex_right.end();
        //SECOND COLUMN END

        main_flex_wrapper_inner.end();

        //TODO: status bar
        let mut status_frame_main = Frame::default().with_label("").with_align(fltk::enums::Align::Inside | fltk::enums::Align::Left);
        status_frame_main.set_label_size(GLOBAL_SETTINGS.ui_font_size);
        main_flex_wrapper.fixed(&status_frame_main, 15);
        
        main_flex_wrapper.end();

        main_win.make_resizable(true);
        main_win.set_border(true);
        main_win.resizable(&main_win);

        main_win.end();
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\tray_icon.png").to_str().unwrap_or("")){
            main_win.set_icon(Some(image));
        }
        ////////////////////---------------END MAIN WIN---------------/////////////////////
        ////////////////////---------------END UI---------------/////////////////////

        win_popup.hotspot(&close_button);
        win_popup_dict.hotspot(&close_button_dict);
        //trying to hide popup window at startup...
        win_popup.show();
        win_popup_dict.show();
        win_popup.set_opacity(GLOBAL_SETTINGS.popup_opacity); //This should be called on a shown window
        win_popup_dict.set_opacity(GLOBAL_SETTINGS.popup_opacity); //0.8
        //fltk bug? panic or high cpu usage when we trying to hide the windows. spawning a new thread and hiding them inside it works
        //TODO: should be called after app's event loop run?
        std::thread::spawn({
            let win_popup = win_popup.clone();
            let win_popup_dict = win_popup_dict.clone();
            move || {
                win_popup.platform_hide();
                win_popup_dict.platform_hide(); //doesn't work and causing cpu utilization issue, w/o spawning separate thread
                app::awake();
            }
        });

        browser.set_callback({
            move |b| {
                // FLTK browser indices are 1-based
                let selected_index = b.value(); 
                if selected_index > 0 && let Some(text) = b.text(selected_index) {
                    println!("Selected: {} at index {}", text, selected_index);
                    unsafe { //Type correctness (selected_index: i32) is insured by the developer
                        if let Some(d) = b.data::<i32>(selected_index) {
                            println!("Selected: {}", d);
                            app_sender.send(AppEvent::UpdateTTState(d));
                        }
                    }
                }
            }
        });
        fav_browser.set_callback({
            move |b| {
                // FLTK browser indices are 1-based
                let selected_index = b.value(); 
                if selected_index > 0 && let Some(text) = b.text(selected_index) {
                    println!("Selected: {} at index {}", text, selected_index);
                    unsafe { //Type correctness (selected_index: i32) is insured by the developer
                        if let Some(d) = b.data::<i32>(selected_index) {
                            println!("Selected: {}", d);
                            app_sender.send(AppEvent::UpdateTTState(d));
                        }
                    }
                }
            }
        });
        tts_browser.set_callback({
            move |b| {
                // FLTK browser indices are 1-based
                let selected_index = b.value(); 
                if selected_index > 0 && let Some(text) = b.text(selected_index) {
                    println!("Selected: {} at index {}", text, selected_index);
                    unsafe { //Type correctness (selected_index: i32) is insured by the developer
                        if let Some(d) = b.data::<String>(selected_index) {
                            println!("Selected: {}", d);
                            app_sender.send(AppEvent::TTSPlay(d));
                        }
                    }
                }
            }
        });
        dict_assets_browser.set_callback({
            move |b| {
                // FLTK browser indices are 1-based
                let selected_index = b.value(); 
                if selected_index > 0 && let Some(text) = b.text(selected_index) {
                    println!("Selected: {} at index {}", text, selected_index);
                    unsafe { //Type correctness (selected_index: i32) is insured by the developer
                        if let Some(d) = b.data::<String>(selected_index) {
                            println!("Selected: {}", d);
                            app_sender.send(AppEvent::TTSPlay(d));
                        }
                    }
                }
            }
        });

        //TRAY ICON EVENTS
        //TODO bug: sometimes, after mouse hover on the tray icon, something starts triggering the app's main loop infinitely. difficult to reproduce, cpu load 0,12%, 7KK cycles delta
        std::thread::spawn({
            move || loop {
                println!("TrayIconEvent loop");
                if let Ok(e) = TrayIconEvent::receiver().recv() {
                    app_sender.send(AppEvent::TrayIcon(e));
                }
                /*if let Ok(e) = TrayIconEvent::receiver().recv() {
                    match e {
                        //TODO
                        TrayIconEvent::DoubleClick{..} => {
                            println!("{:?}", e);
                            app_sender.send(AppEvent::ShowPopup(false));
                        }
                        TrayIconEvent::Click {..} => {
                            
                        }
                        _ =>  {}
                    }
                }   */
            }
        });
        //TRAY MENU EVENTS
        std::thread::spawn({
            move || loop {
                println!("TrayMenuEvent loop");
                if let Ok(e) = MenuEvent::receiver().recv() {
                    app_sender.send(AppEvent::TrayMenuEvent(e));
                }
        }});

        //IMPL RESIZING/DRAGGING BHVR FOR BORDELESS WINDOW
        let is_inner = Rc::new(RefCell::new(false));
        let is_inner_dict = Rc::new(RefCell::new(false));
        frame.handle({
            let mut win_popup = win_popup.clone();
            let is_inner = Rc::clone(&is_inner);
            move |_t, event| {
                borderless_win_frame_handler(event, &mut win_popup, &is_inner)
            }
        });
        frame_dict.handle({
            let mut win_popup_dict = win_popup_dict.clone();
            let is_inner_dict = Rc::clone(&is_inner_dict);
            move |_t, event| {
                borderless_win_frame_handler(event, &mut win_popup_dict, &is_inner_dict)
            }
        });
    
        win_popup.handle({
            //popup borderless window resizing and dragging
            let mut coords = BLWCoords {
                x: 0,
                y: 0,
                x_start: 0,
                y_start: 0,
                initial_window_height: 0,
                initial_window_width: 0,
                init_on_border_left: false,
                init_on_border_right: false,
                init_on_border_top: false,
                init_on_border_bottom: false,
            };
            let is_inner = Rc::clone(&is_inner);

            move |window, event| {
                borderless_win_handler(window, event, &mut coords, &is_inner)
            }
        });
        win_popup_dict.handle({
            //popup borderless window resizing and dragging
            let mut coords = BLWCoords {
                x: 0,
                y: 0,
                x_start: 0,
                y_start: 0,
                initial_window_height: 0,
                initial_window_width: 0,
                init_on_border_left: false,
                init_on_border_right: false,
                init_on_border_top: false,
                init_on_border_bottom: false,
            };
            let is_inner_dict = Rc::clone(&is_inner_dict);

            move |window, event| {
                borderless_win_handler(window, event, &mut coords, &is_inner_dict)
            }
        });

        //WIDGET CALLBACKS
        close_button.set_callback({
            let win_popup = win_popup.clone();
            move |_| {
                win_popup.platform_hide();
            }
        });
        close_button_dict.set_callback({
            let win_popup_dict = win_popup_dict.clone();
            move |_| {
                win_popup_dict.platform_hide();
            }
        });
        fav_button.set_callback({
            let s = app_sender;
            move |_| {
                s.send(AppEvent::ToggleFav(None, false));
            }
        });
        fav_button_dict.set_callback({
            let s = app_sender;
            move |_| {
                s.send(AppEvent::ToggleFav(None, true));
            }
        });
        fav_button_main.set_callback({
            let s = app_sender;
            move |_| {
                s.send(AppEvent::ToggleFav(None, false));
            }
        });
        tts_button.set_callback({
            let s = app_sender;
            move |_| {
                s.send(AppEvent::TTString());
            }
        });
        prnn_button_dict.set_callback({
            let s = app_sender;
            move |_| {
                s.send(AppEvent::PRNNString());
            }
        });
        
        dict_button.set_callback({
            let win_popup = win_popup.clone();
            let mut win_popup_dict = win_popup_dict.clone();
            let s = app_sender;
            move |_button| {
                win_popup_dict.set_pos(win_popup.x_root() + 50, win_popup.y_root() + 50);
                win_popup_dict.set_on_top();
                win_popup_dict.platform_show();
                s.send(AppEvent::SendToDict());
            }
        });

        refresh_button.set_callback({
            let s = app_sender;
            move |_button| {
                s.send(AppEvent::Translate(false, true, false));
            }
        });
        refresh_button_dict.set_callback({
            let s = app_sender;
            move |_button| {
                s.send(AppEvent::RequestDictEntry(false, true, false));
            }
        });
        refresh_button_main.set_callback({
            let s = app_sender;
            move |_button| {
                s.send(AppEvent::Translate(true, true, false));
                s.send(AppEvent::RequestDictEntry(true, true, false));
            }
        });
        open_button.set_callback({
            let s = app_sender;
            let mut main_win = main_win.clone();
            move |_| {
                main_win.show();
            }
        });
        open_button_dict.set_callback({
            let s = app_sender;
            let mut main_win = main_win.clone();
            move |_| {
                main_win.show();
            }
        });
        
        win_popup.clone().set_callback(|w| {
            // We intercept the closing of the window here
            w.platform_hide();
        });
        win_popup_dict.clone().set_callback(|w| {
            // We intercept the closing of the window here
            w.platform_hide();
        });



        AppView {
            //app_sender,

            src_buf,
            src_dict_buf,
            translation_buf,
            dict_buf,
            waiting_buf,
            error_buf,

            is_processing: Arc::new(AtomicBool::new(false)),
            
            txt_popup: txt,
            txt_popup_dict: txt_dict,
            txt_main: main_transl_txt,
            txt_dict_main: main_dict_txt,
            title_frame,
            title_frame_dict,
            //status_frame,
            status_frame_main,
            //status_frame_dict,

            src: "".to_string(),
            src_dict: "".to_string(),
            transl_browser: browser,
            fav_browser,
            tts_browser,
            dict_assets_browser,

            win_popup,
            win_popup_dict,
            main_win,

            translator_buttons,
            dict_buttons,
            fav_button,
            fav_button_dict,
            fav_button_main,

            lang_choice_from,
            lang_choice_to,
            dict_choice,
            transl_choice,
            tts_choice,
            prnn_choice,
        }
    }

    pub fn set_waiting(&mut self) {
        //TODO: is_dict
        self.is_processing.store(true, Ordering::Relaxed);
        self.txt_popup.set_buffer(self.waiting_buf.clone());
        self.txt_main.set_buffer(self.waiting_buf.clone());
        self.txt_popup_dict.set_buffer(self.waiting_buf.clone());
        self.txt_dict_main.set_buffer(self.waiting_buf.clone());
        self.run_anim();
    }

    pub fn set_ready(&mut self) {
        self.is_processing.store(false, Ordering::Relaxed);
        self.txt_popup.set_buffer(self.translation_buf.clone());
        self.txt_main.set_buffer(self.translation_buf.clone());
        self.txt_popup_dict.set_buffer(self.dict_buf.clone());
        self.txt_dict_main.set_buffer(self.dict_buf.clone());
    }
    pub fn set_error(&mut self, text: &str, is_dict: bool) {
        self.error_buf.set_text(text);
        if !is_dict {
            self.txt_popup.set_buffer(self.error_buf.clone());
            self.txt_main.set_buffer(self.error_buf.clone());
        } else {
            self.txt_popup_dict.set_buffer(self.error_buf.clone());
            self.txt_dict_main.set_buffer(self.error_buf.clone());
        }
        //TODO: red highlight
    }

    pub fn clear_ui(&mut self, is_dict: bool) {
        println!("clear_ui");
        self.set_status("", false, false);
        if !is_dict {
            self.title_frame.set_label("");
            self.translation_buf.set_text("");
        } else {
            self.title_frame_dict.set_label("");
            self.dict_buf.set_text("");
        }
    }
    

    pub fn update_ui(
        &mut self,
        state: UIState,
        is_new_source: bool
    ) {
        println!("update_ui");
        let UIState {src_text, tr_uid, translator, src, target, translation_text, is_fav} = state;

        if is_new_source {  
            self.src_buf.set_text(format!("{}\n", &src_text).as_str()); //new line is req bc fltk widget bug
            self.src = src_text;
        } else if src_text != self.src {
            return;
        }

        if let Some(uid) = tr_uid && let Some(ref name) = translator {
            self.set_translator(name, &uid);
        }
        
        if let Some(t_text) = translation_text {
            self.translation_buf.set_text(format!("{}\n", &t_text).as_str());
            self.set_ready();
        }

        if let Some(lang_from) = src && let Some(lang_to) = target && let Some(translator_name) = translator {
            let from = LangNames::from_str(lang_from.as_ref()).unwrap_or(LangNames::En);
            let to = LangNames::from_str(lang_to.as_ref()).unwrap_or(LangNames::En);
            let title_text = format!("{}->{} ({})", from.as_ref(), to.as_ref(), translator_name);
            self.title_frame.set_label(&title_text);
        }

        if let Some(is_fav) = is_fav {
            let working_dir = std::env::current_dir().unwrap();
            if is_fav {
                if let Ok(image) = PngImage::load(working_dir.join(r"icons\fav_filled.png").to_str().unwrap_or("")) {
                    self.fav_button.set_image(Some(image.clone()));
                    self.fav_button_main.set_image(Some(image));
                    self.fav_button_main.set_label("Remove from fav.");
                }
            } else {
                if let Ok(image) = PngImage::load(working_dir.join(r"icons\fav.png").to_str().unwrap_or("")) {
                    self.fav_button.set_image(Some(image.clone()));
                    self.fav_button_main.set_image(Some(image));
                    self.fav_button_main.set_label("Add to favorites");
                }
            }
        }
        
        app::redraw();
        app::awake();
    }

    pub fn update_ui_dict(&mut self, state: UIStateDict, is_new_source: bool) {
        println!("update_ui {is_new_source}");
        let UIStateDict {src_id, src_text_dict, dict_uid, dict_name, dict_text, is_fav} = state;

        if is_new_source {  
            self.src_dict_buf.set_text(format!("{}\n", &src_text_dict).as_str()); //new line is req bc fltk widget bug
            self.src_dict = src_text_dict;
        } else if src_text_dict != self.src_dict {
            return;
        }

        let _ = src_id;
        
        if let Some(uid) = dict_uid && let Some(ref name) = dict_name {
            self.set_dict(name, &uid);
        }

        if let Some(dict_text) = dict_text {
            self.set_ready();
            let text_chuncs = dsl_parse(&dict_text);
            //teal, red, green, blue, indigo
            let mut sbuf = fltk::text::TextBuffer::default();
            let style_a = fltk::text::StyleTableEntry {
                color: fltk::enums::Color::Black,
                font: fltk::enums::Font::Helvetica,
                size: 12,
            };
            let style_b = fltk::text::StyleTableEntry {
                color: fltk::enums::Color::Black,
                font: fltk::enums::Font::HelveticaBold,
                size: 12,
            };
            let style_c = fltk::text::StyleTableEntry {
                color: fltk::enums::Color::Red,
                font: fltk::enums::Font::Helvetica,
                size: 12,
            };
            let style_d = fltk::text::StyleTableEntry {
                color: fltk::enums::Color::DarkGreen,
                font: fltk::enums::Font::Helvetica,
                size: 12,
            };
            let style_e = fltk::text::StyleTableEntry {
                color: fltk::enums::Color::DarkBlue,
                font: fltk::enums::Font::Helvetica,
                size: 12,
            };
            let style_f = fltk::text::StyleTableEntry {
                color: fltk::enums::Color::from_hex(0x008080), //teal
                font: fltk::enums::Font::Helvetica,
                size: 12,
            };
            let style_g = fltk::text::StyleTableEntry {
                color: fltk::enums::Color::from_hex(0x4B0082), //indigo
                font: fltk::enums::Font::Helvetica,
                size: 12,
            };
        

            //sbuf.set_text("");
            let mut str_main = "".to_string();
            let mut str_f = "".to_string();
            for chunc in text_chuncs.iter() {
                
                str_main.push_str(&chunc.text);
                if &chunc.color == "red" {
                    str_f.push_str(&"C".repeat(chunc.text.len()));
                } else if &chunc.color == "green" {
                    str_f.push_str(&"D".repeat(chunc.text.len()));
                } else if &chunc.color == "blue" || &chunc.color == "darkblue" {
                    str_f.push_str(&"E".repeat(chunc.text.len()));
                } else if &chunc.color == "teal" {
                    str_f.push_str(&"F".repeat(chunc.text.len()));
                } else if &chunc.color == "indigo" {
                    str_f.push_str(&"G".repeat(chunc.text.len()));
                } else if chunc.is_bold {
                    str_f.push_str(&"B".repeat(chunc.text.len()));
                } else {
                    str_f.push_str(&"A".repeat(chunc.text.len()));
                }
                
            }

            self.dict_buf.set_text(&str_main);
            sbuf.set_text(&str_f);
            
            self.txt_popup_dict.unset_highlight_data(sbuf.clone());
            self.txt_popup_dict.set_highlight_data(sbuf.clone(), vec![style_a, style_b, style_c, style_d, style_e, style_f, style_g]);

            self.txt_dict_main.unset_highlight_data(sbuf.clone());
            self.txt_dict_main.set_highlight_data(sbuf.clone(), vec![style_a, style_b, style_c, style_d, style_e, style_f, style_g]);
        }

        //let from = LangNames::from_str(src.as_ref()).unwrap_or(LangNames::En);
        //let to = LangNames::from_str(target.as_ref()).unwrap_or(LangNames::En);
        //let title_text = format!("{}->{} ({})", from.as_ref(), to.as_ref(), dict_name);

        if let Some(dict_name) = dict_name {
            let title_text = format!("\"{}\" - {}", &self.src_dict, dict_name);
            self.title_frame_dict.set_label(&title_text);
        }

        if let Some(is_fav) = is_fav {
            let working_dir = std::env::current_dir().unwrap();
            if is_fav {
                if let Ok(image) = PngImage::load(working_dir.join(r"icons\fav_filled.png").to_str().unwrap_or("")) {
                    self.fav_button_dict.set_image(Some(image));
                }
            } else {
                if let Ok(image) = PngImage::load(working_dir.join(r"icons\fav.png").to_str().unwrap_or("")) {
                    self.fav_button_dict.set_image(Some(image));
                }
            }
        }

        app::redraw();
        app::awake();
    }


    fn run_anim(&mut self) {
        let arr = [".  ", ".. ", "...", " ..", "  .", "   "];
        //let arr = ["/", "--", "\\", "|", "/", "--"];
        let is_processing_clone = Arc::clone(&self.is_processing);
        let mut txt_buf_clone = self.waiting_buf.clone();
        std::thread::spawn({
            move || {
                println!("---animation loop start---");
                let mut is_processing_n = 0;
                while is_processing_clone.load(Ordering::Relaxed) {
                    is_processing_n += 1;
                    if is_processing_n > 4 {
                        is_processing_n = 0;
                    }
                    txt_buf_clone.set_text(format!("translating{}", arr[is_processing_n]).as_str());
                    app::awake();
                    thread::sleep(Duration::from_millis(100));
                }
                println!("---animation loop stop---");
            }
        });
    }

    pub fn set_tts_browser_data(&mut self, data: Vec<TTSource>) {
        self.tts_browser.clear();
        for item in data {
            self.tts_browser.add_with_data(item.voice.as_ref(), item.path);
        }
    }
    pub fn set_dict_assets_browser_data(&mut self, data: Vec<PRNNSource>) {
        self.dict_assets_browser.clear();
        for item in data {
            self.dict_assets_browser.add_with_data(item.service.as_ref(), item.path);
        }
    }

    pub fn update_history_browser(&mut self, data: Vec<TranslSource>) {
        self.transl_browser.clear();
        for item in data {
            self.transl_browser.add_with_data(item.text.as_str(), item.id);
        }
        
    }
    pub fn update_fav_browser(&mut self, data: Vec<TranslSource>) {
        self.fav_browser.clear();
        for item in data {
            self.fav_browser.add_with_data(item.text.as_str(), item.id);
        }
    }

    pub fn set_status(&mut self, text: &str, is_error: bool, is_dict: bool) {
        //self.status_frame.set_label(text);
        //self.status_frame_dict.set_label(text);
        self.status_frame_main.set_label(text);
        if is_error {
            self.set_error(text, is_dict);
            //TODO: check src_text
        }
        app::redraw();
        app::awake();
    }

    pub fn set_lang(&mut self, from: &str, to: &str) {
        if let Some(item) = self.lang_choice_from.find_item(from) {
            self.lang_choice_from.set_item(&item);
        }
        if let Some(item) = self.lang_choice_to.find_item(to) {
            self.lang_choice_to.set_item(&item);
        }
    }
    pub fn set_translator(&mut self, name: &str, uid: &str) {
        if let Some(item) = self.transl_choice.find_item(name) {
            self.transl_choice.set_item(&item);
        }
        
        for (key, value) in &mut self.translator_buttons{
            if key == uid {
                value.set(true);
            } else {
                value.set(false);
            }
        }
    }
    pub fn set_dict(&mut self, name: &str, uid: &str) {
        if let Some(item) = self.dict_choice.find_item(name) {
            self.dict_choice.set_item(&item);
        }
        for (key, value) in &mut self.dict_buttons{
            if key == uid {
                value.set(true);
            } else {
                value.set(false);
            }
        }
    }
    pub fn set_tts_engine(&mut self, tts: &str, voice: &str) {
        let name = format!("{}-{}", tts, voice);
        if let Some(item) = self.tts_choice.find_item(&name) {
            self.tts_choice.set_item(&item);
        }
    }
    pub fn set_prnn_service(&mut self, prnn: &str) {
        if let Some(item) = self.prnn_choice.find_item(prnn) {
            self.prnn_choice.set_item(&item);
        }
    }

    pub fn show_popup(&mut self, show_dict: bool, hotspot: bool) {
        let win = if show_dict { &mut self.win_popup_dict } else { &mut self.win_popup };
        if hotspot {
            let position = Mouse::get_mouse_position();
            match position {
                Mouse::Position { x, y } => {
                    win.set_pos(x, y); //TODO: screen edges, offsets, etc.
                },
                Mouse::Error => println!("Error getting mouse position"),
            }
        }
        //win.redraw();
        app::redraw();
        app::awake();
        win.set_on_top();
        win.platform_show();
    }
    
}

/*fn set_w_pos(mut win: DoubleWindow) {    
    println!("getting mouse position");
    win.set_pos(x, y);
}*/

fn borderless_win_frame_handler(event: enums::Event, win_popup: &mut DoubleWindow, is_inner: &Rc<RefCell<bool>>) -> bool {
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

fn borderless_win_handler(
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