use fltk::{
    browser::HoldBrowser,
    button::Button,
    group::{Flex},
    input::Input,
    prelude::*,
};
use std::cell::RefCell;
use std::rc::Rc;
use crate::{settings::{TranslatorOption}, utils::helpers::ui_scale};
//use debug_print::{debug_println as dprintln};
use super::t;

use crate::settings::ServiceType;
use strum::IntoEnumIterator;

use super::bind_checkbox_clbck;
use super::bind_input_clbck;
use super::vec_to_csv;
use super::csv_to_vec;
use super::btn_up_clbck;
use super::btn_down_clbck;
use super::delete_service;
use super::add_service;

pub struct TrTab {
    pub browser: HoldBrowser,
    btn_up: Button,
    btn_down: Button,
    btn_add: Button,
    btn_del: Button,

    input_enabled: fltk::button::CheckButton,
    input_name: Input,
    input_service_type: fltk::menu::Choice,
    input_uid: Input,
    input_openai_url: Input,
    input_api_key_requied: fltk::button::CheckButton,
    input_openai_api_key: Input,
    input_api_key_url: Input,
    input_openai_model: Input,
    input_openai_prompt: Input,
    input_stream: fltk::button::CheckButton,
    input_cookies: fltk::button::CheckButton,
    input_force_split_view: fltk::button::CheckButton,
    input_emulation: Input,
    input_use_proxy: fltk::button::CheckButton,

    input_command: Input,
    input_args: Input,
    input_reload_if_lang_changed: fltk::button::CheckButton,
    input_markdown: fltk::button::CheckButton,
    input_supports_language_detection: fltk::button::CheckButton,

    right_form: Flex,
}

impl TrTab {
    pub fn new() -> Self {
        let mut translators_tab = Flex::default_fill().with_label(&format!("{}\t", t!(translation))).row();
            let mut left_group = Flex::default().column();

            let browser = HoldBrowser::default();
            
            let mut btn_flex = Flex::default().row();
            let btn_up = Button::default().with_label("↑");
            btn_flex.fixed(&btn_up, 20);
            let btn_down = Button::default().with_label("↓");
            btn_flex.fixed(&btn_down, 20);
            let btn_add = Button::default().with_label("+");
            btn_flex.fixed(&btn_add, 20);
            let btn_del = Button::default().with_label("DEL");
            btn_flex.fixed(&btn_del, 20);
            btn_flex.set_margin(ui_scale(5));
            btn_flex.set_pad(ui_scale(5));
            btn_flex.end();
            left_group.fixed(&btn_flex, 30);

            left_group.end();
            translators_tab.fixed(&left_group, 150);

            let mut right_form_wrapper = Flex::default().column();
            let mut right_form = Flex::default().column();
            right_form.set_pad(10);
            right_form.set_margins(ui_scale(135),ui_scale(25),ui_scale(50),ui_scale(5));
            
            let mut tr_main_settings = Flex::default().row();
            let mut input_service_type = fltk::menu::Choice::default().with_label(t!(s_type));
            for variant in ServiceType::iter() {
                input_service_type.add_choice(&variant.to_string());
            }
            input_service_type.set_value(0);

            tr_main_settings.fixed(&input_service_type, 150);
            let spacer = fltk::frame::Frame::default().with_label("");
            tr_main_settings.fixed(&spacer, 50);
            let input_uid = Input::default().with_label("UID:");
            tr_main_settings.fixed(&input_uid, 200);
            let spacer = fltk::frame::Frame::default().with_label("");
            tr_main_settings.fixed(&spacer, 70);
            let input_name = Input::default().with_label("Name:");
            tr_main_settings.fixed(&input_name, 200);
            tr_main_settings.end();
            right_form.fixed(&tr_main_settings, 22);
            //let input_service_type = Input::default().with_label("Type:");
            
            let input_enabled = fltk::button::CheckButton::default().with_label(t!(s_enabled));
            right_form.fixed(&input_enabled, 20);
            let input_force_split_view = fltk::button::CheckButton::default().with_label("Force split view (useful to prevent accidentally spending tokens in openai-services)");
            right_form.fixed(&input_force_split_view, 20);
            let input_markdown = fltk::button::CheckButton::default().with_label("Parse response as markdown");
            right_form.fixed(&input_markdown, 20);
            let input_supports_language_detection = fltk::button::CheckButton::default().with_label("Supports language detection (if checked, rTranslate sends \"auto\" as the source language, \ninstead of trying to detect it using the built-in library (whatlang))");
            right_form.fixed(&input_supports_language_detection, 20);

            let mut tr_conn_settings = Flex::default().row();
            let input_use_proxy = fltk::button::CheckButton::default().with_label(t!(s_use_proxy));
            tr_conn_settings.fixed(&input_use_proxy, 100);
            let input_cookies = fltk::button::CheckButton::default().with_label("Cookies");
            tr_conn_settings.fixed(&input_cookies, 100);
            let spacer = fltk::frame::Frame::default().with_label("");
            tr_conn_settings.fixed(&spacer, 100);
            let input_emulation = Input::default().with_size(10, 200).with_label("Emulation:");
            tr_conn_settings.fixed(&input_emulation, 200);
            tr_conn_settings.end();
            right_form.fixed(&tr_conn_settings, 20);

            let input_openai_url = Input::default().with_label("URL:");
            right_form.fixed(&input_openai_url, 20);

            let mut tr_apikey_settings = Flex::default().row();
            let input_api_key_requied = fltk::button::CheckButton::default().with_label(t!(s_api_key_requied));
            tr_apikey_settings.fixed(&input_api_key_requied, 100);
            let spacer = fltk::frame::Frame::default().with_label("");
            tr_apikey_settings.fixed(&spacer, 110);
            let input_api_key_url = Input::default().with_label("API key URL:");
            tr_apikey_settings.fixed(&input_api_key_url, 250);
            tr_apikey_settings.end();
            right_form.fixed(&tr_apikey_settings, 20);

            
            let input_openai_api_key = Input::default().with_label("API key:");
            right_form.fixed(&input_openai_api_key, 20);
            let input_openai_model = Input::default().with_label("[OpenAI] model:");
            right_form.fixed(&input_openai_model, 20);
            let input_openai_prompt = Input::default().with_label("[OpenAI] prompt file:");
            right_form.fixed(&input_openai_prompt, 20);
            let input_stream = fltk::button::CheckButton::default().with_label("Stream output (if supported)");
            right_form.fixed(&input_stream, 20);

            let input_command = Input::default().with_label("[Sidecar] Command:");
            right_form.fixed(&input_command, 20);
            let input_args = Input::default().with_label("[Sidecar] args:");
            right_form.fixed(&input_args, 20);
            let input_reload_if_lang_changed = fltk::button::CheckButton::default().with_label("[Sidecar] Reload if language is changed");
            right_form.fixed(&input_reload_if_lang_changed, 20);
            // let caution = fltk::frame::Frame::default().with_align(fltk::enums::Align::Inside | fltk::enums::Align::LeftTop).with_label("Caution: Any executable you link in the \"command\" field runs directly on your machine. \nBe careful and only use CLI apps from sources you personally trust.");
            // right_form.fixed(&caution, 25);
            

            
            right_form.end();
            right_form.deactivate(); //z

            let caution1 = fltk::frame::Frame::default().with_align(fltk::enums::Align::Inside | fltk::enums::Align::Left).with_label(t!(s_c1));
            right_form_wrapper.fixed(&caution1, 100);
            let caution2 = fltk::frame::Frame::default().with_align(fltk::enums::Align::Inside | fltk::enums::Align::Left).with_label(t!(s_c2));
            right_form_wrapper.fixed(&caution2, 35);
            let caution3 = fltk::frame::Frame::default().with_align(fltk::enums::Align::Inside | fltk::enums::Align::Left).with_label(t!(s_c3));
            right_form_wrapper.fixed(&caution3, 45);
            

            right_form_wrapper.end();

            translators_tab.end();
            TrTab {
                browser,
                btn_up,
                btn_down,
                btn_add,
                btn_del,
                input_enabled,
                input_name,
                input_uid,
                input_service_type,
                input_openai_url,
                input_api_key_requied,
                input_api_key_url,
                input_openai_api_key,
                input_openai_model,
                input_openai_prompt,
                input_stream,
                input_cookies,
                input_force_split_view,
                input_emulation,
                input_use_proxy,
                right_form,

                input_command,
                input_args,
                input_reload_if_lang_changed,
                input_markdown,
                input_supports_language_detection,
            }
    }

    pub fn init(&mut self, data_tr: Rc<RefCell<Vec<TranslatorOption>>>) {
        self.browser.clear();
        for tr in data_tr.borrow().iter() {
            self.browser.add(&tr.name);
        }
        self.browser.set_callback({
                let data = data_tr.clone();
                let mut input_service_type = self.input_service_type.clone();
                let mut input_uid = self.input_uid.clone();
                let mut input_name = self.input_name.clone();
                let mut input_enabled = self.input_enabled.clone();
                let mut input_openai_url = self.input_openai_url.clone();
                let mut input_api_key_requied = self.input_api_key_requied.clone();
                let mut input_api_key_url = self.input_api_key_url.clone();
                let mut input_openai_api_key = self.input_openai_api_key.clone();
                let mut input_openai_model = self.input_openai_model.clone();
                let mut input_openai_prompt = self.input_openai_prompt.clone();
                let mut input_stream = self.input_stream.clone();
                let mut input_cookies = self.input_cookies.clone();
                let mut input_force_split_view = self.input_force_split_view.clone();
                let mut input_emulation = self.input_emulation.clone();
                let mut input_use_proxy = self.input_use_proxy.clone();

                let mut input_command = self.input_command.clone();
                let mut input_args = self.input_args.clone();
                let mut input_reload_if_lang_changed = self.input_reload_if_lang_changed.clone();
                let mut input_markdown = self.input_markdown.clone();
                let mut input_supports_language_detection = self.input_supports_language_detection.clone();

                let mut right_form = self.right_form.clone();
                //dbg!(&data);
                move |b| {
                    let idx = b.value(); //1-indexed
                    if idx > 0 {
                        right_form.activate();
                        let items = data.borrow();
                        let item = &items[(idx - 1) as usize];

                        let idx = input_service_type.find_index(&item.service_type.to_string());
                        if idx != -1 {
                            input_service_type.set_value(idx);
                        }

                        input_uid.set_value(&item.uid);
                        input_name.set_value(&item.name);
                        input_enabled.set_value(item.enabled);
                        input_stream.set_value(item.stream);
                        input_force_split_view.set_value(item.force_split_view);
                        input_use_proxy.set_value(item.use_proxy);
                        input_cookies.set_value(item.cookies);
                        input_emulation.set_value(&item.emulation);

                        input_openai_url.set_value(&item.openai_url);
                        input_api_key_requied.set_value(item.api_key_requied);
                        input_api_key_url.set_value(&item.api_key_url);
                        input_openai_api_key.set_value(&item.openai_api_key);
                        input_openai_model.set_value(&item.openai_model);
                        input_openai_prompt.set_value(&item.openai_prompt);

                        input_command.set_value(&item.command);
                        input_args.set_value(&vec_to_csv(&item.args));
                        input_reload_if_lang_changed.set_value(item.reload_if_lang_changed);
                        input_markdown.set_value(item.markdown);
                        input_supports_language_detection.set_value(item.supports_language_detection);

                    } else {
                        right_form.deactivate();
                    }
                }
            });

            self.input_service_type.set_callback({
                let data = data_tr.clone();
                let browser = self.browser.clone();             
                move |i| {
                    let idx = browser.value();
                    let service_type = i.choice().unwrap_or("native".to_string());
                    if idx > 0 {
                        let mut items = data.borrow_mut();
                        let arr_idx = (idx - 1) as usize;
                        items[arr_idx].service_type = ServiceType::from_str(&service_type);
                    }
                }
            });

            bind_checkbox_clbck(
                &mut self.input_use_proxy, 
                data_tr.clone(), 
                self.browser.value(), 
                |item, v| item.use_proxy = v
            );
            bind_checkbox_clbck(
                &mut self.input_cookies, 
                data_tr.clone(), 
                self.browser.value(), 
                |item, v| item.cookies = v
            );
            bind_checkbox_clbck(
                &mut self.input_markdown, 
                data_tr.clone(), 
                self.browser.value(), 
                |item, v| item.markdown = v
            );
            bind_checkbox_clbck(
                &mut self.input_stream, 
                data_tr.clone(), 
                self.browser.value(), 
                |item, v| item.stream = v
            );
            bind_checkbox_clbck(
                &mut self.input_force_split_view, 
                data_tr.clone(), 
                self.browser.value(), 
                |item, v| item.force_split_view = v
            );
            bind_checkbox_clbck(
                &mut self.input_supports_language_detection, 
                data_tr.clone(), 
                self.browser.value(), 
                |item, v| item.supports_language_detection = v
            );
            bind_checkbox_clbck(
                &mut self.input_reload_if_lang_changed, 
                data_tr.clone(), 
                self.browser.value(), 
                |item, v| item.reload_if_lang_changed = v
            );
            bind_checkbox_clbck(
                &mut self.input_api_key_requied, 
                data_tr.clone(), 
                self.browser.value(), 
                |item, v| item.api_key_requied = v
            );
                        
            bind_input_clbck(
                &mut self.input_uid, 
                data_tr.clone(), 
                self.browser.clone(), 
                |item, v| item.uid = v.to_string(), 
                false
            );
            bind_input_clbck(
                &mut self.input_name, 
                data_tr.clone(), 
                self.browser.clone(), 
                |item, v| item.name = v.to_string(), 
                true
            );
            bind_checkbox_clbck(
                &mut self.input_enabled, 
                data_tr.clone(), 
                self.browser.value(), 
                |item, v| item.enabled = v
            );
            bind_input_clbck(
                &mut self.input_emulation,
                data_tr.clone(), 
                self.browser.clone(), 
                |item, v| item.emulation = v.to_string(), false
            );

            bind_input_clbck(
                &mut self.input_openai_url, 
                data_tr.clone(), 
                self.browser.clone(), 
                |item, v| item.openai_url = v.to_string(), 
                false
            );
            bind_input_clbck(
                &mut self.input_api_key_url, 
                data_tr.clone(), 
                self.browser.clone(), 
                |item, v| item.api_key_url = v.to_string(), 
                false
            );
            bind_input_clbck(
                &mut self.input_openai_api_key, 
                data_tr.clone(), 
                self.browser.clone(), 
                |item, v| item.openai_api_key = v.to_string(), 
                false
            );
            bind_input_clbck(
                &mut self.input_openai_model, 
                data_tr.clone(), 
                self.browser.clone(), 
                |item, v| item.openai_model = v.to_string(), 
                false
            );
            bind_input_clbck(
                &mut self.input_openai_prompt, 
                data_tr.clone(), 
                self.browser.clone(), 
                |item, v| item.openai_prompt = v.to_string(), 
                false
            );



            bind_input_clbck(
                &mut self.input_command, 
                data_tr.clone(), 
                self.browser.clone(), 
                |item, v| item.command = v.to_string(), 
                false
            );
            self.input_args.set_callback({
                let data = data_tr.clone();
                let browser = self.browser.clone();             
                move |i| {
                    let idx = browser.value();
                    let args = &i.value();
                    if idx > 0 {
                        let mut items = data.borrow_mut();
                        let arr_idx = (idx - 1) as usize;
                        items[arr_idx].args = csv_to_vec(args);
                    }
                }
            });

            self.btn_up.set_callback({
                let data = data_tr.clone();
                let browser = self.browser.clone();
                move |_| {
                    btn_up_clbck(data.clone(), browser.clone())
                }
            });
            self.btn_down.set_callback({
                let data = data_tr.clone();
                let browser = self.browser.clone();
                move |_| {
                    btn_down_clbck(data.clone(), browser.clone())
                }
            });
            self.btn_add.set_callback({
                let data = data_tr.clone();
                let browser = self.browser.clone();
                let srvc = crate::settings::TranslatorOption {
                    uid: "0".to_string(),
                    service_type: ServiceType::OpenAI,
                    name: "new".to_string(),
                    enabled: true,
                    use_proxy: false,
                    command: "".to_string(),
                    args: vec![],
                    reload_if_lang_changed: false,
                    emulation: "".to_string(),
                    openai_url: "".to_string(),
                    openai_api_key: "".to_string(),
                    openai_model: "".to_string(),
                    openai_prompt: "".to_string(),
                    stream: true,
                    api_key_requied: false,
                    api_key_url: "".to_string(),
                    markdown: false,
                    cookies: false,
                    force_split_view: true,
                    supports_language_detection: true,
                    
                };
                move |_| {
                    add_service(srvc.clone(), data.clone(), browser.clone())
                }
            });
            self.btn_del.set_callback({
                let data = data_tr.clone();
                let browser = self.browser.clone();
                move |_| {
                    delete_service(data.clone(), browser.clone())
                }
            });
    }
}
