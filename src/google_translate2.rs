use serde_json::Value;
use crate::types::{AppEvent, Translator, Lang, UIState};
//use ureq::Agent;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time::Duration};
use anyhow::{anyhow, Result};
use super::GLOBAL_SETTINGS;
use super::TOKIO_RT;

use wreq::{
    Client,
    Version,
    header
};
use wreq_util::{
    Emulation
};

pub struct GT {
    is_running: Arc<AtomicBool>,
    app_sender: fltk::app::Sender<AppEvent>,
    name: String,
    uid: String
}

impl GT {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>, name: String, uid: String) -> Self {
        let is_running = Arc::new(AtomicBool::new(false));
        //let uid = "tr_google2".to_string();
        Self {is_running, app_sender, name, uid}
    }
}
impl Translator for GT {
    fn terminate(&mut self) {
        
    }
    fn get_uid(&self) -> String {
        self.uid.clone()
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn translate(&mut self, src_id: i64, text: String, src_lang: Lang, target_lang: Lang, is_lang_detected: bool) {

        if !self.is_running.load(Ordering::SeqCst) {
            thread::spawn({
                let app_sender = self.app_sender;
                let is_running = Arc::clone(&self.is_running);
                let name = self.get_name();
                let uid = self.get_uid();
                move || {
                    is_running.store(true, Ordering::SeqCst);

                    let transl_result = send_tr_request(text.clone(), src_lang.clone(), target_lang.clone(), is_lang_detected);
                    match transl_result {
                        Ok(t_text) => {
                            //println!("lng: {}", t_text.1); //TODO!
                            app_sender.send(AppEvent::SaveTranslation((src_id, text.clone(), uid.clone(), src_lang.clone(), target_lang.clone(), t_text.0.clone())));
                            app_sender.send(AppEvent::UpdateUi(UIState {
                                src_text: text,
                                tr_uid: Some(uid), 
                                translator: Some(name), 
                                src: Some(src_lang), 
                                target: Some(target_lang), 
                                translation_text: Some(t_text.0),
                                is_fav: None
                            }, false));
                        }
                        Err(_e) => {
                            app_sender.send(AppEvent::SetReady());
                            app_sender.send(AppEvent::SetStatus("translation failed (https req error)".into(), true, false));
                        }
                    }
                    thread::sleep(Duration::from_millis((GLOBAL_SETTINGS.http_throttling * 1000.0) as u64));
                    is_running.store(false, Ordering::SeqCst);
                }
            });
        } else {
            self.app_sender.send(AppEvent::SetReady());
            self.app_sender.send(AppEvent::SetStatus("error: rate limit".into(), true, false));
        }
    }
}


fn send_tr_request(selected_text: String, src_lang: Lang, target_lang: Lang, is_lang_detected: bool) -> Result<(String, Option<String>)> {
    let mut response = "".to_string();

    let src_lang = if is_lang_detected {
        src_lang.as_ref()
    } else {
        "auto"
    };

    let rt = TOKIO_RT.get_or_init(|| {
        tokio::runtime::Runtime::new().expect("Tokio Runtime Error")
    });

    let result = rt.block_on(async {
        //TODO: /v1/translateHtml does not preserve line breaks (and if we put an array of strings, they will not share context)

        //let selected_text: String = selected_text.lines().map(|s| serde_json::to_string(s).unwrap_or("\"\"".to_string())).filter(|item| *item != "\"\"".to_string()).collect::<Vec<String>>().join(","); //array of strings (serialized)
        let selected_text = serde_json::to_string(&selected_text)?;
        let src_lang = serde_json::to_string(src_lang)?;
        let target_lang = serde_json::to_string(target_lang.as_ref())?;

        let req_body = format!("[[[{}],{},{}],\"wt_lib\"]", selected_text, src_lang, target_lang);
        println!("{}", req_body);
        let mut headers = header::HeaderMap::new();
        headers.insert("Host", header::HeaderValue::from_static("translate-pa.googleapis.com"));
        headers.insert("X-Goog-API-Key", header::HeaderValue::from_static("AIzaSyATBXajvzQLTDHEQbcpq0Ihe0vWDHmO520"));
        headers.insert("Content-Type", header::HeaderValue::from_static("application/json+protobuf"));

        let client = wreq::Client::builder()
            .emulation(Emulation::Chrome137)
            .default_headers(headers)
            .timeout(Duration::from_secs(GLOBAL_SETTINGS.http_request_timeout))
            .user_agent("User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36")
            .build()?;
        let resp = client.post("https://translate-pa.googleapis.com/v1/translateHtml").version(Version::HTTP_11).body(req_body).send().await?.text().await?;

        println!("{}", resp);
        Ok(resp)
    });

    match result {
        Ok(json_data) => {
            let value: Value = serde_json::from_str(json_data.as_str())?;
            let mut src_lng_suggested = None;
            if let Some(items) = value.as_array()
                && items.get(0).is_some() 
                && let Some(tr_items) = items[0].as_array() {
                    for item_value in tr_items {
                        if let Some(text) = item_value.as_str() {
                            response.push_str(text);
                            response.push_str("\n");
                            //println!("{}", text);
                        }
                    };

                    if let Some(arr1) = items.get(1) && let Some(lang) = arr1.get(0) {
                        src_lng_suggested = Some(lang.to_string());
                    }
                }
            if response.chars().count() > 1 {
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
