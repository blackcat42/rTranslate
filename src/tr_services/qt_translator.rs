use debug_print::{debug_println as dprintln};
use serde_json::Value;
use crate::types::{AppEvent, Translator, Lang, UIState, TranslResult};
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time::Duration};
use anyhow::{anyhow, Result};
use super::GLOBAL_SETTINGS;
use crate::utils::helpers::is_win7_or_greater;
use base64::{prelude::BASE64_STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use crate::utils::rt_request;
use std::fs::File;
use std::io::Read;

use crate::utils::qtranslate::send_tr_request;

pub struct QT {
    is_running: Arc<AtomicBool>,
    app_sender: fltk::app::Sender<AppEvent>,
    name: String,
    uid: String,
    use_proxy: bool,
    emulation: Option<String>
}

impl QT {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>, name: String, uid: String, use_proxy: bool, emulation: Option<String>) -> Self {
        let is_running = Arc::new(AtomicBool::new(false));
        Self {is_running, app_sender, name, uid, use_proxy, emulation}
    }
}
impl Translator for QT {
    fn terminate(&mut self) {
        
    }
    fn get_uid(&self) -> &str {
        &self.uid
    }
    fn get_name(&self) -> &str {
        &self.name
    }

    fn translate(&mut self, src_id: i64, text: String, src_lang: Lang, target_lang: Lang, is_lang_detected: bool) {

        if !self.is_running.load(Ordering::SeqCst) {
            thread::spawn({
                let app_sender = self.app_sender;
                let is_running = Arc::clone(&self.is_running);
                let name = self.get_name().to_string();
                let uid = self.get_uid().to_string();
                let use_proxy = self.use_proxy;
                let emulation = self.emulation.clone(); 
                move || {
                    is_running.store(true, Ordering::SeqCst);
                                        
                    let transl_result = send_tr_request(&uid, text.clone(), src_lang.clone(), target_lang.clone(), is_lang_detected, use_proxy, emulation);
                    match transl_result {
                        Ok(t_text) => {
                            //dprintln!("lng: {}", t_text.1.unwrap_or("".to_string())); //TODO!
                             app_sender.send(AppEvent::SaveTranslation(TranslResult {
                                src_id, 
                                text: text.clone(), 
                                tr_uid: uid.clone(), 
                                src: t_text.1.clone(), 
                                target: target_lang.clone(), 
                                translation_text: t_text.0.clone()
                            }));
                            app_sender.send(AppEvent::UpdateUi(UIState {
                                src_text: text,
                                tr_uid: Some(uid), 
                                translator: Some(name), 
                                src: Some(t_text.1), 
                                target: Some(target_lang), 
                                translation_text: Some(t_text.0),
                                is_fav: None
                            }, false));
                        }
                        Err(e) => {
                            app_sender.send(AppEvent::SetReady(Some(e.to_string()), false));
                            //app_sender.send(AppEvent::SetStatus(e.to_string().as_str().into(), true, false));
                        }
                    }
                    thread::sleep(Duration::from_millis((GLOBAL_SETTINGS.http_throttling * 1000.0) as u64));
                    is_running.store(false, Ordering::SeqCst);
                }
            });
        } else {
            self.app_sender.send(AppEvent::SetReady(Some("error: rate limit".to_string()), false));
            //self.app_sender.send(AppEvent::SetStatus("error: rate limit".into(), true, false));
        }
    }
}
