use debug_print::{debug_println as dprintln};
//use serde_json::Value;
use crate::types::{AppEvent, Dictionary, Lang, UIStateDict, DictResult};
//use ureq::Agent;
use crate::utils::rt_request::{
    Client,
    //Version
};
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time::Duration};
use anyhow::{anyhow, Result};
use super::GLOBAL_SETTINGS;

pub struct WDEn {
    is_running: Arc<AtomicBool>,
    app_sender: fltk::app::Sender<AppEvent>,
    name: String,
    uid: String,
    use_proxy: bool,
    emulation: Option<String>
}



impl WDEn {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>, name: String, uid: String, use_proxy: bool, emulation: Option<String>) -> Self {
        let is_running = Arc::new(AtomicBool::new(false));
        //let uid = "dict_wiktionary_en".to_string();
        Self {is_running, app_sender, name, uid, use_proxy, emulation}
    }
}
impl Dictionary for WDEn {
    fn terminate(&mut self) {}

    fn get_uid(&self) -> &str {
        &self.uid
    }
    fn get_name(&self) -> &str {
        &self.name
    }

    fn translate(&mut self, src_id: i64, text: String, _src_lang: Lang,_target_langg: Lang) {

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

                    let transl_result = send_tr_request(text.clone(), use_proxy, emulation);
                    match transl_result {
                        Ok(t_text) => {
                            //app_sender.send(AppEvent::SaveDictEntry((src_id, text.clone(), uid.clone(), t_text.clone(), None, None)));
                            app_sender.send(AppEvent::SaveDictEntry(DictResult {
                                src_id,
                                dict_uid: uid.clone(),
                                text: t_text.clone(),
                                src: None, 
                                target: None
                            }));

                            app_sender.send(AppEvent::UpdateUiDict(UIStateDict {
                                src_id: Some(src_id),
                                src_text_dict: text.clone(),
                                dict_uid: Some(uid), 
                                dict_name: Some(name), 
                                src: None, 
                                target: None, 
                                dict_text: Some(t_text),
                                is_fav: None
                            }, false));

                        }
                        Err(e) => {
                            app_sender.send(AppEvent::SetReady(Some(e.to_string()), true));
                            //let error_str = format!(r"Error: {e}");
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
fn send_tr_request(selected_text: String, proxy: bool, emulation: Option<String>) -> Result<String> {
    //let mut response = "".to_string();

    let req_string = "https://en.wiktionary.org/w/index.php?action=raw".to_string();
    dprintln!("{}", req_string);

    let mut headers = std::collections::HashMap::new();
    headers.insert("User-Agent".into(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.104 Safari/537.36".into());

    let mut client = Client::builder()
        //.emulation(Emulation::Chrome137)
        .timeout(Duration::from_secs(GLOBAL_SETTINGS.http_request_timeout))
        .default_headers(headers)
        .proxy(proxy);
    if let Some(e) = emulation {
        client = client.emulation(e);
    }
    let client = client.build()?;
    
    let resp = client.get(req_string).query([("title", selected_text.to_lowercase())]).send()?.text()?;
    //dprintln!("{}", resp);
    let result = Ok(resp);


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
