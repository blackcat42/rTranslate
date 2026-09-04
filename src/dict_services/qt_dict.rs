#![allow(clippy::collapsible_if)]

use debug_print::{debug_println as dprintln};
use serde_json::Value;
use anyhow::{anyhow, Result};

use super::GLOBAL_SETTINGS;
use crate::types::{AppEvent, Dictionary, Lang, UIStateDict, DictResult};
use crate::utils::rt_request::{
    Client,
};

use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time::Duration};
use std::str::FromStr;

use crate::utils::qtranslate::send_dict_request;

pub struct QTDict {
    is_running: Arc<AtomicBool>,
    app_sender: fltk::app::Sender<AppEvent>,
    name: String,
    uid: String,
    use_proxy: bool,
    emulation: Option<String>
}

impl QTDict {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>, name: String, uid: String, use_proxy: bool, emulation: Option<String>) -> Self {
        let is_running = Arc::new(AtomicBool::new(false));
        Self {is_running, app_sender, name, uid, use_proxy, emulation}
    }
}
impl Dictionary for QTDict {
    fn terminate(&mut self) {}

    fn get_uid(&self) -> &str {
        &self.uid
    }
    fn get_name(&self) -> &str {
        &self.name
    }

    fn translate(&mut self, src_id: i64, text: String, mut src_lang: Lang, target_lang: Lang) {

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

                    let transl_result = send_dict_request(&uid, text.clone(), src_lang.clone(), target_lang.clone(), use_proxy, emulation);
                    match transl_result {
                        Ok(t_text) => {
                            
                            //dprintln!("lng: {}", text_d.1.unwrap_or("".to_string()));
                            src_lang = t_text.1;

                            app_sender.send(AppEvent::SaveDictEntry(DictResult {
                                src_id, 
                                dict_uid: uid.clone(),
                                text: t_text.0.clone(),
                                src: Some(src_lang.clone()), 
                                target: Some(target_lang.clone())
                            }));

                            app_sender.send(AppEvent::UpdateUiDict(UIStateDict {
                                src_id: Some(src_id),
                                src_text_dict: text.clone(),
                                dict_uid: Some(uid), 
                                dict_name: Some(name), 
                                src: Some(src_lang), 
                                target: Some(target_lang),
                                dict_text: Some(t_text.0),
                                is_fav: None
                            }, false));
                            
                        }
                        Err(e) => {
                            app_sender.send(AppEvent::SetReady(Some(e.to_string()), true));
                        }
                    }
                    thread::sleep(Duration::from_millis((GLOBAL_SETTINGS.http_throttling * 1000.0) as u64));
                    is_running.store(false, Ordering::SeqCst);
                }
            });
        } else {
            self.app_sender.send(AppEvent::SetReady(Some("error: rate limit".to_string()), true));
        }
    }
}
