use debug_print::{debug_println as dprintln};
use serde_json::Value;
use crate::types::{AppEvent, Translator, Lang, UIState, TranslResult};
//use ureq::Agent;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time::Duration};
use anyhow::{anyhow, Result};
use super::GLOBAL_SETTINGS;
use super::TOKIO_RT;
use std::str::FromStr;

use wreq::{
    Client,
    Version,
    header
};
use wreq_util::{
    Emulation
};

pub struct GT2 {
    is_running: Arc<AtomicBool>,
    app_sender: fltk::app::Sender<AppEvent>,
    name: String,
    uid: String,
    use_proxy: bool
}

impl GT2 {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>, name: String, uid: String, use_proxy: bool) -> Self {
        let is_running = Arc::new(AtomicBool::new(false));
        //let uid = "tr_google2".to_string();
        Self {is_running, app_sender, name, uid, use_proxy}
    }
}
impl Translator for GT2 {
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
                move || {
                    is_running.store(true, Ordering::SeqCst);
                    let mut proxy: Option<wreq::Proxy> = None;
                    if use_proxy && let Some(proxy_settings) = &GLOBAL_SETTINGS.proxy {
                        let proxy_url = &proxy_settings.url;
                        if let Ok(mut wreq_proxy) = wreq::Proxy::all(proxy_url) {
                            wreq_proxy = if let Some(username) = &proxy_settings.username && let Some(password) = &proxy_settings.password {
                                wreq_proxy.basic_auth(username, password)
                            } else {
                                wreq_proxy
                            };
                            proxy = Some(wreq_proxy);
                        }
                        
                    }
                    let transl_result = send_tr_request(text.clone(), src_lang.clone(), target_lang.clone(), is_lang_detected, proxy);
                    match transl_result {
                        Ok(t_text) => {
                            //dprintln!("lng: {}", t_text.1.unwrap_or("".to_string())); //TODO!
                             app_sender.send(AppEvent::SaveTranslation(TranslResult {
                                src_id: src_id, 
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


fn send_tr_request(selected_text: String, src_lang: Lang, target_lang: Lang, is_lang_detected: bool, proxy: Option<wreq::Proxy>) -> Result<(String, Lang)> {
    let mut response = "".to_string();

    let src_lang_ref = if is_lang_detected {
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
        let src_lang_ref = serde_json::to_string(src_lang_ref)?;
        let target_lang = serde_json::to_string(target_lang.as_ref())?;

        let req_body = format!("[[[{}],{},{}],\"wt_lib\"]", selected_text, src_lang_ref, target_lang);
        dprintln!("{}", req_body);
        let mut headers = header::HeaderMap::new();
        headers.insert("Host", header::HeaderValue::from_static("translate-pa.googleapis.com"));
        headers.insert("User-Agent", header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36"));
        let api_key = GLOBAL_SETTINGS.google_translate_api_key.clone();
        match api_key {
            Some(api_key) => {
                let api_key = header::HeaderValue::from_str(&api_key)?;
                headers.insert("X-Goog-API-Key", api_key);
                headers.insert("Content-Type", header::HeaderValue::from_static("application/json+protobuf"));

                let mut client = wreq::Client::builder()
                    .emulation(Emulation::Chrome137)
                    .default_headers(headers)
                    .timeout(Duration::from_secs(GLOBAL_SETTINGS.http_request_timeout));
                client = if let Some(proxy) = proxy {
                    client.proxy(proxy)
                } else {
                    client
                };
                let client = client.build()?;

                let resp = client.post("https://translate-pa.googleapis.com/v1/translateHtml").version(Version::HTTP_11).body(req_body).send().await?.text().await?;

                dprintln!("{}", resp);
                Ok(resp)
            }
            None => {
                Err(anyhow!("No api key found. Please check settings.json : \"google_translate_api_key\""))
            }
        }
        
    });

    match result {
        Ok(json_data) => {
            let value: Value = serde_json::from_str(json_data.as_str())?;
            let mut src_lng_suggested = src_lang.clone();
            if let Some(items) = value.as_array() {
                if items.get(0).is_some() && let Some(tr_items) = items[0].as_array() {
                    for item_value in tr_items {
                        if let Some(text) = item_value.as_str() {
                            response.push_str(text);
                            response.push_str("\n");
                        }
                    };

                    if let Some(arr1) = items.get(1) && let Some(lang) = arr1.get(0) {
                        src_lng_suggested = Lang::from_str(lang.as_str().unwrap_or("auto")).unwrap_or(src_lang);
                    }
                } else if items.get(0).is_some() && items.get(1).is_some() 
                && let Some(error) = items[0].as_i64() 
                && error == 3 {
                    if let Some(error_txt) = items[1].as_str() {
                        return Err(anyhow!(error_txt.to_string()));
                    } else {
                        return Err(anyhow!("error"));
                    }
                }
            } 

            if response.chars().count() > 1 {
                //TODO!: parse html entities
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
