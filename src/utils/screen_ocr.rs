use debug_print::{debug_println as dprintln};
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
use image::GenericImageView;
use xcap::Monitor;
use anyhow::{anyhow, Result};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::io::{Write, Read};
use std::rc::Rc;
use std::cell::RefCell;
use std::fs::File;

use crate::types::{
    AppEvent,
    BLWCoords,
    OCRModelOption
};
use crate::utils::helpers::{
    borderless_win_handler, 
    borderless_win_frame_handler,
    is_win7_or_greater
};
use crate::utils::widgets::IconExt;
use crate::utils::helpers::ui_scale;
use super::GLOBAL_SETTINGS;
//use super::UICONFIG;

enum OCREvent {
    Success(String),
}

pub struct ScreenOCR {
    pub win: DoubleWindow,
    pub img: Option<MaskableImage>,
    win_screenshot_wrapper: fltk::group::Group,
    overlay_win: DoubleWindow,
    app_sender: fltk::app::Sender<AppEvent>,
    checkbox_ocr_append: fltk::button::CheckButton,
    choice_ocr_model: fltk::menu::Choice,
    btn_ocr: fltk::button::Button,

    kill_sender: Option<std::sync::mpsc::Sender<()>>,
    ocr_thread: Option<std::thread::JoinHandle<()>>,
    is_processing: Arc<AtomicBool>,
    crop: Option<CropSegment>,

    pub ocr_text_buf: fltk::text::TextBuffer,
    //pub ocr_waiting_buf: fltk::text::TextBuffer,
    text_widget: fltk::text::TextEditor,
}

impl ScreenOCR {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>) -> Self {
        let (screen_w, screen_h) = app::screen_size();
        let working_dir = std::env::current_dir().unwrap();

        let mut win = window::Window::default().with_pos(0, 0).with_size(screen_w as i32, screen_h as i32).with_label("Screen OCR");
         win.set_border(true);
    
        let mut win_screenshot_wrapper = fltk::group::Group::new(0, 0, screen_w as i32, screen_h as i32, "");
        let mut img: Option<MaskableImage> = None;        
        win_screenshot_wrapper.end();
        win_screenshot_wrapper.make_resizable(false);

        win.end();
        win.make_resizable(true);
        win.fullscreen(GLOBAL_SETTINGS.ocr_fullscreen);
        win.show();
        win.hide();


        let mut overlay_win = fltk::window::Window::default().with_size(ui_scale(550), ui_scale(200));

        let mut frame_wrapper = fltk::group::Flex::default().column().size_of_parent();
        frame_wrapper.set_margins(ui_scale(3),ui_scale(3),ui_scale(3),ui_scale(3));

        let mut frame = fltk::group::Flex::default().column();
        frame.set_spacing(ui_scale(3));
        frame.set_frame(fltk::enums::FrameType::EngravedBox);

        let mut flex_titlebar = fltk::group::Flex::default().row();
        flex_titlebar.set_margins(ui_scale(2),ui_scale(2),ui_scale(3),0);
        flex_titlebar.set_pad(ui_scale(5));
        let mut close_button = fltk::button::Button::new(ui_scale(5), ui_scale(5), ui_scale(18), ui_scale(18), "");
        close_button.set_png_icon("close");
        flex_titlebar.fixed(&close_button, ui_scale(18));
        frame.fixed(&flex_titlebar, ui_scale(20));
        flex_titlebar.end();

        let mut flex = fltk::group::Flex::default().column();
        flex.set_margins(0, 0, 0, 0);
        flex.set_spacing(ui_scale(7));
        let ocr_text_buf = fltk::text::TextBuffer::default();
        //let ocr_waiting_buf = text::TextBuffer::default();
        let mut txt = fltk::text::TextEditor::default();
        txt.set_frame(fltk::enums::FrameType::FlatBox);
        txt.set_buffer(ocr_text_buf.clone());
        txt.wrap_mode(fltk::text::WrapMode::AtBounds, 0);

        let mut flex2 = fltk::group::Flex::default().column();
        flex2.set_spacing(5);
        let mut flex_buttons_wrapper = group::Flex::default().column();
        flex_buttons_wrapper.set_margins(ui_scale(15), 0, ui_scale(15), 0);
        let mut flex_buttons = group::Flex::default().row();
        let mut checkbox_ocr_append = button::CheckButton::default().with_label("Keep previous text")
            .with_align(fltk::enums::Align::Inside | fltk::enums::Align::Left | fltk::enums::Align::ImageNextToText);
        flex_buttons.fixed(&checkbox_ocr_append, ui_scale(150));

        fltk::frame::Frame::default();
        let mut choice_ocr_model = fltk::menu::Choice::default().with_label("Model:").with_align(fltk::enums::Align::Left);

        for ocr_m in GLOBAL_SETTINGS.ocr_models.iter() {
            let name = (*ocr_m.name).to_string();
            choice_ocr_model.add_choice(
                &name
            );
        }
        /*if let Some(item) = choice_ocr_model.find_item("PP-OCRv6_small") {
            choice_ocr_model.set_item(&item);
        }*/
        choice_ocr_model.set_value(0);
        flex_buttons.fixed(&choice_ocr_model, ui_scale(150));

        /*let mut checkbox_ocr_fast = button::CheckButton::default().with_label("fast (less accuracy)")
            .with_align(fltk::enums::Align::Inside | fltk::enums::Align::Left | fltk::enums::Align::ImageNextToText);
        flex_buttons.fixed(&checkbox_ocr_fast, 400);*/
        flex_buttons.end();
        flex2.fixed(&flex_buttons_wrapper, ui_scale(25));
        flex_buttons_wrapper.end();
        let mut flex_buttons_wrapper2 = group::Flex::default().column();
        flex_buttons_wrapper2.set_margins(ui_scale(15), 0, ui_scale(15), 0);
        let flex_buttons2 = group::Flex::default().row();
        let mut btn_ocr = button::Button::new(0, ui_scale(15), ui_scale(100), ui_scale(40), "Run OCR")
            .with_align(fltk::enums::Align::Center | fltk::enums::Align::ImageNextToText);
        let mut btn_copy = button::Button::new(0, ui_scale(15), ui_scale(100), ui_scale(40), "Copy")
            .with_align(fltk::enums::Align::Center | fltk::enums::Align::ImageNextToText);
        let mut btn_translate = button::Button::new(0, ui_scale(15), ui_scale(100), ui_scale(40), "Translate")
            .with_align(fltk::enums::Align::Center | fltk::enums::Align::ImageNextToText);
        let mut btn_exit = button::Button::new(0, ui_scale(15), ui_scale(100), ui_scale(40), "Exit (Esc)")
            .with_align(fltk::enums::Align::Center | fltk::enums::Align::ImageNextToText);
        flex_buttons2.end();
        flex2.fixed(&flex_buttons_wrapper2, ui_scale(25));
        flex_buttons_wrapper2.end();
        flex.fixed(&flex2, ui_scale(60));
        flex2.end();

        flex.end();
        frame.end();
        frame_wrapper.end();
        overlay_win.make_resizable(true);
        overlay_win.set_border(false);
        overlay_win.set_frame(fltk::enums::FrameType::UpBox);
        overlay_win.resizable(&overlay_win);
        overlay_win.size_range(ui_scale(400), ui_scale(150), 0 ,0); //1510
        overlay_win.end();
        
        win.set_callback(move |w| {
            if app::event() == fltk::enums::Event::Close {
                app_sender.send(AppEvent::OCRDrop);
            }
        });
        win.handle(move |w, ev| match ev {
            enums::Event::KeyDown => {
                if app::event_key() == enums::Key::Escape {
                    app_sender.send(AppEvent::OCRDrop);
                    true
                } else {
                    false
                }
            }
            _ => false,
        });

        close_button.set_callback({
            let mut overlay_win = overlay_win.clone();
            let mut win = win.clone();
            move |_| {
                overlay_win.hide();
            }
        });
        btn_ocr.set_callback({
            move |_| {
                app_sender.send(AppEvent::OCRun);
            }
        });
        btn_copy.set_callback({
            let ocr_text_buf = ocr_text_buf.clone();
            move |_| {
                fltk::app::copy(&ocr_text_buf.text());
            }
        });
        btn_translate.set_callback({
            let ocr_text_buf = ocr_text_buf.clone();
            move |_| {
                app_sender.send(AppEvent::TranslateText(ocr_text_buf.text(), false));
            }
        });
        btn_exit.set_callback({
            let win = win.clone();
            move |_| {
                app_sender.send(AppEvent::OCRDrop);
            }
        });



        let is_inner = Rc::new(RefCell::new(false));
        frame.handle({
            let mut overlay_win = overlay_win.clone();
            let is_inner = Rc::clone(&is_inner);
            move |_t, event| {
                borderless_win_frame_handler(event, &mut overlay_win, &is_inner)
            }
        });
        overlay_win.handle({
            //popup borderless window resizing and dragging
            let mut coords = BLWCoords::default();
            let is_inner = Rc::clone(&is_inner);
            move |window, event| {
                match event {
                    enums::Event::KeyDown => {
                        if app::event_key() == enums::Key::Escape {
                            app_sender.send(AppEvent::OCRDrop);
                            true
                        } else {
                            false
                        }
                    }
                    _ => {
                        borderless_win_handler(window, event, &mut coords, &is_inner)
                    }
                }
            }
        });


        ScreenOCR {
            win,
            img,
            crop: None,
            app_sender,
            win_screenshot_wrapper,
            overlay_win,
            kill_sender: None,
            ocr_thread: None,
            is_processing: Arc::new(AtomicBool::new(false)),
            text_widget: txt,
            ocr_text_buf,
            //ocr_waiting_buf,
            checkbox_ocr_append,
            choice_ocr_model,
            btn_ocr
        }
    }

    pub fn set_ocr_results(&mut self, s: String) {
        self.set_ready();
        let s = extract_ocr_text(&s);
        if self.checkbox_ocr_append.is_checked() {
            self.ocr_text_buf.append(&s);
        } else {
            self.ocr_text_buf.set_text(&s);
        }
    }

    pub fn process_image(&mut self) -> Result<()> {
        let position = app::get_mouse();
        let monitor_idx = app::screen_num(position.0, position.1);
        let scale = app::screen_scale(monitor_idx);
        let s_position = (position.0 as f32 * scale, position.1 as f32 * scale);

        let monitor = Monitor::from_point(s_position.0 as i32, s_position.1 as i32)?; 
        let image = monitor.capture_image()?;

        let (screen_w, screen_h) = app::screen_size();
        let screen_w = if let Ok(w) = monitor.width() {w as f64} else {screen_w};
        let screen_h = if let Ok(h) = monitor.height() {h as f64} else {screen_h};

        if let Some(ref old_widget) = self.img {
            self.win.remove(&old_widget.frame);
            app::delete_widget(old_widget.frame.clone());
        }

        self.img = Some(MaskableImage::new(0, 0, screen_w as i32, screen_h as i32, Some(image), self.app_sender, self.win.clone(), self.overlay_win.clone()));

        if let Some(ref new_widget) = self.img {
            self.win_screenshot_wrapper.add(&new_widget.frame);
            self.win_screenshot_wrapper.redraw();
        }

        app::awake();
        self.win.hide();
        self.win.show();
        
        //self.win.set_pos(position.0, position.1);
        let position = app::get_mouse();
        let monitor_idx = app::screen_num(position.0, position.1);
        let screen_scale = app::screen_scale(monitor_idx);

        let screen_rect = app::Screen::xywh_num(monitor_idx)?;
        self.win.resize(
            (screen_rect.x as f32 * screen_scale) as i32, 
            (screen_rect.y as f32 * screen_scale) as i32, 
            (screen_rect.w as f32 * screen_scale) as i32, 
            (screen_rect.h as f32 * screen_scale) as i32
        );

        self.win.take_focus();
        Ok(())
    }

    fn terminate(&mut self) {
        if let Some(s) = self.kill_sender.take() {
            let _ = s.send(());
        }
        if let Some(handle) = self.ocr_thread.take() {
            let _ = handle.join();
        }
        self.set_ready();
    }

    pub fn run_ocr(&mut self) -> Result<()> {

        if !is_win7_or_greater() {
            return self.run_ocr_winxp();
        }

        if self.is_processing.load(Ordering::SeqCst) {
            self.terminate();
            return Ok(());
        } else {
            self.terminate();
        }
        
        if self.ocr_thread.is_some() {
            return Ok(());
        }
        let working_dir = std::env::current_dir().unwrap();

        let image_data;
        let width_bytes;
        let height_bytes;
        if let Some(i) = &self.crop {
            image_data = i.img.clone();
            width_bytes = i.width.to_le_bytes();
            height_bytes = i.height.to_le_bytes();
        } else {
            return Err(anyhow!("no image data"));
        }
        let size = image_data.len() as u32;
        
        let (kill_tx, kill_rx) = std::sync::mpsc::channel();
        self.kill_sender = Some(kill_tx);

        let mut det_model = "ocr_models/PP-OCRv6_small_det.mnn".to_string();
        let mut rec_model = "ocr_models/PP-OCRv6_small_rec.mnn".to_string();
        let mut charset = "ocr_models/ppocr_keys_v6_small.txt".to_string();
        /*if self.checkbox_ocr_fast.is_checked() {
            det_model = "ocr_models/PP-OCRv6_tiny_det.mnn".to_string();
            rec_model = "ocr_models/PP-OCRv6_tiny_rec.mnn".to_string();
            charset = "ocr_models/ppocr_keys_v6_tiny.txt".to_string();
        }*/

        if let Some(text) = self.choice_ocr_model.choice() {
            let last_match: Option<&OCRModelOption> = GLOBAL_SETTINGS.ocr_models
                .iter()
                .rev()
                .find(|model| model.name == text);
            if let Some(model) = last_match {
                det_model = model.det_model.clone();
                rec_model = model.rec_model.clone();
                charset = model.charset.clone();
            }
        } 

        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let command = ".\\rt_ocr".to_string();

        let mut child;
        if which::which(&command).is_ok() {
            child = std::process::Command::new(working_dir.join(&command))
                .arg("--pipe")
                .arg("--det_model").arg(&det_model)
                .arg("--rec_model").arg(&rec_model)
                .arg("--charset").arg(&charset)
                .creation_flags(CREATE_NO_WINDOW)
                .current_dir(working_dir)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()?;
        } else {
            self.app_sender.send(AppEvent::SetReady(Some("error".to_string()), false));
            return Ok(());
        }

        let mut stdin = child.stdin.take().ok_or(anyhow!("stdin error"))?;
        stdin.write_all(&width_bytes)?;
        stdin.write_all(&height_bytes)?;
        stdin.write_all(&image_data)?;
        stdin.flush()?;
        drop(stdin);

        let mut stdout = child.stdout.take().ok_or(anyhow!("stdout error"))?;

        let app_sender = self.app_sender;
        self.set_waiting();
        let handle = std::thread::spawn(move || {
            loop {
                if let Ok(_) = kill_rx.try_recv() {
                    dprintln!("Kill signal received");
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
                match child.try_wait() {
                    Ok(Some(status)) => {
                        dprintln!("Process finished with: {:?}", status);
                        break;
                    }
                    Ok(None) => {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(e) => {
                        eprintln!("Error checking process: {}", e);
                        break;
                    }
                }
            }

            let mut final_output = String::new();
            stdout.read_to_string(&mut final_output).unwrap();
            let _ = app_sender.send(AppEvent::OCRSuccess(final_output));
            dprintln!("ocr_thread_reader stopping");
        });
        self.ocr_thread = Some(handle);

        Ok(())
    }

    pub fn run_ocr_winxp(&mut self) -> Result<()> {
        //TODO
        if self.is_processing.load(Ordering::SeqCst) {
            self.terminate();
            return Ok(());
        } else {
            self.terminate();
        }
        
        if self.ocr_thread.is_some() {
            return Ok(());
        }
        let working_dir = std::env::current_dir().unwrap();

        let image_data;
        let width_bytes;
        let height_bytes;
        if let Some(i) = &self.crop {
            image_data = i.img.clone();
            width_bytes = i.width as u32;
            height_bytes = i.height as u32;
        } else {
            return Err(anyhow!("no image data"));
        }
        let size = image_data.len() as u32;

        image::save_buffer(
            "ocr_image.png",
            &image_data,
            width_bytes,
            height_bytes,
            image::ColorType::Rgba8,
        )?;
        
        let (kill_tx, kill_rx) = std::sync::mpsc::channel();
        self.kill_sender = Some(kill_tx);

        let mut det_model = "ocr_models/PP-OCRv6_small_det.mnn".to_string();
        let mut rec_model = "ocr_models/PP-OCRv6_small_rec.mnn".to_string();
        let mut charset = "ocr_models/ppocr_keys_v6_small.txt".to_string();

        if let Some(text) = self.choice_ocr_model.choice() {
            let last_match: Option<&OCRModelOption> = GLOBAL_SETTINGS.ocr_models
                .iter()
                .rev()
                .find(|model| model.name == text);
            if let Some(model) = last_match {
                det_model = model.det_model.clone();
                rec_model = model.rec_model.clone();
                charset = model.charset.clone();
            }
        } 

        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let command = ".\\rt_ocr".to_string();

        let mut child_output = String::new();
        let mut child_err = String::new();

        let mut child;
        if which::which(&command).is_ok() {
            let output_file = File::create("ocr_output.tmp")?;
            let output_err_file = File::create("ocr_output_err.tmp")?;

            child = std::process::Command::new(working_dir.join(&command))
                .arg("-f").arg("ocr_image.png")
                .arg("--det_model").arg(&det_model)
                .arg("--rec_model").arg(&rec_model)
                .arg("--charset").arg(&charset)
                .creation_flags(CREATE_NO_WINDOW)
                .current_dir(working_dir)
                .stdin(std::process::Stdio::null()) 
                .stdout(std::process::Stdio::from(output_file)) 
                .stderr(std::process::Stdio::from(output_err_file))
                .spawn()?;
        } else {
            self.app_sender.send(AppEvent::SetReady(Some("error".to_string()), false));
            return Ok(());
        }

        let app_sender = self.app_sender;
        self.set_waiting();
        let handle = std::thread::spawn(move || {
            loop {
                if let Ok(_) = kill_rx.try_recv() {
                    dprintln!("Kill signal received");
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
                match child.try_wait() {
                    Ok(Some(status)) => {
                        dprintln!("Process finished with: {:?}", status);
                        break;
                    }
                    Ok(None) => {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(e) => {
                        eprintln!("Error checking process: {}", e);
                        break;
                    }
                }
            }

            if let Ok(mut file) = File::open("ocr_output.tmp") {
                file.read_to_string(&mut child_output).unwrap();
            }
            if let Ok(mut file) = File::open("ocr_output_err.tmp") {
                file.read_to_string(&mut child_err).unwrap();
            }

            //let mut output = String::new();
            //stdout.read_to_string(&mut output)?;
            let _ = app_sender.send(AppEvent::OCRSuccess(child_output));
            dprintln!("ocr_thread_reader stopping");
            if let Err(e) = std::fs::remove_file("ocr_output.tmp") {
                println!("error remove file: {}", e);
            }
            if let Err(e) = std::fs::remove_file("ocr_output_err.tmp") {
                println!("error remove file: {}", e);
            }
            if let Err(e) = std::fs::remove_file("ocr_image.png") {
                println!("error remove file: {}", e);
            }

        });
        self.ocr_thread = Some(handle);

        Ok(())
    }


    pub fn update_crop(&mut self, crop: CropSegment) {
        self.crop = Some(crop);
    }

    pub fn clear(&mut self) {
        self.terminate();
        if let Some(ref old_widget) = self.img {
            self.win.remove(&old_widget.frame);
            app::delete_widget(old_widget.frame.clone());
        }
        self.img = None;
        self.crop = None;
        self.win.hide();
        self.overlay_win.hide();
        self.set_ready();
    }


    pub fn set_waiting(&mut self) {
        self.is_processing.store(true, Ordering::SeqCst);
        //self.text_widget.set_buffer(self.ocr_waiting_buf.clone());
        self.run_anim();
    }

    pub fn set_ready(&mut self) {
        self.is_processing.store(false, Ordering::SeqCst);
        //self.text_widget.set_buffer(self.ocr_text_buf.clone());
    }
    /*pub fn set_error(&mut self, text: &str, is_dict: bool) {
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
*/
    fn run_anim(&mut self) {
        let arr = [".  ", ".. ", "...", " ..", "  .", "   "];

        let is_processing_clone = Arc::clone(&self.is_processing);
        //let mut txt_buf_clone = self.ocr_waiting_buf.clone();
        let mut btn_ocr_clone = self.btn_ocr.clone();
        std::thread::spawn({
            move || {
                dprintln!("---animation loop start---");
                btn_ocr_clone.set_align(fltk::enums::Align::Inside | fltk::enums::Align::Left | fltk::enums::Align::ImageNextToText);
                let mut is_processing_n = 0;
                while is_processing_clone.load(Ordering::SeqCst) {
                    is_processing_n += 1;
                    if is_processing_n > 4 {
                        is_processing_n = 0;
                    }
                    btn_ocr_clone.set_label(format!("OCR (cancel) {}", arr[is_processing_n]).as_str());
                    //txt_buf_clone.set_text(format!("{}", arr[is_processing_n]).as_str());
                    app::awake();
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                btn_ocr_clone.set_label("OCR");
                btn_ocr_clone.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageNextToText);
                dprintln!("---animation loop stop---");
            }
        });
    }
}



#[derive(Clone)]
#[derive(Debug)]
pub struct CropSegment{
    pub x: i32, 
    pub y: i32, 
    pub width: i32, 
    pub height: i32,
    pub img: Vec<u8>,
}

impl CropSegment {
    pub fn new(x: i32, y: i32, width: i32, height: i32, cs_img: Vec<u8>) -> Self {
        let mut cs = CropSegment{
            x: x,
            y: y, 
            width: width,
            height: height,
            img: cs_img, 
        };
        cs
    }
}
pub struct MaskableImage {
    pub frame: Frame,
}

impl MaskableImage {
    pub fn new(x: i32, y: i32, w: i32, h: i32, img_path: Option<image::RgbaImage>, s: fltk::app::Sender<AppEvent>, mut main_win: DoubleWindow, mut overlay_win: DoubleWindow) -> Self {

        let mut sb = MaskableImage {
            frame: fltk::frame::Frame::new(x,y,w,h,"")
        };
        sb.frame.set_frame(fltk::enums::FrameType::BorderBox);
        // sb.frame.set_frame(FrameType::FlatBox);
        sb.frame.set_color(fltk::enums::Color::Green);


        let mut img = img_path.unwrap();
        
        println!("w is: {}, h is: {}", img.width(), img.height());
        let (x, y) = img.dimensions();

        let mut grey_img = img.clone();

        let mut dyn_img = image::DynamicImage::ImageRgba8(img.clone());

        let img = image::imageops::colorops::brighten(&dyn_img, -35);

        let rgb_image = fltk::image::RgbImage::new(&img, x as i32, y as i32, fltk::enums::ColorDepth::Rgba8).unwrap();
        let rgb_image = rgb_image.convert(fltk::enums::ColorDepth::L8).unwrap();

        //start selection
        let mut s_x = 0;
        let mut s_y = 0;

        let mut f_x = 0;
        let mut f_y = 0; 

        let mut released = false; 
        let mut first_iter = true;

        let position = app::get_mouse();
        let monitor_idx = app::screen_num(position.0, position.1);
        let screen_scale = app::screen_scale(monitor_idx);

        //let screen_scale = main_win.pixels_per_unit();
        sb.frame.handle(move |t, ev| {
            let mut grey_img_c = grey_img.clone();
            let s_clone = s;
            let mut rgc = rgb_image.clone();
            let (ex,ey) = app::event_coords();

            match ev {
                enums::Event::KeyDown => {
                    if app::event_key() == enums::Key::Escape {
                        s.send(AppEvent::OCRDrop);
                    } 
                }

                fltk::enums::Event::Enter => {
                    main_win.set_cursor(enums::Cursor::Cross);
                }

                fltk::enums::Event::Push => {
                    let (tx, ty) = app::event_coords();
                    s_x = tx;
                    s_y = ty;
                    f_x = tx + 1;
                    f_y = ty + 1; 
                    released = false;
                }
                
                fltk::enums::Event::Released =>{
                    let (tx, ty) = app::event_coords();
                    if ty <= t.y(){
                        f_y = t.y();
                    } else if (ty >= t.y() + t.height()) {
                        f_y = t.y() + t.height();
                    } else {
                        f_y = ty;
                    }
                    if tx <= t.x(){
                        f_x = t.x();
                    } else if (tx >= t.x() + t.width()) {
                        f_x = t.x() + t.width();
                    } else {
                        f_x = tx;
                    }

                    released = true;
                    
                    /*let overlay_x = if (tx + overlay_win.width()) > (t.x() + t.width()) {
                        (t.x() + t.width()) - overlay_win.width()
                    } else {
                        tx
                    };

                    let overlay_y = if (ty + overlay_win.height()) > (t.y() + t.height()) {
                        (t.y() + t.height()) - overlay_win.height()
                    } else {
                        ty
                    };*/

                    if !overlay_win.shown() {
                        overlay_win.show();
                        let position = app::get_mouse();
                        let rect = app::Screen::xywh_mouse();
                        let screen_w = rect.w as i32;
                        let screen_h = rect.h as i32;
                        let max_x = rect.x + rect.w - overlay_win.w();
                        let max_y = rect.y + rect.h - overlay_win.h();
                        if max_x < rect.x || max_y < rect.y {
                            overlay_win.set_pos(rect.x, rect.y);
                        } else {
                            let x = position.0.clamp(rect.x, max_x);
                            let y = position.1.clamp(rect.y, max_y);
                            overlay_win.set_pos(x, y);
                        }
                        overlay_win.take_focus();
                    } else {
                        overlay_win.show();
                        overlay_win.take_focus();
                    }
                }

                fltk::enums::Event::Drag =>{
                    
                    let (tx, ty) = app::event_coords();

                    if ty <= t.y(){
                        f_y = t.y();
                    } else if (ty >= t.y() + t.height()) {
                        f_y = t.y() + t.height();
                    } else {
                        f_y = ty;
                    }
                    if tx <= t.x(){
                        f_x = t.x();
                    } else if (tx >= t.x() + t.width()) {
                        f_x = t.x() + t.width();
                    } else {
                        f_x = tx;
                    }

                    released = false; 
                }

                _ => {
                    t.redraw();
                    if first_iter {
                        first_iter = false;
                    } else {
                        return true;
                    }
                }
            };

            t.draw(move |b|{
                //TODO(bug): if monitor.0.scale > 1 && monitor.1.scale > 1
                if screen_scale == 1.0 {
                    rgc.draw(b.x(), b.y(), x as i32, y as i32);
                } else {
                    fltk::draw::override_scale();
                    rgc.draw(b.x(), b.y(), x as i32, y as i32);
                    fltk::draw::restore_scale(screen_scale);
                }

                fltk::draw::set_draw_color(fltk::enums::Color::Yellow); 

                let coverupw = ((f_x-s_x).abs() as f32 * screen_scale) as u32;
                let coveruph = ((f_y-s_y).abs() as f32 * screen_scale) as u32;

                if coverupw > 0 && coveruph > 0 {
                    let mut sel_x: u32 = 0;
                    let mut sel_y: u32 = 0;

                    if f_x < s_x {
                        sel_x = (f_x-b.x()) as u32;
                    } else if f_x > s_x {
                        sel_x = (s_x-b.x()) as u32;
                    }
                    if f_y < s_y {
                        sel_y = (f_y-b.y()) as u32;
                    } else if f_y > s_y {
                        sel_y = (s_y-b.y()) as u32;
                    }

                    let sel_x = sel_x as f32 * screen_scale;
                    let sel_y = sel_y as f32 * screen_scale;
                    
                    let my_crop_data = image::imageops::crop(&mut grey_img_c, sel_x as u32, sel_y as u32, coverupw as u32, coveruph as u32).to_image().to_vec();
                    let mut my_crop = fltk::image::RgbImage::new(&my_crop_data, coverupw as i32, coveruph as i32, fltk::enums::ColorDepth::Rgba8).unwrap();
                    fltk::draw::override_scale();
                    my_crop.draw(sel_x as i32, sel_y as i32, coverupw as i32, coveruph as i32);
                    fltk::draw::set_draw_color(fltk::enums::Color::Yellow);
                    fltk::draw::draw_rect(sel_x as i32, sel_y as i32, coverupw as i32, coveruph as i32);
                    fltk::draw::restore_scale(screen_scale);
                    let crop_s = CropSegment::new(sel_x as i32, sel_y as i32, coverupw as i32, coveruph as i32, my_crop_data);

                    if released {
                        s_clone.send(AppEvent::OCRCropUpdate(crop_s));
                        released = false;
                    }
                }
            });
            t.redraw();
            true
        });

        sb
    }
}

fn extract_ocr_text(input: &str) -> String {
    //TODO: utils
    let start_tag = "<OCR_RESULTS_BEGIN>";
    let end_tag = "<OCR_RESULTS_END>";

    let start_idx = input.find(start_tag);
    let end_idx = input.rfind(end_tag).unwrap_or(input.len());
    let text_start = if let Some(start_idx) = start_idx { 
        start_idx + start_tag.len() 
    } else {
        0
    };

    if text_start <= end_idx {
        input[text_start..end_idx].to_string()
    } else {
        input.to_string()
    }
}
