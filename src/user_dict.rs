//use serde_json::Value;
#![allow(clippy::len_zero)]

use crate::types::{AppEvent, Dictionary, Lang, UIStateDict};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread};
use std::io::{Seek, SeekFrom};
use std::io::{BufRead};
//use std::path::Path;
use std::path::PathBuf;
//use super::GLOBAL_SETTINGS;
use rusqlite::{params, Connection};
use anyhow::{Result};
use std::rc::Rc;
use std::cell::RefCell;

use std::fs::File; 
use std::io::BufReader;
use std::sync::{Arc};
use fltk::{app, dialog, };
use regex::Regex;

pub struct DSLDict {
    is_running: Arc<AtomicBool>,
    app_sender: fltk::app::Sender<AppEvent>,
    uid: String,
    name: String,
    dict_path: String,
    db: Rc<RefCell<Option<Connection>>>,
}

//TODO: multiple titles support (not allowed by spec, but widely used)

impl DSLDict {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>, uid: String, name: String, dict_path: String, db: Rc<RefCell<Option<Connection>>>) -> Self {

        let re_uid = Regex::new(r"^\w+$").unwrap();
        if !re_uid.is_match(&uid) {
            //app_panic_message("settings.json: Failed to parse uid");
            panic!("settings.json: Failed to parse uid");
        }

        let is_running = Arc::new(AtomicBool::new(false));

        //TODO: return result
        println!("create db");
        let db_ref = db.borrow();
        if db_ref.is_none() {
            //return Err(());
        }
        if let Some(db) = &*db_ref {
            db.execute(
                "CREATE TABLE IF NOT EXISTS user_dicts_metadata (
                    dict_uid TEXT PRIMARY KEY,
                    is_indexed INTEGER DEFAULT 0
                )",
                params![],
            ).unwrap();

            db.execute(
                "INSERT INTO user_dicts_metadata (dict_uid, is_indexed) VALUES (?1, 0)",
                params![uid],
            ).unwrap_or(0);
        }
        drop(db_ref);

        Self {is_running, app_sender, uid, name, dict_path, db}
    }

    pub fn rebuild_index(&self) -> Result<()> {

        if self.is_running.load(Ordering::SeqCst) {
            return Ok(());
        }
        
        let db_ref = self.db.borrow();
        if db_ref.is_none() {
            self.app_sender.send(AppEvent::SetStatus("An error occurred while opening the database file".into(), true, true));
            return Ok(());
        }

        println!("create db");
        if let Some(db) = &*db_ref {

            let q = format!("DROP TABLE IF EXISTS {}", self.uid);
            db.execute(
                &q,
                params![],
            )?;

            let q = format!("CREATE TABLE IF NOT EXISTS {} (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    title TEXT NOT NULL,
                    offset INTEGER NOT NULL
                )", self.uid);
            db.execute(
                &q,
                params![],
            )?;

            let q = format!("CREATE INDEX IF NOT EXISTS title_index ON {} (title)", self.uid);
            db.execute(
                &q,
                params![],
            )?;


            println!("start");
            //parse dsl file
            let path = PathBuf::from(self.dict_path.clone());
            let file = File::open(path.clone())?;
            let metadata = file.metadata().expect("Failed to get file metadata");
            let mut reader = BufReader::new(file);
            
            let filesize_mb = (metadata.len() / 1048576) as f64;
            
            thread::spawn({
                let app_sender = self.app_sender;
                let is_running = Arc::clone(&self.is_running);
                //let index_db = Arc::clone(&self.db);
                
                //let name = self.get_name();
                let uid = self.get_uid();
                move || {
                    is_running.store(true, Ordering::SeqCst);
                    let mut line_num = 1;
                    let mut articles_num = 0;
                    let mut buffer: Vec<u8> = Vec::new();
                    let mut bom_offset = 2;

                    let mut index_db = Connection::open("dictionary_index.db").unwrap();
                    let tx = index_db.transaction().unwrap();
                    //let q = format!("INSERT INTO {} (title, offset) VALUES (?2, ?3)", uid);
                    //let mut stmt = tx.prepare(&q).unwrap();
                    loop {
                        buffer.clear();
                        let position = reader.stream_position().unwrap();
                        let bytes_read = reader.read_until(0x0A, &mut buffer).unwrap(); //find utf8 lf in utf16

                        if line_num % 100 == 0 {
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
                            let _ = tx.execute(
                                "REPLACE INTO user_dicts_metadata (dict_uid, is_indexed) VALUES (?1, 1)",
                                params![uid],
                            ).unwrap();
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
                                    let q = format!("REPLACE INTO {} (title, offset) VALUES (?2, ?3)", uid);
                                    let _ = tx.execute(
                                        &q,
                                        params![uid, &s, position as i64],
                                    ).unwrap();
                                    articles_num += 1;
                                }
                            },
                            Err(e) => eprintln!("Error decoding UTF-16: {}", e),
                        }
                        line_num += 1;
                    }
                    tx.commit().unwrap();
                    is_running.store(false, Ordering::SeqCst);
                    app_sender.send(AppEvent::SetStatus("Dictionary index created. Please retry request or make a new one.".into(), true, true));
                }
            });
        };
        Ok(())
    }

}
impl Dictionary for DSLDict {
    fn terminate(&mut self) {}

    fn get_uid(&self) -> String {
        self.uid.clone()
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn translate(&mut self, src_id: i64, text: String, _src_lang: Lang,_target_lang: Lang) {

        if self.is_running.load(Ordering::SeqCst) {
            return;
        }
        let orig_text = text.clone();

        //TODO: from settings
        let text = text.to_lowercase();
        
        let db_ref = self.db.borrow();
        if db_ref.is_none() {
            self.app_sender.send(AppEvent::SetReady());
            self.app_sender.send(AppEvent::SetStatus("An error occurred while opening the database file".into(), true, true));
            return;
        }

        //TODO save check result
        let mut is_indexed = false;
        if let Some(db) = &*db_ref {
            let is_indexed_n = db.query_row(
                    "SELECT is_indexed FROM user_dicts_metadata 
                             WHERE dict_uid = ?1",
                    params![&self.get_uid()], //text
                    |row| {
                        Ok(row.get(0).unwrap_or(0))
                    },
            );
            is_indexed = is_indexed_n.unwrap_or(0) != 0;
        }

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
                    self.app_sender.send(AppEvent::SetReady());
                    self.app_sender.send(AppEvent::SetStatus("index not found".into(), true, true));
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

        if let Some(db) = &*db_ref {
            println!("open db dict");
            let app_sender = self.app_sender;
            //let name = self.get_name();

            let mut offset: u64 = 0;
            let q = format!("SELECT title, offset FROM {} 
                             WHERE title LIKE ?1 ORDER BY 
                              CASE 
                                WHEN title = ?2 THEN 1
                                ELSE 2 
                              END,
                              title;", self.uid);
            let pattern = format!("{}%", text);
            let index = db.query_row(
                    &q,
                    params![&pattern, &text], //text
                    |row| {
                        let title = row.get(0).unwrap_or("".to_string());
                        let offset = row.get(1).unwrap_or(0_i64);
                        Ok((title, offset))
                    },
                );
            println!("dict query end");
            match index {
                    Ok(row) => {
                        println!("cached src found");
                        println!("{}", offset);
                        offset = row.1 as u64;
                    }
                    Err(e) => {
                        //app_sender.send(AppEvent::SetReady());
                        println!("{}", e);
                    }
                } 

            //get offset by title
            
            if offset == 0 {
                app_sender.send(AppEvent::SetReady());
                app_sender.send(AppEvent::SetStatus("not found".into(), true, true));
                return;
                //return anyhow!("error");
            }
            let transl_result = send_tr_request(&self.dict_path, offset);
            match transl_result {
                Ok(t_text) => {
                    app_sender.send(AppEvent::SaveDictEntry((src_id, orig_text.clone(), self.get_uid(), t_text.clone() )));
                    app_sender.send(AppEvent::UpdateUiDict(UIStateDict {
                        src_id: Some(src_id),
                        src_text_dict: orig_text.clone(),
                        dict_uid: Some(self.get_uid()), 
                        dict_name: Some(self.get_name()),
                        //src: src_lang.clone(), 
                        //target: target_lang.clone(), 
                        dict_text: Some(t_text),
                        is_fav: None
                    }, false));
                    app_sender.send(AppEvent::SetReady());
                }
                Err(_e) => {
                    app_sender.send(AppEvent::SetReady());
                    app_sender.send(AppEvent::SetStatus("error".into(), true, true));
                }
            };
        };
    }
}

fn send_tr_request(path: &str, offset: u64) -> Result<String> {
    let mut file = File::open(path)?;
    let response = read_line_at_offset(&mut file, offset)?;
    Ok(response)
}

fn read_line_at_offset(file: &mut File, offset: u64) -> std::io::Result<String> {

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
                //println!("{}", decoded_string);
                if is_title && (decoded_string.starts_with("\t") || decoded_string.starts_with(" ")) {
                    is_title = false;
                }
                if !is_title && !decoded_string.starts_with("\t") && !decoded_string.starts_with(" ") {
                    break;
                }
                if line_num > 150 {
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
