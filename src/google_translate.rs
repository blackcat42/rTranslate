use serde_json::Value;
use crate::types::{AppEvent, Translator, Lang, UIState};
//use ureq::Agent;
use wreq::{
    Client,
    Version
};
/*use wreq_util::{
    Emulation
};*/
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time::Duration};
use anyhow::{anyhow, Result};
use super::GLOBAL_SETTINGS;
use super::TOKIO_RT;

pub struct GT {
    is_running: Arc<AtomicBool>,
    app_sender: fltk::app::Sender<AppEvent>,
    name: String,
    uid: String
}

impl GT {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>, name: String, uid: String) -> Self {
        let is_running = Arc::new(AtomicBool::new(false));
        //let uid = "tr_google".to_string();
        Self {is_running, app_sender, name, uid}
    }
}
impl Translator for GT {
    fn terminate(&mut self) {
        
    }
    fn get_uid(&self) -> String {
        //self.uid.clone();
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
                                src_text: text.clone(),
                                tr_uid: Some(uid), 
                                translator: Some(name), 
                                src: Some(src_lang), 
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


fn send_tr_request(selected_text: String, src_lang: Lang, target_lang: Lang, is_lang_detected: bool) -> Result<(String, Option<String>)> {
    let mut response = "".to_string();

    let src_lang = if is_lang_detected {
        src_lang.as_ref()
    } else {
        "auto"
    };
    let req_string = format!("https://translate.googleapis.com/translate_a/single?client=gtx&sl={}&dt=t&tl={}", src_lang, target_lang.as_ref());
    println!("{}", req_string);

    let rt = TOKIO_RT.get_or_init(|| {
        tokio::runtime::Runtime::new().expect("Tokio Runtime Error")
    });

    let result = rt.block_on(async {
        let client = Client::builder()
            //.emulation(Emulation::Chrome137)
            .timeout(Duration::from_secs(GLOBAL_SETTINGS.http_request_timeout))
            .user_agent("User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.104 Safari/537.36")
            .build()?;
        let resp = client.get(req_string).query(&[("q", selected_text)]).send().await?.text().await?;
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
                        if item_value.get(0).is_some() && let Some(text) = item_value[0].as_str() {
                            response.push_str(text);
                            //println!("{}", text);
                        }
                    };
                    if let Some(lang) = items.get(2) {
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
    /*let config = Agent::config_builder()
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
    Ok(response)*/
}
