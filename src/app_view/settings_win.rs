use fltk::{
    browser::HoldBrowser,
    button::Button,
    group::{Flex, Tabs},
    prelude::*,
    window::DoubleWindow,
};
use std::cell::RefCell;
use std::rc::Rc;
use crate::{settings::{DictOption, Named, PRNNSourceOption, TTServiceOption, TranslatorOption}, utils::helpers::ui_scale};
//use debug_print::{debug_println as dprintln};
use super::t;
use super::t_s1;

use crate::types::{
    AppEvent
};

use super::GLOBAL_SETTINGS;

mod tr_tab;
mod dict_tab;
mod tts_tab;
mod prnn_tab;
mod ocr_tab;

mod general_tab;

use crate::utils::widgets::TaskbarExt;



pub struct SettingsWinView {
    pub window: DoubleWindow,
}

impl SettingsWinView {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>) -> Self {

        let mut window = DoubleWindow::default().with_size(1024, 768).with_label("Settings");
        let mut main_flex = Flex::default_fill().column();

        let tabs_wrapper = Flex::default().row().with_pos(ui_scale(5), 0);
        //tabs_wrapper.set_margin(ui_scale(5));
        let mut tabs = Tabs::default().with_size(ui_scale(100), ui_scale(50));

        let mut general_tab = general_tab::GeneralTab::new();
        
        let mut tr_tab = tr_tab::TrTab::new();
        let mut dict_tab = dict_tab::DictTab::new();
        let mut tts_tab = tts_tab::TTSTab::new();
        let mut prnn_tab = prnn_tab::PRNNTab::new();

        let mut ocr_tab = ocr_tab::OCRTab::new();//r


        tabs.end();
        tabs.auto_layout();
        tabs_wrapper.end();

        let mut win_btns = Flex::default().row();
        let mut btn_backup = Button::default().with_label("Make backup file");
        win_btns.fixed(&btn_backup, 200);
        let mut btn_ok = Button::default().with_label("Save and restart");
        win_btns.fixed(&btn_ok, 150);
        let mut btn_cancel = Button::default().with_label("Cancel");
        
        win_btns.fixed(&btn_cancel, 100);

        win_btns.set_margin(ui_scale(5));
        win_btns.end();

        main_flex.fixed(&win_btns, 40);
        main_flex.end();
        window.end();
        window.make_modal(true);
        window.set_callback({
            //let app_sender = self.app_sender;
            move |_| {
                if fltk::app::event() == fltk::enums::Event::Close {
                    app_sender.send(AppEvent::CloseSettings);
                }
            }
        });


        let data_all = Rc::new(RefCell::new(GLOBAL_SETTINGS.clone()));
        
        let data_tr = Rc::new(RefCell::new(data_all.borrow().translators.clone()));
        let data_dict = Rc::new(RefCell::new(data_all.borrow().dictionaries.clone()));
        let data_tts = Rc::new(RefCell::new(data_all.borrow().tts_services.clone()));
        let data_prnn = Rc::new(RefCell::new(data_all.borrow().prnn_services.clone()));
        let data_ocr = Rc::new(RefCell::new(data_all.borrow().ocr_models.clone()));
        
        general_tab.init(data_all.clone());
        general_tab.misc_add_services.set_callback({
            let data_tr = data_tr.clone();
            let data_dict = data_dict.clone();
            let data_tts = data_tts.clone();
            let data_prnn = data_prnn.clone();
            let tr_browser = tr_tab.browser.clone();
            let dict_browser = dict_tab.browser.clone();
            let tts_browser = tts_tab.browser.clone();
            let prnn_browser = prnn_tab.browser.clone();
            //let ocr_browser = self.ocr_tab.browser.clone(); //TODO!
            move |_| {
                add_services_from_file(
                    data_tr.clone(), data_dict.clone(), data_tts.clone(), data_prnn.clone(), 
                    tr_browser.clone(), dict_browser.clone(), tts_browser.clone(), prnn_browser.clone()
                )
            }
        });

        tr_tab.init(data_tr.clone());
        dict_tab.init(data_dict.clone());
        tts_tab.init(data_tts.clone());
        prnn_tab.init(data_prnn.clone());
        ocr_tab.init(data_ocr.clone());
        
        { //TRANSLATORS TAB

        }
        
        btn_cancel.set_callback({
            //let data_all = data_all.clone();
            move |_| {
                app_sender.send(AppEvent::CloseSettings);
                //println!("{}", Rc::strong_count(&data_all));
            }
        });
        btn_backup.set_callback({
            move |_| {
                app_sender.send(AppEvent::BackupSettings);
            }
        });
        

        btn_ok.set_callback({
            let data_all = data_all.clone();
            let data_tr = data_tr.clone();
            let data_dict = data_dict.clone();
            let data_tts = data_tts.clone();
            let data_prnn = data_prnn.clone();
            let data_ocr = data_ocr.clone();
            move |_| {                
                let mut data_all = data_all.borrow_mut().clone();
                data_all.translators = data_tr.borrow().clone();
                data_all.dictionaries = data_dict.borrow().clone();
                data_all.tts_services = data_tts.borrow().clone();
                data_all.prnn_services = data_prnn.borrow().clone();
                data_all.ocr_models = data_ocr.borrow().clone();
                //dbg!(&data_all);
                app_sender.send(AppEvent::WriteSettingsAndRestart(Box::new(data_all)));
                //self.save_and_restart(data_all.clone());
            }
        });
        
        window.show_with_taskbar();

        SettingsWinView{
            window,
        }
    
    }
}



fn btn_up_clbck<T> (data: Rc<RefCell<Vec<T>>>, mut browser: HoldBrowser) {
    let b_idx = browser.value();
    if b_idx > 1 {
        let idx = (b_idx - 1) as usize;
        data.borrow_mut().swap(idx, idx - 1);
        
        let text = browser.text(b_idx).unwrap();
        browser.remove(b_idx);
        browser.insert(b_idx - 1, &text);
        browser.select(b_idx - 1); 
        drop(data);
        browser.do_callback();
    }
}
fn btn_down_clbck<T> (data: Rc<RefCell<Vec<T>>>, mut browser: HoldBrowser) {
    let b_idx = browser.value();
    let len = data.borrow().len() as i32;
    if b_idx > 0 && b_idx < len {
        let idx = (b_idx - 1) as usize;
        data.borrow_mut().swap(idx, idx + 1);
        
        let text = browser.text(b_idx).unwrap();
        browser.remove(b_idx);
        browser.insert(b_idx + 1, &text);
        browser.select(b_idx + 1);
        drop(data);
        browser.do_callback();
    }
}


fn vec_to_csv(vec: &[String]) -> String {
    vec.iter()
    .map(|s| s.replace(',', ""))
    .collect::<Vec<String>>()
    .join(", ")
}

fn csv_to_vec(s: &str) -> Vec<String> {
    if s.is_empty() {
        return Vec::new();
    }
    s.split(',').map(|slice| slice.trim().to_string()).collect()
}


fn bind_input_clbck<T, B, F>(
    input: &mut fltk::input::Input,
    data: Rc<RefCell<Vec<T>>>,
    mut browser: B,
    setter: F,
    is_name: bool
) where
    T: 'static,
    B: BrowserExt + 'static,
    F: Fn(&mut T, &str) + 'static,
{
    input.set_callback(move |i| {
        let idx = browser.value();
        let value = i.value();
        if idx > 0 {
            let mut items = data.borrow_mut();
            if let Some(item) = items.get_mut((idx - 1) as usize) {
                setter(item, &value);
                if is_name {
                    browser.set_text(idx, &value);
                }
            }
        }
    });
}


fn bind_checkbox_clbck<T, F>(
    input: &mut fltk::button::CheckButton,
    data: Rc<RefCell<Vec<T>>>,
    idx: i32,
    setter: F
) where
    T: 'static,
    F: Fn(&mut T, bool) + 'static,
{
    input.set_callback(move |i| {
        let value = i.value();
        if idx > 0 {
            let mut items = data.borrow_mut();
            if let Some(item) = items.get_mut((idx - 1) as usize) {
                setter(item, value);
            }
        }
    });
}
fn add_service<T: Named> (service: T, data: Rc<RefCell<Vec<T>>>, mut browser: HoldBrowser) {
    let mut data = data.borrow_mut();
    browser.add(service.name());
    data.push(service);
}

#[allow(clippy::too_many_arguments)]
fn add_services_from_file (
    data_tr: Rc<RefCell<Vec<TranslatorOption>>>, data_dict: Rc<RefCell<Vec<DictOption>>>, 
    data_tts: Rc<RefCell<Vec<TTServiceOption>>>, data_prnn: Rc<RefCell<Vec<PRNNSourceOption>>>, 
    mut tr_browser: HoldBrowser, mut dict_browser: HoldBrowser,
    mut tts_browser: HoldBrowser, mut prnn_browser: HoldBrowser
) {
    
    let mut dialog = fltk::dialog::FileDialog::new(fltk::dialog::FileDialogType::BrowseFile);
    dialog.set_title("Services");
    dialog.set_filter("json5\t*.json");
    dialog.show();

    let filename = dialog.filename();
    //dbg!(filename);
    if !filename.is_empty() {
        let mut data_tr = data_tr.borrow_mut();
        let mut data_dict = data_dict.borrow_mut();
        let mut data_tts = data_tts.borrow_mut();
        let mut data_prnn = data_prnn.borrow_mut();

        let seen_uids_tr = seen_uids(&data_tr);
        let seen_uids_dict = seen_uids(&data_dict);
        let seen_uids_tts = seen_uids(&data_tts);
        let seen_uids_prnn = seen_uids(&data_prnn);

        //let mut seen_uids_dict: std::collections::HashSet<String> = std::collections::HashSet::new();
        let settings = crate::settings::app_settings::from_file(filename.to_str().unwrap_or(""));

        let mut added = 0;
        let tr = settings.translators.clone();
        let dict = settings.dictionaries.clone();
        let tts = settings.tts_services.clone();
        let prnn = settings.prnn_services.clone();
        
        for item in tr.iter() {
            if !seen_uids_tr.contains(&item.uid) {
                tr_browser.add(item.name());
                data_tr.push(item.clone());
                added += 1;
            }
        }
        for item in dict.iter() {
            if !seen_uids_dict.contains(&item.uid) {
                dict_browser.add(item.name());
                data_dict.push(item.clone());
                added += 1;
            }
        }
        for item in tts.iter() {
            if !seen_uids_tts.contains(&item.uid) {
                tts_browser.add(item.name());
                data_tts.push(item.clone());
                added += 1;
            }
        }
        for item in prnn.iter() {
            if !seen_uids_prnn.contains(&item.uid) {
                prnn_browser.add(item.name());
                data_prnn.push(item.clone());
                added += 1;
            }
        }

        crate::utils::helpers::app_message(&format!("New services added: {}", added));
    }    
}
fn seen_uids<T: Named>(data: &std::cell::RefMut<'_, Vec<T>>) -> std::collections::HashSet<String>{
    let mut seen_uids_arr: std::collections::HashSet<String> = std::collections::HashSet::new();
    for item in data.iter() {
        seen_uids_arr.insert(item.uid().to_string());
    }
    seen_uids_arr
}
fn delete_service<T: Named> (data: Rc<RefCell<Vec<T>>>, mut browser: HoldBrowser) {
    let pos = crate::utils::helpers::screen_center();
    let choice = fltk::dialog::choice2(
        pos.0, pos.1, 
        "Delete service?", 
        "No",
        "Yes",
        ""
    );
    match choice {
        Some(0) => {
        }
        Some(1) => {
            let idx = browser.value();
            if idx > 0 {
                let mut data = data.borrow_mut();
                data.remove((idx - 1) as usize);
                let browser_len = browser.size();
                if browser_len > 0 {
                    browser.remove(browser_len); 
                }
                for (i, item) in (1..).zip(data.iter()) {
                    browser.remove(i);
                    browser.insert(i, item.name());
                }
                if idx >= browser_len {
                    browser.select(browser_len - 1);
                } else if idx <= 1 {
                    browser.select(1);
                } else {
                    browser.select(idx - 1);
                }
                drop(data);
                browser.do_callback();
            }
        }
        _ => {
        }
    }
}
