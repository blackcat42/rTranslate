//use serde_json::Value;
#![allow(clippy::len_zero)]
use debug_print::{debug_println as dprintln};

use crate::types::{AppEvent, Dictionary, Lang, UIStateDict, DictResult};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread};
use std::io::{Seek, SeekFrom};
use std::io::{Read, BufRead, BufWriter, Write};
//use std::path::Path;
use std::path::PathBuf;
//use super::GLOBAL_SETTINGS;
use anyhow::{anyhow, Result};
use std::rc::Rc;
use std::cell::RefCell;

use std::fs::File; 
use std::io::BufReader;
use std::sync::{Arc};
use fltk::{app, dialog, };
use regex::Regex;

use super::app_panic_message;

pub struct DSLDict {
    is_running: Arc<AtomicBool>,
    app_sender: fltk::app::Sender<AppEvent>,
    uid: String,
    name: String,
    dict_path: String
}

//TODO: multiple titles support (not allowed by spec, but widely used)

impl DSLDict {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>, uid: String, name: String, dict_path: String) -> Self {

        let re_uid = Regex::new(r"^\w+$").unwrap();
        if !re_uid.is_match(&uid) {
            app_panic_message("settings.json: Failed to parse uid");
            panic!("settings.json: Failed to parse uid");
        }

        let is_running = Arc::new(AtomicBool::new(false));
        Self {is_running, app_sender, uid, name, dict_path}
    }

    pub fn rebuild_index(&self) -> Result<()> {

        if self.is_running.load(Ordering::SeqCst) {
            return Ok(());
        }

        dprintln!("rebuild_index start");
        //parse dsl file
        let path = PathBuf::from(self.dict_path.clone());
        let index_path = path.with_extension("idx");
        let file = File::open(path.clone())?;
        let mut idx_file = File::create(index_path)?;
        let metadata = file.metadata()?;
        let mut reader = BufReader::new(file);
        let filesize_mb = (metadata.len() / 1048576) as f64;
        
        thread::spawn({
            let app_sender = self.app_sender;
            let is_running = Arc::clone(&self.is_running);
            move || {
                let _ = ( || -> Result<()> {
                    is_running.store(true, Ordering::SeqCst);
                    let mut line_num = 1;
                    let mut articles_num = 0;
                    let mut buffer: Vec<u8> = Vec::new();
                    let mut bom_offset = 2;

                    let mut writer = BufWriter::with_capacity(512 * 1024, idx_file);
                    loop {
                        buffer.clear();
                        let position = reader.stream_position()?;
                        let bytes_read = reader.read_until(0x0A, &mut buffer)?; //find utf8 lf in utf16

                        if line_num % 1000 == 0 {
                            let pos_in_mb: f64 = position as f64 / 1048576_f64;
                            let status_str = format!("processed {:.2}/{} mb; articles indexed: {}", pos_in_mb, filesize_mb, articles_num);
                            app_sender.send(AppEvent::SetStatus(status_str.as_str().into(), true, true));
                            app::awake();
                            app::redraw();
                        }

                        if line_num == 1 && buffer[0] != 0xFF && buffer[1] != 0xFE {
                            bom_offset = 0; //utf-16le w/o BOM or not a utf-16le
                            if buffer.len() >= (10 + bom_offset) 
                               && buffer[0 + bom_offset] != 0x23 
                               && buffer[2 + bom_offset] != 0x4E { 
                                break;
                            }
                            //23 00  4E 00  41 00  4D 00  45 00 (#NAME)
                        }
                        if bytes_read == 0 {
                            break; //end of file
                        }
                        
                        if line_num == 1 {
                            buffer.remove(0); //remove bom (first byte) todo:
                        }
                        if buffer.len() < 1 {
                            continue;
                        }

                        buffer.remove(0); //remove bom (second byte) OR remove tail byte of linefeed from prev chunk
                        buffer.push(0x00); //restore little-endian linefeed
                        
                        let utf16_vec = convert_u8_to_u16(buffer.clone());
                        match String::from_utf16(&utf16_vec) {
                            Ok(s) => {
                                if !s.starts_with("\t") && !s.starts_with(" ") && !s.starts_with("\n") {
                                    let s = s.trim();
                                    writer.write_all(&position.to_le_bytes())?;
                                    articles_num += 1;
                                }
                            },
                            Err(e) => eprintln!("Error decoding UTF-16: {}", e),
                        }
                        line_num += 1;
                    }
                    writer.flush()?;
                    //tx.commit().unwrap();
                    is_running.store(false, Ordering::SeqCst);
                    app_sender.send(AppEvent::SetStatus("Dictionary index created. Please retry request or make a new one.".into(), true, true));
                    Ok(())
                })();
            }
        });
        Ok(())
    }

}
impl Dictionary for DSLDict {
    fn terminate(&mut self) {}

    fn get_uid(&self) -> &str {
        &self.uid
    }
    fn get_name(&self) -> &str {
        &self.name
    }

    fn translate(&mut self, src_id: i64, text: String, _src_lang: Lang,_target_lang: Lang) {

        if self.is_running.load(Ordering::SeqCst) {
            return;
        }
        let orig_text = text.clone();

        //TODO: from settings
        let text = text.to_lowercase();

        let dsl_path = PathBuf::from(&self.dict_path);
        let index_path = dsl_path.with_extension("idx"); 

        let mut is_indexed = index_path.exists();

        if !is_indexed {
            let pos = screen_center();
            let choice = dialog::choice2(
                pos.0, pos.1, 
                "Create index for the selected dictionary (may take several minutes)?", 
                "No",
                "Yes",
                ""
            );
            match choice {
                Some(0) => {
                    // User clicked "No"
                    self.app_sender.send(AppEvent::SetReady(Some("index not found".to_string()), true));
                    //self.app_sender.send(AppEvent::SetStatus("index not found".into(), true, true));
                    return;
                }
                Some(1) => {
                    let _ = self.rebuild_index();
                    // User clicked "Yes"
                    //dialog::message(100, 100, "Action confirmed. Proceeding...");
                }
                _ => {
                    // Dialog closed without a choice, treat as cancellation
                }
            };
        }


        let transl_result = send_tr_request(&self.dict_path, &text);
        match transl_result {
            Ok(t_text) => {
                //self.app_sender.send(AppEvent::SaveDictEntry((src_id, orig_text.clone(), self.get_uid().to_string(), t_text.clone(), None, None )));
                self.app_sender.send(AppEvent::SaveDictEntry(DictResult {
                    src_id: src_id, 
                    dict_uid: self.get_uid().to_string(),
                    text: t_text.clone(),
                    src: None, 
                    target: None
                }));
                self.app_sender.send(AppEvent::UpdateUiDict(UIStateDict {
                    src_id: Some(src_id),
                    src_text_dict: orig_text.clone(),
                    dict_uid: Some(self.get_uid().to_string()), 
                    dict_name: Some(self.get_name().to_string()),
                    src: None, 
                    target: None, 
                    dict_text: Some(t_text),
                    is_fav: None
                }, false));
                self.app_sender.send(AppEvent::SetReady(None, true));
            }
            Err(e) => {
                self.app_sender.send(AppEvent::SetReady(Some(e.to_string()), true));
                //app_sender.send(AppEvent::SetStatus("error".into(), true, true));
            }
        };
    }
}

fn send_tr_request(path: &str, term: &str) -> Result<String> {
    let path_buf = PathBuf::from(path);
    let mut file = File::open(&path_buf)?;
    let index_path = path_buf.with_extension("idx");
    let mut idx_file = File::open(&index_path)?;
    //let response = read_line_at_offset(&mut file, offset, false)?;
    let response = search_term(&mut file, &mut idx_file, term)?;
    Ok(response)
}

pub fn search_term(mut dsl_file: &mut File, idx_file: &mut File, target_term: &str) -> Result<String> {

    let target_term = target_term.to_lowercase();

    let total_bytes = idx_file.seek(SeekFrom::End(0))?;
    let total_records = total_bytes / 8;

    if total_records == 0 {
        return Err(anyhow!("error"));
    }

    let mut left: i64 = 0;
    let mut right: i64 = (total_records - 1) as i64;

    while left <= right {
        let mid = left + (right - left) / 2;

        idx_file.seek(SeekFrom::Start((mid as u64) * 8))?;
        
        let mut buf = [0u8; 8];
        idx_file.read_exact(&mut buf)?;

        let offset = u64::from_le_bytes(buf[0..8].try_into()?);

        //dprintln!("offset: {}", offset);
        let current_term = read_line_at_offset(dsl_file, offset, true)?;
        let current_term = current_term.trim().to_lowercase();
        //dprintln!("current_term: {}", current_term);

        if current_term == target_term {
            let definition = read_line_at_offset(dsl_file, offset, false)?;
            return Ok(definition);
        } else if current_term < target_term {
            left = mid + 1;
        } else {
            right = mid - 1;
        }
    }
    Err(anyhow!("error not found"))
}

fn read_line_at_offset(file: &mut File, offset: u64, title_only: bool) -> std::io::Result<String> {

    file.seek(SeekFrom::Start(offset))?;

    let mut reader = BufReader::new(file);
    let mut buffer: Vec<u8> = Vec::new();
    let mut line_num = 0;
    let mut is_title = true;

    let mut result_string = "".to_string();

    loop {
        buffer.clear();

        let bytes_read = reader.read_until(0x0A, &mut buffer)?;

        if bytes_read == 0 {
            break;
        }

        if offset == 0 {
            buffer.remove(0); //remove bom (first byte)
        }

        buffer.remove(0); //remove bom (second byte) OR remove tail byte of linefeed from prev chunk
        buffer.push(0x00); //restore little-endian linefeed

        let utf16_vec = convert_u8_to_u16(buffer.clone());
        match String::from_utf16(&utf16_vec) {
            Ok(decoded_string) => {
                //dprintln!("{}", decoded_string);
                if is_title && (decoded_string.starts_with("\t") || decoded_string.starts_with(" ")) {
                    is_title = false;
                }
                if !is_title && !decoded_string.starts_with("\t") && !decoded_string.starts_with(" ") {
                    break;
                }
                if line_num > 150 {
                    break;
                }
                if title_only && !is_title {
                    break;
                }
                result_string.push_str(&decoded_string);
            },
            Err(e) => {
                eprintln!("Error decoding UTF-16: {}", e);
            }
        }
        line_num += 1;
    }
    Ok(result_string)
}

fn convert_u8_to_u16(data_u8: Vec<u8>) -> Vec<u16> {
    // Ensure the vector length is even (each u16 needs two u8s)
    //if data_u8.len() % 2 != 0 {
    if !data_u8.len().is_multiple_of(2) {
        return vec![0];
    }

    // Convert to Vec<u16> using little-endian byte order
    let data_u16_le: Vec<u16> = data_u8
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    // Convert to Vec<u16> using big-endian byte order
    /*let data_u16_be: Vec<u16> = data_u8
        .chunks_exact(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .collect();*/

    data_u16_le
}



pub fn screen_center() -> (i32, i32) {
    (
        (app::screen_size().0 / 2.0) as i32,
        (app::screen_size().1 / 2.0) as i32,
    )
}
