//use serde_json::Value;
use crate::types::{AppEvent, Dictionary, Lang, UIStateDict};
use ureq::Agent;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time::Duration};
use anyhow::Result;
use super::GLOBAL_SETTINGS;

pub struct WDEn {
    is_running: Arc<AtomicBool>,
    app_sender: fltk::app::Sender<AppEvent>,
}



impl WDEn {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>) -> Self {
        let is_running = Arc::new(AtomicBool::new(false));
        Self {is_running, app_sender}
    }
}
impl Dictionary for WDEn {
    fn terminate(&mut self) {}

    fn get_uid(&self) -> String {
        //self.uid.clone();
        "dict_wiktionary_en".to_string()
    }
    fn get_name(&self) -> String {
        "English Wiktionary".to_string()
    }

    fn translate(&mut self, src_id: i64, text: String, src_lang: Lang, target_lang: Lang, is_fav: bool) {

        if !self.is_running.load(Ordering::SeqCst) {
            thread::spawn({
                let app_sender = self.app_sender;
                let is_running = Arc::clone(&self.is_running);
                let name = self.get_name();
                move || {
                    is_running.store(true, Ordering::SeqCst);

                    let transl_result = send_tr_request(text.clone(), src_lang.clone(), target_lang.clone());
                    match transl_result {
                        Ok(t_text) => {
                            app_sender.send(AppEvent::SaveDictEntry((src_id, text.clone(), "dict_wiktionary_en".to_string(), t_text.clone())));

                            app_sender.send(AppEvent::UpdateUiDict(UIStateDict {
                                src_id: Some(src_id),
                                src_text_dict: Some(text.clone()),
                                dict_uid: Some("dict_wiktionary_en".to_string()), 
                                dict_name: Some(name), 
                                //src: src_lang.clone(), 
                                //target: target_lang.clone(), 
                                dict_text: Some(t_text),
                                is_fav: Some(is_fav)
                            }));

                        }
                        Err(_e) => {
                            app_sender.send(AppEvent::SetStatus("translation failed (https req error)".into(), true, true));
                        }
                    }
                    thread::sleep(Duration::from_millis((GLOBAL_SETTINGS.http_throttling * 1000.0) as u64));
                    is_running.store(false, Ordering::SeqCst);
                }
            });
        } else {
            self.app_sender.send(AppEvent::SetStatus("error: rate limit".into(), true, true));
        }
    }
}

#[allow(unused_variables)]
fn send_tr_request(selected_text: String, src_lang: Lang, target_lang: Lang) -> Result<String> {
    let mut response = "".to_string();

    let req_string = "https://en.wiktionary.org/w/index.php?action=raw".to_string();
    println!("{}", req_string);
    let config = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(GLOBAL_SETTINGS.http_request_timeout)))
        .build();
    let agent: Agent = config.into();
    let json_data: String = agent.get(req_string)
        .query("title", selected_text.to_lowercase())
        .call()?
        .body_mut()
        .read_to_string()?;

    response.push_str(json_data.as_str());
    Ok(response)
    /*let value: Value = serde_json::from_str(json_data.as_str())?;

    if let Some(items) = value.as_array()
        && let Some(tr_items) = items[0].as_array() {
            for item_value in tr_items {
                if let Some(text) = item_value[0].as_str() {
                    response.push_str(text);
                    println!("{}", text);
                }
            }
        }
    Ok(response)*/
}
