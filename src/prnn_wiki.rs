use crate::types::{AppEvent, PRNNService};
use std::{thread, time::Duration};
use std::sync::{Arc };
use std::sync::atomic::{AtomicBool, Ordering};
use super::GLOBAL_SETTINGS;

//#[allow(dead_code)]
//#[allow(clippy::upper_case_acronyms)]
pub struct WP {
    is_running: Arc<AtomicBool>,
    app_sender: fltk::app::Sender<AppEvent>,
    name: String,
}

use anyhow::{Result};

impl WP {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>, name: String) -> Self {
        let is_running = Arc::new(AtomicBool::new(false));
        Self { is_running, app_sender, name}
    }
}

impl PRNNService for WP {
    fn get_name(&self) -> String {
        self.name.clone()
    }
    
    fn generate(&self, text: String, src_id: i64) -> Result<()> {
        if !self.is_running.load(Ordering::SeqCst) {
            thread::spawn({
                let app_sender = self.app_sender;
                let is_running = Arc::clone(&self.is_running);
                move || {
                    is_running.store(true, Ordering::SeqCst);

                    let transl_result = send_pr_request(text.clone(), src_id);
                    match transl_result {
                        Ok(t_text) => {
                            for item in t_text.iter() {
                                app_sender.send(AppEvent::SavePRNN((src_id, "prnn_wiki".to_string(), item.clone() )));
                                app_sender.send(AppEvent::TTSPlay(item.clone()));
                            }
                        }
                        Err(_e) => {
                            app_sender.send(AppEvent::Message("tts error".into()));
                            //app_sender.send(AppEvent::SetStatus("tts error".into(), false, true));
                        }
                    }
                    thread::sleep(Duration::from_millis((GLOBAL_SETTINGS.http_throttling * 1000.0) as u64));
                    is_running.store(false, Ordering::SeqCst);
                }
            });
        } else {
            self.app_sender.send(AppEvent::Message("tts error: rate limit".into()));
            //self.app_sender.send(AppEvent::SetStatus("error: rate limit".into(), false, true));
        }
        Ok(())
    }

}

fn send_pr_request(_selected_text: String, src_id: i64) -> Result<Vec<String>> {
    //let mut response = "".to_string();

    let mut arr: Vec<String> = vec![];
    let str1 = format!("{}_prnn_wiki_example", src_id);
    let str2 = format!("{}_prnn_wiki_example2", src_id);
    arr.push(str1);
    arr.push(str2);

    /*
    TODO
    "https://en.wiktionary.org/api/rest_v1/page/media-list/maybe"
    items.0.type == "audio"
    items.0.title == "File:En-uk-maybe.ogg"

    "https://en.wiktionary.org/w/api.php?action=query&format=json&prop=imageinfo&titles=File:en-us-maybe.ogg&iiprop=url"

    query.pages.-1.imageinfo.0.url == "https://upload.wikimedia.org/wikipedia/commons/4/4d/En-us-maybe.ogg"*/

    /*let req_string = format!("https://translate.googleapis.com/translate_a/single?client=gtx&sl={}&dt=t&tl={}", src_lang.as_ref(), target_lang.as_ref());
    //let req_string = "https://translate.googleapis.com/translate_a/single?client=gtx&sl=en&dt=t&tl=ru".to_string();
    println!("{}", req_string);
    let config = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
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
        }*/
    Ok(arr)
}