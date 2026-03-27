use serde_json::Value;
use crate::types::{AppEvent, Translator, Lang, UIState};
use ureq::Agent;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time::Duration};
use anyhow::Result;
use super::GLOBAL_SETTINGS;

pub struct GT {
    is_running: Arc<AtomicBool>,
    app_sender: fltk::app::Sender<AppEvent>,
    name: String
}

impl GT {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>, name: String) -> Self {
        let is_running = Arc::new(AtomicBool::new(false));
        Self {is_running, app_sender, name}
    }
}
impl Translator for GT {
    fn terminate(&mut self) {
        
    }
    fn get_uid(&self) -> String {
        //self.uid.clone();
        "tr_google".to_string()
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
                move || {
                    is_running.store(true, Ordering::SeqCst);

                    let transl_result = send_tr_request(text.clone(), src_lang.clone(), target_lang.clone(), is_lang_detected);
                    match transl_result {
                        Ok(t_text) => {
                            app_sender.send(AppEvent::SaveTranslation((src_id, text.clone(), "tr_google".to_string(), src_lang.clone(), target_lang.clone(), t_text.clone())));
                            app_sender.send(AppEvent::UpdateUi(UIState {
                                src_text: text.clone(),
                                tr_uid: Some("tr_google".to_string()), 
                                translator: Some(name), 
                                src: Some(src_lang.clone()), 
                                target: Some(target_lang.clone()), 
                                translation_text: Some(t_text),
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


fn send_tr_request(selected_text: String, src_lang: Lang, target_lang: Lang, is_lang_detected: bool) -> Result<String> {
    let mut response = "".to_string();

    let src_lang = if is_lang_detected {
        src_lang.as_ref()
    } else {
        "auto"
    };
    let req_string = format!("https://translate.googleapis.com/translate_a/single?client=gtx&sl={}&dt=t&tl={}", src_lang, target_lang.as_ref());
    println!("{}", req_string);
    let config = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(GLOBAL_SETTINGS.http_request_timeout)))
        .build();
    let agent: Agent = config.into();
    let json_data: String = agent.get(req_string)
        .query("q", selected_text)
        .call()?
        .body_mut()
        .read_to_string()?;

    let value: Value = serde_json::from_str(json_data.as_str())?;

    if let Some(items) = value.as_array()
        && let Some(tr_items) = items[0].as_array() {
            for item_value in tr_items {
                if let Some(text) = item_value[0].as_str() {
                    response.push_str(text);
                    //println!("{}", text);
                }
            }
        }
    Ok(response)
}
