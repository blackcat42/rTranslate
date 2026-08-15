use debug_print::{debug_println as dprintln};
use fltk::{
    app,
    prelude::*,
    //window,
    //window::DoubleWindow,
    text,
    enums,
    //browser,
    //button,
    //group,
    image::PngImage,
    //image::IcoImage,
    //frame::Frame,
};

//use std::rc::Rc;
//use std::cell::RefCell;
use std::time::Duration;
use std::thread;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
//use std::str::FromStr;
use std::convert::AsRef;
//use std::collections::HashMap;

//use strum::IntoEnumIterator;
//use mouse_position::mouse_position::{Mouse};
use crate::types::{
    AppEvent, Lang, TTSource, PRNNSource, TranslSource, 
    UIState, UIStateDict,
};

use crate::bbcode::{dsl_parse};
/*use crate::utils::helpers::{
    borderless_win_handler, 
    borderless_win_frame_handler
};
use crate::utils::widgets::TooltipExt;*/

use super::GLOBAL_SETTINGS;
use super::UICONFIG;
use super::t;

mod dict_popup;
use dict_popup::{DictPopupView};
mod transl_popup;
use transl_popup::{TranslPopupView};
mod main_win;
use main_win::{MainWinView};

pub struct AppView {
    //app_sender: fltk::app::Sender<AppEvent>,
    pub main_win: MainWinView,
    pub transl_popup: TranslPopupView,
    pub dict_popup: DictPopupView,
    pub src_buf: text::TextBuffer,
    pub src_dict_buf: text::TextBuffer,
    translation_buf: text::TextBuffer,
    dict_buf: text::TextBuffer,
    waiting_buf: text::TextBuffer,
    error_buf: text::TextBuffer,
    is_processing: Arc<AtomicBool>,    
}

impl AppView {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>) -> Self {

        //let working_dir = std::env::current_dir().unwrap();

        //GLOBAL COLORS
        let win_bg_color = enums::Color::from_hex_str(&GLOBAL_SETTINGS.win_bg_color).unwrap_or(enums::Color::from_hex(0xD6CFC6));
        let win_bg_color_rgb = win_bg_color.to_rgb();
        app::set_background_color(win_bg_color_rgb.0, win_bg_color_rgb.1, win_bg_color_rgb.2);

        let text_bg_color_main = enums::Color::from_hex_str(&GLOBAL_SETTINGS.text_bg_color_main).unwrap_or(enums::Color::from_hex(0xFFFFFF));
        let text_bg_color_main_rgb = text_bg_color_main.to_rgb();
        app::set_background2_color(text_bg_color_main_rgb.0, text_bg_color_main_rgb.1, text_bg_color_main_rgb.2);


        
        let dict_buf = text::TextBuffer::default();
        let src_buf = text::TextBuffer::default();
        let src_dict_buf = text::TextBuffer::default();
        let translation_buf = text::TextBuffer::default();
        let waiting_buf = text::TextBuffer::default();
        let error_buf = text::TextBuffer::default();

        ////////////////////---------------BEGIN UI---------------/////////////////////

        let mut tooltip_win = fltk::window::OverlayWindow::default().with_size(160, 20);
        let mut tooltip_text = fltk::frame::Frame::default().size_of_parent().center_of_parent();
        tooltip_text.set_frame(fltk::enums::FrameType::BorderBox);
        tooltip_text.set_color(fltk::enums::Color::from_rgb(255, 255, 191));
        tooltip_text.set_label("");
        tooltip_win.end();
        tooltip_win.set_border(false);
        tooltip_win.set_override();
        tooltip_win.hide();

        let mut main_win = MainWinView::new(app_sender);
        let dict_popup = DictPopupView::new(app_sender, main_win.window.clone(), tooltip_win.clone(), tooltip_text.clone());
        let transl_popup = TranslPopupView::new(app_sender, main_win.window.clone(), dict_popup.win_popup_dict.clone(), tooltip_win.clone(), tooltip_text.clone());
        

        main_win.main_src_txt.set_buffer(src_buf.clone());
        main_win.txt_main.set_buffer(translation_buf.clone());
        main_win.txt_dict_main.set_buffer(dict_buf.clone());

        ////////////////////---------------END UI---------------/////////////////////

        
        
        //fltk bug? panic or high cpu usage when we trying to hide the windows. spawning a new thread and hiding them inside it works
        //TODO: should be called after app's event loop run?
        /*std::thread::spawn({
            let win_popup = win_popup.clone();
            let win_popup_dict = win_popup_dict.clone();
            move || {
                win_popup.platform_hide();
                win_popup_dict.platform_hide(); //doesn't work and causing cpu utilization issue, w/o spawning separate thread
                app::awake();
            }
        });*/
        // !!!!!!!!!!!!!!!!!!!!!!!!! app::flush();
        //app::wait_for(0.0);
        
        
        /*app::add_timeout3(0.1, {
            let win_popup = win_popup.clone();
            let win_popup_dict = win_popup_dict.clone();
            move|_| {
                win_popup.platform_hide();
                win_popup_dict.platform_hide();
            }
        });*/

        AppView {
            //app_sender,
            dict_popup,
            transl_popup,
            main_win,
            src_buf,
            dict_buf,
            src_dict_buf,
            translation_buf,
            
            waiting_buf,
            error_buf,

            is_processing: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_waiting(&mut self, text: Option<String>, is_dict: bool) {
        self.is_processing.store(true, Ordering::Relaxed);
        if !is_dict {
            self.transl_popup.txt_popup.set_buffer(self.waiting_buf.clone());
            self.main_win.txt_main.set_buffer(self.waiting_buf.clone());      
        } else {
            self.dict_popup.txt_popup_dict.set_buffer(self.waiting_buf.clone());
            self.main_win.txt_dict_main.set_buffer(self.waiting_buf.clone());
        }
        self.run_anim(text);
    }

    pub fn set_ready(&mut self, error: Option<String>, is_dict: bool) {
        self.is_processing.store(false, Ordering::Relaxed);
        if let Some(err) = error {
            self.set_error(err.as_str(), is_dict);
            self.main_win.status_frame_main.set_label(err.as_str());
            //TODO: check src_text
        } else if !is_dict {
            self.transl_popup.txt_popup.set_buffer(self.translation_buf.clone());
            self.main_win.txt_main.set_buffer(self.translation_buf.clone());           
        } else {
            self.dict_popup.txt_popup_dict.set_buffer(self.dict_buf.clone());
            self.main_win.txt_dict_main.set_buffer(self.dict_buf.clone());
        }
    }
    pub fn set_error(&mut self, text: &str, is_dict: bool) {
        self.error_buf.set_text(text);
        if !is_dict {
            self.transl_popup.txt_popup.set_buffer(self.error_buf.clone());
            self.main_win.txt_main.set_buffer(self.error_buf.clone());
        } else {
            self.dict_popup.txt_popup_dict.set_buffer(self.error_buf.clone());
            self.main_win.txt_dict_main.set_buffer(self.error_buf.clone());
        }
        //TODO: red highlight
    }

    pub fn clear_ui(&mut self, is_dict: bool) {
        dprintln!("clear_ui");
        self.set_status("", false, false);
        if !is_dict {
            self.transl_popup.title_frame.set_label("");
            self.translation_buf.set_text("");
        } else {
            self.dict_popup.title_frame_dict.set_label("");
            self.dict_buf.set_text("");
        }
    }
    

    pub fn update_ui(
        &mut self,
        state: UIState,
        is_new_source: bool
    ) {
        dprintln!("update_ui");
        let UIState {src_text, tr_uid, translator, src, target, translation_text, is_fav} = state;

        if is_new_source {  
            self.src_buf.set_text(format!("{}\n", &src_text).as_str()); //new line is req bc fltk widget bug
            self.transl_popup.src = src_text;
        } else if src_text != self.transl_popup.src {
            return;
        }

        if let Some(uid) = tr_uid && let Some(ref name) = translator {
            self.set_translator(name, &uid);
        }
        
        if let Some(t_text) = translation_text {
            self.translation_buf.set_text(format!("{}\n", &t_text).as_str());
            self.set_ready(None, false);
        }

        if let Some(lang_from) = src && let Some(lang_to) = target && let Some(translator_name) = translator {
            let title_text = format!("{}->{} ({})", lang_from.name(), lang_to.name(), translator_name);
            self.transl_popup.title_frame.set_label(&title_text);
        }

        if let Some(is_fav) = is_fav {
            let working_dir = std::env::current_dir().unwrap();
            if is_fav {
                if let Ok(image) = PngImage::load(working_dir.join(r"icons\fav_filled.png").to_str().unwrap_or("")) {
                    self.transl_popup.fav_button.set_image(Some(image.clone()));
                    self.main_win.fav_button_main.set_image(Some(image));
                    self.main_win.fav_button_main.set_label(t!(remove_from_fav));
                }
            } else {
                if let Ok(image) = PngImage::load(working_dir.join(r"icons\fav.png").to_str().unwrap_or("")) {
                    self.transl_popup.fav_button.set_image(Some(image.clone()));
                    self.main_win.fav_button_main.set_image(Some(image));
                    self.main_win.fav_button_main.set_label(t!(add_to_fav));
                }
            }
        }
        
        app::redraw();
        app::awake();
    }

    pub fn update_ui_dict(&mut self, state: UIStateDict, is_new_source: bool) {
        dprintln!("update_ui {is_new_source}");
        let UIStateDict {src_id, src_text_dict, dict_uid, dict_name, src, target, dict_text, is_fav} = state;

        if is_new_source {  
            self.src_dict_buf.set_text(format!("{}\n", &src_text_dict).as_str()); //new line is req bc fltk widget bug
            self.dict_popup.src_dict = src_text_dict;
            self.dict_popup.prnn_index = -1;
        } else if src_text_dict != self.dict_popup.src_dict {
            return;
        }

        let _ = src_id;
        
        if let Some(uid) = dict_uid && let Some(ref name) = dict_name {
            self.set_dict(name, &uid);
        }

        if let Some(dict_text) = dict_text {
            self.set_ready(None, true);
            let text_chuncs = dsl_parse(&dict_text);
            //teal, red, green, blue, indigo
            let mut sbuf = fltk::text::TextBuffer::default();
            let style_a = fltk::text::StyleTableEntry {
                color: fltk::enums::Color::Black,
                font: fltk::enums::Font::Helvetica,
                size: GLOBAL_SETTINGS.text_font_size,
            };
            let style_b = fltk::text::StyleTableEntry {
                color: fltk::enums::Color::Black,
                font: fltk::enums::Font::HelveticaBold,
                size: GLOBAL_SETTINGS.text_font_size,
            };
            let style_c = fltk::text::StyleTableEntry {
                color: fltk::enums::Color::Red,
                font: fltk::enums::Font::Helvetica,
                size: GLOBAL_SETTINGS.text_font_size,
            };
            let style_d = fltk::text::StyleTableEntry {
                color: fltk::enums::Color::DarkGreen,
                font: fltk::enums::Font::Helvetica,
                size: GLOBAL_SETTINGS.text_font_size,
            };
            let style_e = fltk::text::StyleTableEntry {
                color: fltk::enums::Color::DarkBlue,
                font: fltk::enums::Font::Helvetica,
                size: GLOBAL_SETTINGS.text_font_size,
            };
            let style_f = fltk::text::StyleTableEntry {
                color: fltk::enums::Color::from_hex(0x008080), //teal
                font: fltk::enums::Font::Helvetica,
                size: GLOBAL_SETTINGS.text_font_size,
            };
            let style_g = fltk::text::StyleTableEntry {
                color: fltk::enums::Color::from_hex(0x4B0082), //indigo
                font: fltk::enums::Font::Helvetica,
                size: GLOBAL_SETTINGS.text_font_size,
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
            
            self.dict_popup.txt_popup_dict.unset_highlight_data(sbuf.clone());
            self.dict_popup.txt_popup_dict.set_highlight_data(sbuf.clone(), vec![style_a, style_b, style_c, style_d, style_e, style_f, style_g]);

            self.main_win.txt_dict_main.unset_highlight_data(sbuf.clone());
            self.main_win.txt_dict_main.set_highlight_data(sbuf.clone(), vec![style_a, style_b, style_c, style_d, style_e, style_f, style_g]);
        }

        //let from = LangNames::from_str(src.as_ref()).unwrap_or(LangNames::En);
        //let to = LangNames::from_str(target.as_ref()).unwrap_or(LangNames::En);
        //let title_text = format!("{}->{} ({})", from.as_ref(), to.as_ref(), dict_name);

        if let Some(dict_name) = dict_name {
            let mut title_text = format!("\"{}\" - {}", &self.dict_popup.src_dict, dict_name);
            if src.is_some() || target.is_some() {
                title_text.push_str(" (");
                if let Some(src) = src {
                    title_text.push(' ');
                    title_text.push_str(src.as_ref());
                }
                if let Some(target) = target {
                    title_text.push_str("->");
                    title_text.push_str(target.as_ref());
                }
                title_text.push(')');
            }
            self.dict_popup.title_frame_dict.set_label(&title_text);
        }

        if let Some(is_fav) = is_fav {
            let working_dir = std::env::current_dir().unwrap();
            if is_fav {
                if let Ok(image) = PngImage::load(working_dir.join(r"icons\fav_filled.png").to_str().unwrap_or("")) {
                    self.dict_popup.fav_button_dict.set_image(Some(image));
                }
            } else {
                if let Ok(image) = PngImage::load(working_dir.join(r"icons\fav.png").to_str().unwrap_or("")) {
                    self.dict_popup.fav_button_dict.set_image(Some(image));
                }
            }
        }

        app::redraw();
        app::awake();
    }


    fn run_anim(&mut self, text: Option<String>) {
        let arr = if text.is_some() {
            [".  ", ".. ", "...", " ..", "  .", "   "]
        } else {
            ["/", "--", "\\", "|", "/", "--"]
        };
        let mut txt: String = "".to_string();
        if let Some(t) = text {
            txt = t;
        }

        let is_processing_clone = Arc::clone(&self.is_processing);
        let mut txt_buf_clone = self.waiting_buf.clone();
        std::thread::spawn({
            move || {
                dprintln!("---animation loop start---");
                let mut is_processing_n = 0;
                while is_processing_clone.load(Ordering::Relaxed) {
                    is_processing_n += 1;
                    if is_processing_n > 4 {
                        is_processing_n = 0;
                    }
                    txt_buf_clone.set_text(format!("{txt}{}", arr[is_processing_n]).as_str());
                    app::awake();
                    thread::sleep(Duration::from_millis(100));
                }
                dprintln!("---animation loop stop---");
            }
        });
    }

    pub fn set_tts_browser_data(&mut self, data: Vec<TTSource>) {
        self.main_win.tts_browser.clear();
        for item in data {
            self.main_win.tts_browser.add_with_data(item.voice.as_ref(), item.path);
        }
    }
    pub fn set_dict_assets_browser_data(&mut self, data: Vec<PRNNSource>) {
        self.main_win.dict_assets_browser.clear();
        for item in data {
            let name = format!("{}: {}", &item.service, &item.path);
            self.main_win.dict_assets_browser.add_with_data(&name, item.path);
        }
    }

    pub fn update_history_browser(&mut self, data: Vec<TranslSource>) {
        self.main_win.transl_browser.clear();
        for item in data {
            self.main_win.transl_browser.add_with_data(item.text.as_str(), item.id);
        }
        
    }
    pub fn update_fav_browser(&mut self, data: Vec<TranslSource>) {
        self.main_win.fav_browser.clear();
        for item in data {
            self.main_win.fav_browser.add_with_data(item.text.as_str(), item.id);
        }
    }

    pub fn set_status(&mut self, text: &str, is_error: bool, is_dict: bool) {
        //self.status_frame.set_label(text);
        //self.status_frame_dict.set_label(text);
        self.main_win.status_frame_main.set_label(text);
        if is_error {
            self.set_error(text, is_dict);
            //TODO: check src_text
        }
        app::redraw();
        app::awake();
    }

    pub fn set_src_lang(&mut self, from: Lang) {
        if let Some(item) = self.main_win.lang_choice_from.find_item(from.name()) {
            self.main_win.lang_choice_from.set_item(&item);
        }
    }
    pub fn set_target_lang(&mut self, to: Lang) {
        if let Some(item) = self.main_win.lang_choice_to.find_item(to.name()) {
            self.main_win.lang_choice_to.set_item(&item);
        }
    }
    pub fn set_translator(&mut self, name: &str, uid: &str) {
        if let Some(item) = self.main_win.transl_choice.find_item(name) {
            self.main_win.transl_choice.set_item(&item);
        }
        
        for (key, value) in &mut self.transl_popup.translator_buttons{
            if key == uid {
                value.set(true);
            } else {
                value.set(false);
            }
        }
    }
    pub fn set_dict(&mut self, name: &str, uid: &str) {
        if let Some(item) = self.main_win.dict_choice.find_item(name) {
            self.main_win.dict_choice.set_item(&item);
        }
        for (key, value) in &mut self.dict_popup.dict_buttons{
            if key == uid {
                value.set(true);
            } else {
                value.set(false);
            }
        }
    }
    pub fn set_tts_engine(&mut self, tts: &str, voice: &str) {
        let name = format!("{}-{}", tts, voice);
        if let Some(item) = self.main_win.tts_choice.find_item(&name) {
            self.main_win.tts_choice.set_item(&item);
        }
    }
    pub fn set_prnn_service(&mut self, prnn: &str) {
        if let Some(item) = self.main_win.prnn_choice.find_item(prnn) {
            self.main_win.prnn_choice.set_item(&item);
        }
    }

    pub fn show_popup(&mut self, show_dict: bool, hotspot: bool) {
        let win = if show_dict { &mut self.dict_popup.win_popup_dict } else { &mut self.transl_popup.win_popup };

        win.hide();
        win.show();

        if hotspot {
            //TODO: multi-monitor setup
            let position = app::get_mouse();
            let screen_w = app::screen_size().0 as i32;
            let screen_h = app::screen_size().1 as i32;
            let x = position.0.clamp(0, screen_w - win.w());
            let y = position.1.clamp(0, screen_h - win.h());
            win.set_pos(x, y);
        }

        
        let mut win_clone = win.clone();
        fltk::app::add_timeout3(0.1, move |_| {
            win_clone.set_on_top();
            let _ = win_clone.take_focus();
        });
        //app::redraw();
        //app::awake();
    }
    
}
