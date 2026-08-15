use debug_print::{debug_println as dprintln};
use fltk::{
    prelude::*,
    window,
    window::DoubleWindow,
    text,
    button,
    group,
    image::PngImage,
    //image::IcoImage,
    frame::Frame,
    browser,
};

use strum::IntoEnumIterator;
//use mouse_position::mouse_position::{Mouse};
use crate::types::{
    AppEvent, Lang,
};


use super::GLOBAL_SETTINGS;
use super::UICONFIG;
use super::t;


pub struct MainWinView {
    pub window: DoubleWindow,

    pub transl_browser: fltk::browser::HoldBrowser,
    pub fav_browser: fltk::browser::HoldBrowser,
    pub tts_browser: fltk::browser::HoldBrowser,
    pub dict_assets_browser: fltk::browser::HoldBrowser,
    pub fav_button_main: button::Button,

    pub lang_choice_from: fltk::menu::Choice,
    pub lang_choice_to: fltk::menu::Choice,
    pub dict_choice: fltk::menu::Choice,
    pub transl_choice: fltk::menu::Choice,
    pub tts_choice: fltk::menu::Choice,
    pub prnn_choice: fltk::menu::Choice,

    pub txt_main: text::TextDisplay,
    pub txt_dict_main: text::TextDisplay,
    pub main_src_txt: text::TextEditor,
    pub status_frame_main: Frame,
}

impl MainWinView {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>) -> Self {

        let working_dir = std::env::current_dir().unwrap();
        //let text_bg_color_popup = enums::Color::from_hex_str(&GLOBAL_SETTINGS.text_bg_color_popup).unwrap_or(enums::Color::from_hex(0xF0F0F0));

    	////////////////////---------------BEGIN MAIN WIN---------------/////////////////////        
        let mut main_win = window::Window::default().with_size(800, 600).with_label("rTranslate");
        let mut main_flex_wrapper = group::Flex::new(0,0,800,600,None);
        main_flex_wrapper.set_type(group::FlexType::Column);
        let main_flex_wrapper_inner = group::Flex::default().row();

        let mut main_flex_left = group::Flex::new(0,0,400,585,None);
        main_flex_left.set_type(group::FlexType::Column);

        let mut main_src_txt_wrapper = group::Flex::default().column();
        let mut main_src_txt = text::TextEditor::default().with_label(t!(source_text)).with_align(fltk::enums::Align::TopLeft);
        main_src_txt.set_text_size(GLOBAL_SETTINGS.text_font_size);
        //main_src_txt.set_buffer(src_buf.clone());
        main_src_txt.wrap_mode(text::WrapMode::AtBounds, 0);
        /*src_buf.add_modify_callback(|pos, inserted, deleted, restyled, text| {
            
        });*/
        main_src_txt_wrapper.set_pad(5);
        main_src_txt_wrapper.set_margins(5,25,5,5);
        main_src_txt_wrapper.end();

        let mut main_controls_left = group::Flex::default().column();
        let mut col_left_row_lng = group::Flex::default().row();
        let mut lang_choice_from = fltk::menu::Choice::default().with_size(30, 10).with_label(t!(from)).with_align(fltk::enums::Align::TopLeft);

        for lng in Lang::iter() {
            lang_choice_from.add(
                lng.clone().name(),
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
        let mut lang_choice_to = fltk::menu::Choice::default().with_size(30, 10).with_label(t!(to)).with_align(fltk::enums::Align::TopLeft);
        for lng in Lang::iter() {
            lang_choice_to.add(
                lng.clone().name(),
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

        let mut transl_choice = fltk::menu::Choice::default().with_size(30, 10).with_label(t!(translate_with)).with_align(fltk::enums::Align::TopLeft);

        for transl_ch in GLOBAL_SETTINGS.translators.iter() {
            transl_choice.add(
                &transl_ch.name,
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

        let mut run_transl_btn_main = button::Button::default().with_label(t!(translate_refresh)).with_size(20, 20);
        run_transl_btn_main.set_callback({
                let s = app_sender;
                move |_b| {
                    s.send(AppEvent::Translate(false, true, true));
                }
        });
        col_left_row_tr.fixed(&run_transl_btn_main, 150);

        col_left_row_tr.set_margins(0,15,0,0);
        main_controls_left.fixed(&col_left_row_tr, 40);
        col_left_row_tr.end();

        let mut col_left_row_dict = group::Flex::default().row();
        let mut dict_choice = fltk::menu::Choice::default().with_size(30, 10).with_label(t!(dictionary)).with_align(fltk::enums::Align::TopLeft);

        for dict_ch in GLOBAL_SETTINGS.dictionaries.iter() {
            dict_choice.add(
                &dict_ch.name,
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
        let mut run_dict_btn_main = button::Button::default().with_label(t!(send_to_dict)).with_size(20, 20);
        run_dict_btn_main.set_callback({
                let s = app_sender;
                move |_b| {
                    s.send(AppEvent::RequestDictEntry(false, true, true));
                }
        });
        col_left_row_dict.fixed(&run_dict_btn_main, 150);

        col_left_row_dict.set_margins(0,15,0,0);
        main_controls_left.fixed(&col_left_row_dict, 40);
        col_left_row_dict.end();

        let mut col_left_row_tts = group::Flex::default().row();
        let mut tts_choice = fltk::menu::Choice::default().with_size(50, 10).with_label(t!(tts_engine_voice)).with_align(fltk::enums::Align::TopLeft);

        for srvc in GLOBAL_SETTINGS.tts_services.iter() {
            for tts_voice in srvc.voices.iter() {
                let name = format!("{}-{}", &*srvc.name, tts_voice);
                tts_choice.add(
                    &name,
                    fltk::enums::Shortcut::None,
                    fltk::menu::MenuFlag::Normal,
                    {
                        let s = app_sender;
                        move |_b| {
                            s.send(AppEvent::SetTTSEngine(srvc.uid.clone(), tts_voice.clone()));
                        }
                    },
                );
            }
        }

        let mut _col1_row1_tts_but = button::Button::default().with_size(10, 10).with_label(t!(play));
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
    

        let mut prnn_choice = fltk::menu::Choice::default().with_size(50, 10).with_label(t!(pronunciation)).with_align(fltk::enums::Align::TopLeft);

        for qwe in GLOBAL_SETTINGS.prnn_services.iter() {
            let name = (*qwe.name).to_string();
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

        let mut _col1_row2_prnn_but = button::Button::default().with_size(10, 10);
        //TODO! Play and Update buttons
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\download.png").to_str().unwrap_or("")) {
            _col1_row2_prnn_but.set_image(Some(image));
            _col1_row2_prnn_but.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }
        _col1_row2_prnn_but.set_callback({
                let s = app_sender;
                move |_b| {
                    s.send(AppEvent::PRNNString(true));
                }
        });
        
        col_left_row_tts.fixed(&_col1_row1_tts_but, 55); //25
        col_left_row_tts.fixed(&_col1_row2_prnn_but, 25);
        //col_left_row_tts.fixed(&_col1_row2_prnn_refresh_but, 25);

        col_left_row_tts.set_margins(0,15,0,0);
        main_controls_left.fixed(&col_left_row_tts, 40);
        col_left_row_tts.end();


        let mut col_left_row_fav = group::Flex::default().row();
        let mut fav_button_main = button::Button::new(51, 5, 18, 18, "").with_label(t!(add_to_fav));
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\fav.png").to_str().unwrap_or("")) {
            fav_button_main.set_image(Some(image));
            fav_button_main.set_align(fltk::enums::Align::Inside | fltk::enums::Align::Left | fltk::enums::Align::ImageNextToText);
        }
        /*let mut refresh_button_main = button::Button::new(51, 5, 18, 18, "").with_label("Refresh");
        if let Ok(image) = PngImage::load(working_dir.join(r"icons\refresh.png").to_str().unwrap_or("")) {
            refresh_button_main.set_image(Some(image));
            refresh_button_main.set_align(fltk::enums::Align::Inside | fltk::enums::Align::Left | fltk::enums::Align::ImageNextToText);
        }*/
        col_left_row_fav.fixed(&fav_button_main, 135);
        //col_left_row_fav.fixed(&refresh_button_main, 100);
        col_left_row_fav.set_margins(0,15,0,0);
        main_controls_left.fixed(&col_left_row_fav, 40);
        col_left_row_fav.end();


        main_controls_left.set_margins(5,15,5,5);
        main_flex_left.fixed(&main_controls_left, 240);
        main_controls_left.end();

        //TABS
        let mut col_main_tabs = group::Flex::default().row().with_pos(5, 0);
        let mut tab = group::Tabs::default().with_size(100, 50); //::default_fill not working in debug mode

        let history_tab = group::Flex::default_fill().with_label(&format!("{}\t", t!(recent_history))).column();
        let mut history_browser_wrapper = group::Flex::default().row();
        let mut browser = browser::HoldBrowser::new(0, 20, 200, 200, None);
        browser.set_column_widths(&[100, 100]);
        browser.set_column_char('\t');
        history_browser_wrapper.set_pad(5);
        history_browser_wrapper.set_margin(5);
        history_browser_wrapper.end();
        history_tab.end();

        let fav_tab = group::Flex::default_fill().with_label(&format!("{}\t", t!(favorites))).column();
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
        
        let mut main_transl_txt = text::TextDisplay::default().with_label(t!(translation)).with_align(fltk::enums::Align::TopLeft);
        main_transl_txt.set_text_size(GLOBAL_SETTINGS.text_font_size);
        //
        main_transl_txt.wrap_mode(text::WrapMode::AtBounds, 0);

        let mut main_dict_txt = text::TextDisplay::default().with_label(t!(dictionary_entry)).with_align(fltk::enums::Align::TopLeft);
        main_dict_txt.set_text_size(GLOBAL_SETTINGS.text_font_size);
        //
        main_dict_txt.wrap_mode(text::WrapMode::AtBounds, 0);
        //main_dict_txt.above_of(&dict_assets_browser, 20);
        main_flex_right.fixed(&main_dict_txt, 125);

        let mut dict_assets_browser = browser::HoldBrowser::new(0, 0, 200, 200, None).with_label(t!(prnn_cached)).with_align(fltk::enums::Align::TopLeft);
        dict_assets_browser.set_column_widths(&[100, 100]);
        dict_assets_browser.set_column_char('\t');
        main_flex_right.fixed(&dict_assets_browser, 125);

        let mut tts_browser = browser::HoldBrowser::new(0, 0, 200, 200, None).with_label(t!(tts_cached)).with_align(fltk::enums::Align::TopLeft);
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


        browser.set_callback({
            move |b| {
                // FLTK browser indices are 1-based
                let selected_index = b.value(); 
                if selected_index > 0 && let Some(text) = b.text(selected_index) {
                    dprintln!("Selected: {} at index {}", text, selected_index);
                    unsafe { //Type correctness (selected_index: i32) is insured by the developer
                        if let Some(d) = b.data::<i32>(selected_index) {
                            dprintln!("Selected: {}", d);
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
                    dprintln!("Selected: {} at index {}", text, selected_index);
                    unsafe { //Type correctness (selected_index: i32) is insured by the developer
                        if let Some(d) = b.data::<i32>(selected_index) {
                            dprintln!("Selected: {}", d);
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
                    dprintln!("Selected: {} at index {}", text, selected_index);
                    unsafe { //Type correctness (selected_index: i32) is insured by the developer
                        if let Some(d) = b.data::<String>(selected_index) {
                            dprintln!("Selected: {}", d);
                            let filename = format!("{}.ogg", d);
                            app_sender.send(AppEvent::TTSPlay(filename));
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
                    dprintln!("Selected: {} at index {}", text, selected_index);
                    unsafe { //Type correctness (selected_index: i32) is insured by the developer
                        if let Some(d) = b.data::<String>(selected_index) {
                            dprintln!("Selected: {}", d);
                            //let filename = format!("{}.ogg", d);
                            app_sender.send(AppEvent::TTSPlay(d));
                        }
                    }
                }
            }
        });

        fav_button_main.set_callback({
            let s = app_sender;
            move |_| {
                s.send(AppEvent::ToggleFav(false));
            }
        });
        
        main_win.resize(100, 100, UICONFIG.main_window_w, UICONFIG.main_window_h);

		MainWinView {
            window: main_win,
            txt_main: main_transl_txt,
            txt_dict_main: main_dict_txt,
            main_src_txt,
            
            //status_frame,
            status_frame_main,

            transl_browser: browser,
            fav_browser,
            tts_browser,
            dict_assets_browser,

            fav_button_main,

            lang_choice_from,
            lang_choice_to,
            dict_choice,
            transl_choice,
            tts_choice,
            prnn_choice,
		}
    }
}
