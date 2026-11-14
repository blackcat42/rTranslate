use fltk::{
    browser::HoldBrowser,
    button::Button,
    group::{Flex},
    input::Input,
    prelude::*,
};
use std::cell::RefCell;
use std::rc::Rc;
use crate::{settings::{OCRModelOption}, utils::helpers::ui_scale};
//use debug_print::{debug_println as dprintln};
use super::t;

use super::bind_input_clbck;
use super::btn_up_clbck;
use super::btn_down_clbck;

pub struct OCRTab {
    pub browser: HoldBrowser,
    btn_up: Button,
    btn_down: Button,
    input_name: Input,
    input_det_model: Input,
    input_rec_model: Input,
    input_charset: Input,
    right_form: Flex,
}

impl OCRTab {
    pub fn new() -> Self {
        let mut tab = Flex::default_fill().with_label("OCR").row();
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

        let input_name = Input::default().with_label("Name:");
        right_form.fixed(&input_name, 20);
        let input_det_model = Input::default().with_label("det_model:");
        right_form.fixed(&input_det_model, 20);
        let input_rec_model = Input::default().with_label("rec_model:");
        right_form.fixed(&input_rec_model, 20);
        let input_charset = Input::default().with_label("charset:");
        right_form.fixed(&input_charset, 20);

        let note = fltk::frame::Frame::default().with_align(fltk::enums::Align::Inside | fltk::enums::Align::LeftTop).with_label(t!(s_ocr_note));
        right_form.fixed(&note, 150);

        right_form.end();
        right_form.deactivate();

        tab.end();
        OCRTab {
            browser,
            btn_up,
            btn_down,
            input_name,
            input_det_model,
            input_rec_model,
            input_charset,

            right_form,
        }
    }

    pub fn init(&mut self, data_ocr: Rc<RefCell<Vec<OCRModelOption>>>) {
        self.browser.clear();
        for m in data_ocr.borrow().iter() {
            self.browser.add(&m.name);
        }
        


        self.browser.set_callback({
            let data = data_ocr.clone();
            let mut right_form = self.right_form.clone();

            let mut input_name = self.input_name.clone();
            let mut input_det_model = self.input_det_model.clone();
            let mut input_rec_model = self.input_rec_model.clone();
            let mut input_charset = self.input_charset.clone();

            //dbg!(&data);
            move |b| {
                let idx = b.value(); //1-indexed
                if idx > 0 {
                    right_form.activate();
                    let items = data.borrow();
                    let item = &items[(idx - 1) as usize];

                    input_name.set_value(&item.name);
                    input_det_model.set_value(&item.det_model);
                    input_rec_model.set_value(&item.rec_model);
                    input_charset.set_value(&item.charset);
                    
                } else {
                    right_form.deactivate();
                }
            }
        });

        bind_input_clbck(
            &mut self.input_name, 
            data_ocr.clone(), 
            self.browser.clone(), 
            |item, v| item.name = v.to_string(), 
            true
        );
        bind_input_clbck(
            &mut self.input_det_model, 
            data_ocr.clone(), 
            self.browser.clone(), 
            |item, v| item.det_model = v.to_string(), 
            true
        );
        bind_input_clbck(
            &mut self.input_rec_model, 
            data_ocr.clone(), 
            self.browser.clone(), 
            |item, v| item.rec_model = v.to_string(), 
            true
        );
        bind_input_clbck(
            &mut self.input_charset, 
            data_ocr.clone(), 
            self.browser.clone(), 
            |item, v| item.charset = v.to_string(), 
            true
        );
        

        self.btn_up.set_callback({
            let data = data_ocr.clone();
            let browser = self.browser.clone();
            move |_| {
                btn_up_clbck(data.clone(), browser.clone())
            }
        });
        self.btn_down.set_callback({
            let data = data_ocr.clone();
            let browser = self.browser.clone();
            move |_| {
                btn_down_clbck(data.clone(), browser.clone())
            }
        });
        
        self.input_name.set_callback({
            let data = data_ocr.clone();
            let mut browser = self.browser.clone();
            move |i| {
                let idx = browser.value();
                let name = &i.value();
                println!("{}", idx);
                if idx > 0 {
                    let mut items = data.borrow_mut();
                    let arr_idx = (idx - 1) as usize;
                    items[arr_idx].name = name.to_string();
                    browser.set_text(idx, name);
                }
            }
        });
            
    }
}
