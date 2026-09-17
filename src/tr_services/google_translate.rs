use debug_print::{debug_println as dprintln};
use serde_json::Value;
use crate::types::{AppEvent, Translator, Lang, UIState, TranslResult, TranslatorOption};

use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time::Duration};
use anyhow::{anyhow, Result};
use super::GLOBAL_SETTINGS;
use std::str::FromStr;
use std::collections::HashMap;

use crate::utils::rt_request::{
    Client
};
use crate::utils::helpers::google_tk;

pub struct GT {
    is_running: Arc<AtomicBool>,
    app_sender: fltk::app::Sender<AppEvent>,
    options: TranslatorOption
}

impl GT {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>, options: TranslatorOption) -> Result<Self> {
        let is_running = Arc::new(AtomicBool::new(false));
        //let uid = "tr_google".to_string();
        Ok(Self {is_running, app_sender, options})
    }
}
impl Translator for GT {
    fn terminate(&mut self) {
        
    }
    fn is_processing(&self) -> bool {
        false
    }
    fn get_uid(&self) -> &str {
        &self.options.uid
    }
    fn get_name(&self) -> &str {
        &self.options.name
    }

    fn translate(&mut self, src_id: i64, text: String, src_lang: Lang, target_lang: Lang, is_lang_detected: bool) {

        if !self.is_running.load(Ordering::SeqCst) {
            thread::spawn({
                let app_sender = self.app_sender;
                let is_running = Arc::clone(&self.is_running);
                let name = self.get_name().to_string();
                let uid = self.get_uid().to_string();
                let use_proxy = self.options.use_proxy;
                let emulation = self.options.emulation.clone();
                move || {
                    is_running.store(true, Ordering::SeqCst);
                    
                    
                    let transl_result = send_tr_request(text.clone(), src_lang.clone(), target_lang.clone(), is_lang_detected, use_proxy, emulation);
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
                                src_text: text.clone(),
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


fn send_tr_request(selected_text: String, src_lang: Lang, target_lang: Lang, is_lang_detected: bool, proxy: bool, emulation: Option<String>) -> Result<(String, Lang)> {
    let mut response = "".to_string();

    let src_lang_ref = if is_lang_detected {
        src_lang.as_ref()
    } else {
        "auto"
    };
    let tk = if let Ok(token) = google_tk(&selected_text, "") {
        format!("&tk={}", token)
    } else {
        "".to_string()
    };

    let req_string = format!("https://translate.googleapis.com/translate_a/single?client=gtx&sl={}&dt=t&tl={}{}", src_lang_ref, target_lang.as_ref(), tk);
    dprintln!("{}", req_string);


    let mut headers = HashMap::new();
    headers.insert("User-Agent".into(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.104 Safari/537.36".into());

    // Create a new req client
    let mut client = Client::builder()
        //.emulation(Emulation::Chrome137)
        .timeout(Duration::from_secs(GLOBAL_SETTINGS.http_request_timeout))
        .default_headers(headers)
        .proxy(proxy);
    if let Some(e) = emulation {
        client = client.emulation(e);
    }
    let client = client.build()?;
    
    let resp = client.get(req_string).query([("q", selected_text)]).send()?.text()?;
    dprintln!("{}", resp);
    let result =  Ok(resp);

    match result {
        Ok(json_data) => {
            let value: Value = serde_json::from_str(json_data.as_str())?;
            let mut src_lng_suggested = src_lang.clone();

            if let Some(tr_items) = value[0].as_array() {
                for item_value in tr_items {
                    if let Some(text) = item_value[0].as_str() {
                        response.push_str(text);
                        //dprintln!("{}", text);
                    }
                }
                
                if let Some(lang) = value[8][0][0].as_str() {
                    src_lng_suggested = Lang::from_str(lang).unwrap_or(src_lang);
                }
            }

            if response.chars().nth(1).is_some() {
                Ok((response, src_lng_suggested))
            } else {
                Err(anyhow!("error"))
            }
        }
        Err(err) => {
            Err(err)
        }
    }
}
