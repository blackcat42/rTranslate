use fltk::{
    browser::HoldBrowser,
    button::Button,
    group::{Flex},
    input::Input,
    prelude::*,
};
use std::cell::RefCell;
use std::rc::Rc;
use crate::{settings::{TTServiceOption}, utils::helpers::ui_scale};
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


pub struct TTSTab {
    pub browser: HoldBrowser,
    btn_up: Button,
    btn_down: Button,
    
    input_enabled: fltk::button::CheckButton,
    input_name: Input,
    input_service_type: fltk::menu::Choice,
    input_uid: Input,
    input_command: Input,
    input_args: Input,
    input_voices: Input,
    input_speed: Input,

    input_openai_url: Input,
    input_api_key_requied: fltk::button::CheckButton,
    input_openai_api_key: Input,
    input_api_key_url: Input,
    input_openai_model: Input,
    input_openai_response_format: Input,

    input_emulation: Input,
    input_use_proxy: fltk::button::CheckButton,
    
    right_form: Flex,
}

impl TTSTab {
    pub fn new() -> Self {
        let mut tab = Flex::default_fill().with_label("TTS").row();
        let mut left_group = Flex::default().column();

        let browser = HoldBrowser::default();
        
        let mut btn_flex = Flex::default().row();
        let btn_up = Button::default().with_label("↑");
        btn_flex.fixed(&btn_up, 20);
        let btn_down = Button::default().with_label("↓");
        btn_flex.fixed(&btn_down, 20);
        btn_flex.set_margin(ui_scale(5));
        btn_flex.set_pad(ui_scale(5));
        btn_flex.end();
        left_group.fixed(&btn_flex, 30);

        left_group.end();
        tab.fixed(&left_group, 200);

        let mut right_form = Flex::default().column();
        right_form.set_pad(10);
        right_form.set_margins(ui_scale(150),ui_scale(25),ui_scale(50),ui_scale(10));

        let mut input_service_type = fltk::menu::Choice::default().with_label(t!(s_type));
        for variant in ServiceType::iter() {
            input_service_type.add_choice(&variant.to_string());
        }
        input_service_type.set_value(0);
        right_form.fixed(&input_service_type, 20);
        let input_uid = Input::default().with_label("UID:");
        right_form.fixed(&input_uid, 20);
        let input_name = Input::default().with_label("Name:");
        right_form.fixed(&input_name, 20);
        let input_enabled = fltk::button::CheckButton::default().with_label(t!(s_enabled));
        right_form.fixed(&input_enabled, 20);

        let input_use_proxy = fltk::button::CheckButton::default().with_label(t!(s_use_proxy));
        right_form.fixed(&input_use_proxy, 20);
        let input_emulation = Input::default().with_label("Emulation:");
        right_form.fixed(&input_emulation, 20);

        let input_openai_url = Input::default().with_label("URL:");
        right_form.fixed(&input_openai_url, 20);
        let input_api_key_requied = fltk::button::CheckButton::default().with_label(t!(s_api_key_requied));
        right_form.fixed(&input_api_key_requied, 20);
        let input_api_key_url = Input::default().with_label("API key URL:");
        right_form.fixed(&input_api_key_url, 20);
        let input_openai_api_key = Input::default().with_label("API key:");
        right_form.fixed(&input_openai_api_key, 20);
        let input_openai_model = Input::default().with_label("[OpenAI] model:");
        right_form.fixed(&input_openai_model, 20);
        let input_openai_response_format = Input::default().with_label("[OpenAI] response format:");
        right_form.fixed(&input_openai_response_format, 20);
        let input_speed = Input::default().with_label("[OpenAI] speed:");
        right_form.fixed(&input_speed, 20);

        let input_command = Input::default().with_label("[Sidecar] Command:");
        right_form.fixed(&input_command, 20);
        let input_args = Input::default().with_label("[Sidecar] Process args:");
        right_form.fixed(&input_args, 20);
        let input_voices = Input::default().with_label("Voices:");
        right_form.fixed(&input_voices, 20);

        right_form.end();
        right_form.deactivate();

        tab.end();
        TTSTab {
            browser,
            btn_up,
            btn_down,
            input_enabled,
            input_name,
            input_uid,
            input_service_type,
            input_command,
            input_args,
            input_voices,

            input_emulation,
            input_use_proxy,

            input_openai_url,
            input_api_key_requied,
            input_api_key_url,
            input_openai_api_key,
            input_openai_model,
            input_openai_response_format,
            input_speed,
            
            right_form,
        }
    }

    pub fn init(&mut self, data_tts: Rc<RefCell<Vec<TTServiceOption>>>) {
        self.browser.clear();
        for tts in data_tts.borrow().iter() {
            self.browser.add(&tts.name);
        }
        


        self.browser.set_callback({
            let data = data_tts.clone();
            let mut input_service_type = self.input_service_type.clone();
            let mut input_uid = self.input_uid.clone();
            let mut input_name = self.input_name.clone();
            let mut input_enabled = self.input_enabled.clone();
            let mut input_openai_url = self.input_openai_url.clone();
            let mut input_api_key_requied = self.input_api_key_requied.clone();
            let mut input_api_key_url = self.input_api_key_url.clone();
            let mut input_openai_api_key = self.input_openai_api_key.clone();
            let mut input_openai_model = self.input_openai_model.clone();
            let mut input_openai_response_format = self.input_openai_response_format.clone();
            let mut input_speed = self.input_speed.clone();
            let mut input_emulation = self.input_emulation.clone();
            let mut input_use_proxy = self.input_use_proxy.clone();

            let mut input_command = self.input_command.clone();
            let mut input_args = self.input_args.clone();
            let mut input_voices = self.input_voices.clone();
            
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
                    //input_stream.set_value(item.stream);
                    input_use_proxy.set_value(item.use_proxy);
                    input_emulation.set_value(&item.emulation);

                    input_openai_url.set_value(&item.openai_url);
                    input_api_key_requied.set_value(item.api_key_requied);
                    input_api_key_url.set_value(&item.api_key_url);
                    input_openai_api_key.set_value(&item.openai_api_key);
                    input_openai_model.set_value(&item.openai_model);
                    input_openai_response_format.set_value(&item.openai_response_format);

                    input_speed.set_value(&item.speed.to_string());

                    input_command.set_value(&item.command);
                    input_args.set_value(&vec_to_csv(&item.args));
                    input_voices.set_value(&vec_to_csv(&item.voices));
                } else {
                    right_form.deactivate();
                }
            }
        });

        self.input_service_type.set_callback({
            let data = data_tts.clone();
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
            data_tts.clone(), 
            self.browser.value(), 
            |item, v| item.use_proxy = v
        );
        bind_checkbox_clbck(
            &mut self.input_api_key_requied, 
            data_tts.clone(), 
            self.browser.value(), 
            |item, v| item.api_key_requied = v
        );
                    
        bind_input_clbck(
            &mut self.input_uid, 
            data_tts.clone(), 
            self.browser.clone(), 
            |item, v| item.uid = v.to_string(), 
            false
        );
        bind_input_clbck(
            &mut self.input_name, 
            data_tts.clone(), 
            self.browser.clone(), 
            |item, v| item.name = v.to_string(), 
            true
        );
        bind_checkbox_clbck(
            &mut self.input_enabled,
            data_tts.clone(), 
            self.browser.value(), 
            |item, v| item.enabled = v
        );
        bind_input_clbck(
            &mut self.input_emulation,
            data_tts.clone(), 
            self.browser.clone(), 
            |item, v| item.emulation = v.to_string(), false
        );

        bind_input_clbck(
            &mut self.input_openai_url, 
            data_tts.clone(), 
            self.browser.clone(), 
            |item, v| item.openai_url = v.to_string(), 
            false
        );
        bind_input_clbck(
            &mut self.input_api_key_url, 
            data_tts.clone(), 
            self.browser.clone(), 
            |item, v| item.api_key_url = v.to_string(), 
            false
        );
        bind_input_clbck(
            &mut self.input_openai_api_key, 
            data_tts.clone(), 
            self.browser.clone(), 
            |item, v| item.openai_api_key = v.to_string(), 
            false
        );
        bind_input_clbck(
            &mut self.input_openai_model, 
            data_tts.clone(), 
            self.browser.clone(), 
            |item, v| item.openai_model = v.to_string(), 
            false
        );
        bind_input_clbck(
            &mut self.input_openai_response_format, 
            data_tts.clone(), 
            self.browser.clone(), 
            |item, v| item.openai_response_format = v.to_string(), 
            false
        );



        bind_input_clbck(
            &mut self.input_speed, 
            data_tts.clone(), 
            self.browser.clone(), 
            |item, v| item.speed = v.parse().unwrap_or(1.0), 
            false
        );
        bind_input_clbck(
            &mut self.input_command, 
            data_tts.clone(), 
            self.browser.clone(), 
            |item, v| item.command = v.to_string(), 
            false
        );
        self.input_args.set_callback({
            let data = data_tts.clone();
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
        self.input_voices.set_callback({
            let data = data_tts.clone();
            let browser = self.browser.clone();             
            move |i| {
                let idx = browser.value();
                let voices = &i.value();
                if idx > 0 {
                    let mut items = data.borrow_mut();
                    let arr_idx = (idx - 1) as usize;
                    items[arr_idx].voices = csv_to_vec(voices);
                }
            }
        });

        self.btn_up.set_callback({
            let data = data_tts.clone();
            let browser = self.browser.clone();
            move |_| {
                btn_up_clbck(data.clone(), browser.clone())
            }
        });
        self.btn_down.set_callback({
            let data = data_tts.clone();
            let browser = self.browser.clone();
            move |_| {
                btn_down_clbck(data.clone(), browser.clone())
            }
        });
            
    }
}
