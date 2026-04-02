//use serde_json::Value;
use crate::types::{AppEvent, Dictionary, Lang, UIStateDict};
//use ureq::Agent;
use wreq::{
    Client,
    Version
};
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time::Duration};
use anyhow::{anyhow, Result};
use super::GLOBAL_SETTINGS;
use super::TOKIO_RT;

pub struct WDEn {
    is_running: Arc<AtomicBool>,
    app_sender: fltk::app::Sender<AppEvent>,
    name: String,
    uid: String
}



impl WDEn {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>, name: String, uid: String) -> Self {
        let is_running = Arc::new(AtomicBool::new(false));
        //let uid = "dict_wiktionary_en".to_string();
        Self {is_running, app_sender, name, uid}
    }
}
impl Dictionary for WDEn {
    fn terminate(&mut self) {}

    fn get_uid(&self) -> String {
        self.uid.clone()
    }
    fn get_name(&self) -> String {
        //"English Wiktionary".to_string()
        self.name.clone()
    }

    fn translate(&mut self, src_id: i64, text: String, src_lang: Lang, target_lang: Lang) {

        if !self.is_running.load(Ordering::SeqCst) {
            thread::spawn({
                let app_sender = self.app_sender;
                let is_running = Arc::clone(&self.is_running);
                let name = self.get_name();
                let uid = self.get_uid();
                move || {
                    is_running.store(true, Ordering::SeqCst);

                    let transl_result = send_tr_request(text.clone(), src_lang.clone(), target_lang.clone());
                    match transl_result {
                        Ok(t_text) => {
                            app_sender.send(AppEvent::SaveDictEntry((src_id, text.clone(), uid.clone(), t_text.clone())));

                            app_sender.send(AppEvent::UpdateUiDict(UIStateDict {
                                src_id: Some(src_id),
                                src_text_dict: text.clone(),
                                dict_uid: Some(uid), 
                                dict_name: Some(name), 
                                //src: src_lang.clone(), 
                                //target: target_lang.clone(), 
                                dict_text: Some(t_text),
                                is_fav: None
                            }, false));

                        }
                        Err(e) => {
                            app_sender.send(AppEvent::SetReady(Some(e.to_string()), true));
                            let error_str = format!(r"Error: {e}");
                            //app_sender.send(AppEvent::SetStatus(error_str.into(), true, true));
                            //TODO?: if Err(Error::StatusCode(404)) --> SaveDictEntry("not found")
                        }
                    }
                    thread::sleep(Duration::from_millis((GLOBAL_SETTINGS.http_throttling * 1000.0) as u64));
                    is_running.store(false, Ordering::SeqCst);
                }
            });
        } else {
            self.app_sender.send(AppEvent::SetReady(Some("error: rate limit".to_string()), true));
            //self.app_sender.send(AppEvent::SetStatus("error: rate limit".into(), true, true));
        }
    }
}

#[allow(unused_variables)]
fn send_tr_request(selected_text: String, src_lang: Lang, target_lang: Lang) -> Result<String> {
    //let mut response = "".to_string();

    let req_string = "https://en.wiktionary.org/w/index.php?action=raw".to_string();
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
        let resp = client.get(req_string).query(&[("title", selected_text.to_lowercase())]).send().await?.text().await?;
        //println!("{}", resp);
        Ok(resp)
    });

    match result {
        Ok(r) => {
            //response.push_str(r.as_str());
            if r.chars().count() > 1 {
                Ok(r)
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
        .query("title", selected_text.to_lowercase())
        .call()?
        .body_mut()
        .read_to_string()?;

    response.push_str(json_data.as_str());
    Ok(response)*/
}
