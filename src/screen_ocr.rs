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

use crate::types::{
    AppEvent,
    BLWCoords
};
use crate::utils::{
    borderless_win_handler, 
    borderless_win_frame_handler
};
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


        let mut overlay_win = fltk::window::Window::default().with_size(550, 200);

        let mut frame_wrapper = fltk::group::Flex::default().column().size_of_parent();
        frame_wrapper.set_margins(3,3,3,3);

        let mut frame = fltk::group::Flex::default().column();
        frame.set_spacing(3);
        frame.set_frame(fltk::enums::FrameType::EngravedBox);

        let mut flex_titlebar = fltk::group::Flex::default().row();
        flex_titlebar.set_margins(2,2,3,0);
        flex_titlebar.set_pad(5);
        let mut close_button = fltk::button::Button::new(5, 5, 18, 18, "");
        if let Ok(image) = fltk::image::PngImage::load(working_dir.join(r"icons\close.png").to_str().unwrap_or("")) {
            close_button.set_image(Some(image));
            close_button.set_align(fltk::enums::Align::Center | fltk::enums::Align::ImageBackdrop);
        }
        flex_titlebar.fixed(&close_button, 18);
        frame.fixed(&flex_titlebar, 20);
        flex_titlebar.end();

        let mut flex = fltk::group::Flex::default().column();
        flex.set_margins(0, 0, 0, 0);
        flex.set_spacing(7);
        let ocr_text_buf = fltk::text::TextBuffer::default();
        //let ocr_waiting_buf = text::TextBuffer::default();
        let mut txt = fltk::text::TextEditor::default();
        txt.set_frame(fltk::enums::FrameType::FlatBox);
        txt.set_buffer(ocr_text_buf.clone());
        txt.wrap_mode(fltk::text::WrapMode::AtBounds, 0);

        let mut flex2 = fltk::group::Flex::default().column();
        flex2.set_spacing(5);
        let mut flex_buttons_wrapper = group::Flex::default().column();
        flex_buttons_wrapper.set_margins(15, 0, 15, 0);
        let mut flex_buttons = group::Flex::default().row();
        let mut checkbox_ocr_append = button::CheckButton::default().with_label("Keep previous text")
            .with_align(fltk::enums::Align::Inside | fltk::enums::Align::Left | fltk::enums::Align::ImageNextToText);
        flex_buttons.fixed(&checkbox_ocr_append, 200);
        flex_buttons.end();
        flex2.fixed(&flex_buttons_wrapper, 15);
        flex_buttons_wrapper.end();
        let mut flex_buttons_wrapper2 = group::Flex::default().column();
        flex_buttons_wrapper2.set_margins(15, 0, 15, 0);
        let flex_buttons2 = group::Flex::default().row();
        let mut btn_ocr = button::Button::new(0, 15, 100, 40, "Run OCR")
            .with_align(fltk::enums::Align::Center | fltk::enums::Align::ImageNextToText);
        let mut btn_copy = button::Button::new(0, 15, 100, 40, "Copy")
            .with_align(fltk::enums::Align::Center | fltk::enums::Align::ImageNextToText);
        let mut btn_translate = button::Button::new(0, 15, 100, 40, "Translate")
            .with_align(fltk::enums::Align::Center | fltk::enums::Align::ImageNextToText);
        let mut btn_exit = button::Button::new(0, 15, 100, 40, "Exit (Esc)")
            .with_align(fltk::enums::Align::Center | fltk::enums::Align::ImageNextToText);
        flex_buttons2.end();
        flex2.fixed(&flex_buttons_wrapper2, 25);
        flex_buttons_wrapper2.end();
        flex.fixed(&flex2, 50);
        flex2.end();

        flex.end();
        frame.end();
        frame_wrapper.end();
        overlay_win.make_resizable(true);
        overlay_win.set_border(false);
        overlay_win.set_frame(fltk::enums::FrameType::UpBox);
        overlay_win.resizable(&overlay_win);
        overlay_win.size_range(400, 150, 0 ,0);
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
        let monitor = Monitor::from_point(50, 50).unwrap(); //TODO!!!
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

        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let command = ".\\rt_ocr".to_string();

        let mut child;
        if which::which(&command).is_ok() {
            child = std::process::Command::new(working_dir.join(&command))
                .arg("--pipe")
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
        // println!("image depth is {}", img.depth() );
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
                    
                    let overlay_x = if (tx + overlay_win.width()) > (t.x() + t.width()) {
                        (t.x() + t.width()) - overlay_win.width()
                    } else {
                        tx
                    };

                    let overlay_y = if (ty + overlay_win.height()) > (t.y() + t.height()) {
                        (t.y() + t.height()) - overlay_win.height()
                    } else {
                        ty
                    };

                    if !overlay_win.shown() {
                        overlay_win.show();
                        overlay_win.set_pos(overlay_x, overlay_y);
                    } else {
                        overlay_win.show();
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
                rgc.draw(b.x(), b.y(), x as i32, y as i32);
                fltk::draw::set_draw_color(fltk::enums::Color::Yellow);
                let coverupw = (f_x-s_x).abs() as u32;
                let coveruph = (f_y-s_y).abs() as u32;

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

                    let my_crop_data = image::imageops::crop(&mut grey_img_c, sel_x, sel_y, coverupw, coveruph).to_image().to_vec();
                    let mut my_crop = fltk::image::RgbImage::new(&my_crop_data, coverupw as i32, coveruph as i32, fltk::enums::ColorDepth::Rgba8).unwrap();
                    my_crop.draw(sel_x as i32, sel_y as i32, coverupw as i32, coveruph as i32);
                    fltk::draw::draw_rect(sel_x as i32, sel_y as i32, coverupw as i32, coveruph as i32);

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
