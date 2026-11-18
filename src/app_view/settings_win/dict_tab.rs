use fltk::{
    browser::HoldBrowser,
    button::Button,
    group::{Flex},
    input::Input,
    prelude::*,
};
use std::cell::RefCell;
use std::rc::Rc;
use crate::{settings::{DictOption}, utils::helpers::ui_scale};
use super::t;
use crate::utils::helpers::{RE_UID, app_message};

use crate::settings::ServiceType;
use strum::IntoEnumIterator;

use super::bind_checkbox_clbck;
use super::bind_input_clbck;
use super::btn_up_clbck;
use super::btn_down_clbck;
use super::delete_service;
use super::add_service;
pub struct DictTab {
    pub browser: HoldBrowser,
    btn_up: Button,
    btn_down: Button,
    btn_add: Button,
    btn_del: Button,
    
    input_enabled: fltk::button::CheckButton,
    input_name: Input,
    input_service_type: fltk::menu::Choice,
    input_uid: Input,
    input_use_proxy: fltk::button::CheckButton,
    input_dict_path: Input,
    input_cookies: fltk::button::CheckButton,
    input_emulation: Input,

    right_form: Flex,
}

impl DictTab {
    pub fn new() -> Self {
        let mut tab = Flex::default_fill().with_label(&format!("{}\t", t!(dictionary))).row();
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
        let input_dict_path = Input::default().with_label("Path (dsl file):");
        right_form.fixed(&input_dict_path, 20);

        let input_use_proxy = fltk::button::CheckButton::default().with_label(t!(s_use_proxy));
        right_form.fixed(&input_use_proxy, 20);
        let input_emulation = Input::default().with_label("Emulation:");
        right_form.fixed(&input_emulation, 20);
        let input_cookies = fltk::button::CheckButton::default().with_label("Cookies");
        right_form.fixed(&input_cookies, 20);

        right_form.end();
        right_form.deactivate();

        tab.end();
        DictTab {
            browser,
            btn_up,
            btn_down,
            btn_add,
            btn_del,
            input_enabled,
            input_name,
            input_uid,
            input_service_type,
            input_cookies,
            input_emulation,
            input_use_proxy,
            input_dict_path,

            right_form,
        }
    }

    pub fn init(&mut self, data_dict: Rc<RefCell<Vec<DictOption>>>) {
        self.browser.clear();
        for d in data_dict.borrow().iter() {
            self.browser.add(&d.name);
        }
        


        self.browser.set_callback({
                let data = data_dict.clone();
                let mut input_service_type = self.input_service_type.clone();
                let mut input_uid = self.input_uid.clone();
                let mut input_name = self.input_name.clone();
                let mut input_enabled = self.input_enabled.clone();
                let mut input_cookies = self.input_cookies.clone();
                let mut input_emulation = self.input_emulation.clone();
                let mut input_use_proxy = self.input_use_proxy.clone();
                let mut input_dict_path = self.input_dict_path.clone();
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
                        input_use_proxy.set_value(item.use_proxy);
                        input_cookies.set_value(item.cookies);
                        input_emulation.set_value(&item.emulation);
                        input_dict_path.set_value(&item.dict_path);
                    } else {
                        right_form.deactivate();
                    }
                }
            });

            self.input_service_type.set_callback({
                let data = data_dict.clone();
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
                data_dict.clone(), 
                self.browser.value(), 
                |item, v| item.use_proxy = v
            );
            bind_checkbox_clbck(
                &mut self.input_cookies, 
                data_dict.clone(), 
                self.browser.value(), 
                |item, v| item.cookies = v
            );

            bind_input_clbck(
                &mut self.input_uid, 
                data_dict.clone(), 
                self.browser.clone(), 
                |item, v| {
                    if !RE_UID.is_match(v) {
                        app_message(&format!("invalid uid({})", v));
                    } else {
                        item.uid = v.to_string();
                    }
                }, 
                false
            );
            bind_input_clbck(
                &mut self.input_name,
                data_dict.clone(), 
                self.browser.clone(), 
                |item, v| item.name = v.to_string(), true
            );
            bind_checkbox_clbck(
                &mut self.input_enabled,
                data_dict.clone(), 
                self.browser.value(), 
                |item, v| item.enabled = v
            );
            bind_input_clbck(
                &mut self.input_dict_path,
                data_dict.clone(), 
                self.browser.clone(), 
                |item, v| item.dict_path = v.to_string(), false
            );
            bind_input_clbck(
                &mut self.input_emulation,
                data_dict.clone(), 
                self.browser.clone(), 
                |item, v| item.emulation = v.to_string(), false
            );


            self.btn_up.set_callback({
                let data = data_dict.clone();
                let browser = self.browser.clone();
                move |_| {
                    btn_up_clbck(data.clone(), browser.clone())
                }
            });
            self.btn_down.set_callback({
                let data = data_dict.clone();
                let browser = self.browser.clone();
                move |_| {
                    btn_down_clbck(data.clone(), browser.clone())
                }
            });

            self.btn_add.set_callback({
                let data = data_dict.clone();
                let browser = self.browser.clone();
                let srvc = crate::settings::DictOption {
                    uid: "0".to_string(),
                    service_type: ServiceType::DSLDict,
                    name: "new".to_string(),
                    enabled: true,
                    use_proxy: false,
                    command: "".to_string(),
                    emulation: "".to_string(),
                    cookies: false,
                    dict_path: "".to_string(),
                    path: "".to_string(),                    
                };
                move |_| {
                    add_service(srvc.clone(), data.clone(), browser.clone())
                }
            });
            self.btn_del.set_callback({
                let data = data_dict.clone();
                let browser = self.browser.clone();
                move |_| {
                    delete_service(data.clone(), browser.clone())
                }
            });
            
    }
}
