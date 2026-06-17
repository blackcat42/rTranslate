use debug_print::{debug_println as dprintln};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

pub struct GD {
    is_running: Arc<AtomicBool>,
    app_sender: fltk::app::Sender<AppEvent>,
    name: String,
    uid: String,
    use_proxy: bool
}
use std::str::FromStr;


impl GD {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>, name: String, uid: String, use_proxy: bool) -> Self {
        let is_running = Arc::new(AtomicBool::new(false));
        Self {is_running, app_sender, name, uid, use_proxy}
    }
}
impl Dictionary for GD {
    fn terminate(&mut self) {}

    fn get_uid(&self) -> String {
        self.uid.clone()
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn translate(&mut self, src_id: i64, text: String, mut src_lang: Lang, target_lang: Lang) {

        if !self.is_running.load(Ordering::SeqCst) {
            thread::spawn({
                let app_sender = self.app_sender;
                let is_running = Arc::clone(&self.is_running);
                let name = self.get_name();
                let uid = self.get_uid();
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
                    let transl_result = send_tr_request(text.clone(), src_lang.clone(), target_lang.clone(), proxy);
                    match transl_result {
                        Ok(t_text) => {
                            let tt_text = parse_resp_json_to_dsl(t_text.clone());
                            if let Ok(text_d) = tt_text {
                                //dprintln!("lng: {}", text_d.1.unwrap_or("".to_string()));
                                if let Some(lng) = text_d.1 && let Ok(detected_lng) = Lang::from_str(&lng) {
                                    src_lang = detected_lng;
                                }
                                app_sender.send(AppEvent::SaveDictEntry((src_id, t_text.clone(), uid.clone(), text_d.0.clone(), Some(src_lang.clone()), Some(target_lang.clone()))));

                                app_sender.send(AppEvent::UpdateUiDict(UIStateDict {
                                    src_id: Some(src_id),
                                    src_text_dict: text.clone(),
                                    dict_uid: Some(uid), 
                                    dict_name: Some(name), 
                                    src: Some(src_lang), 
                                    target: Some(target_lang),
                                    dict_text: Some(text_d.0),
                                    is_fav: None
                                }, false));
                                //app_sender.send(AppEvent::SetReady(None, true));
                            } else {
                                app_sender.send(AppEvent::SetReady(Some("failed to parse response json".to_string()), true));
                            }
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
fn send_tr_request(selected_text: String, src_lang: Lang, target_lang: Lang, proxy: Option<wreq::Proxy>) -> Result<String> {
    //let mut response = "".to_string();
    let src_lang = src_lang.as_ref();
    let target_lang = target_lang.as_ref();

    let token: Option<String> = None;

    let mut req_string = format!("https://translate.googleapis.com/translate_a/single?client=gtx&sl={src_lang}&tl={target_lang}&hl={target_lang}");
    //&dt=bd&dt=t&dt=ld&dt=rm&ie=UTF-8&oe=UTF-8");

    req_string.push_str("&dt=t"); //translation
    req_string.push_str("&dt=at"); //alternate translations
    req_string.push_str("&dt=bd"); //dictionary
    req_string.push_str("&dt=md"); //definitions
    req_string.push_str("&dt=ex"); //examples
    req_string.push_str("&dt=ld"); //???
    
    
    req_string.push_str("&dt=rw"); //see also list ???
    req_string.push_str("&dt=rm"); //transcription / transliteration
    req_string.push_str("&dt=ss"); //synonyms
    
    req_string.push_str("&dj=1&ie=UTF-8&oe=UTF-8"); //&dj=1 - json response with names
    //&dt=qca ???

    if let Some(token) = token {
        req_string.push_str("&tk=");
        req_string.push_str(&token);
    }

    dprintln!("{}", req_string);

    let rt = TOKIO_RT.get_or_init(|| {
        tokio::runtime::Runtime::new().expect("Tokio Runtime Error")
    });

    let result = rt.block_on(async {
        let mut client = Client::builder()
            //.emulation(Emulation::Chrome137)
            .timeout(Duration::from_secs(GLOBAL_SETTINGS.http_request_timeout))
            .user_agent("User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.104 Safari/537.36");
        client = if let Some(proxy) = proxy {
            client.proxy(proxy)
        } else {
            client
        };
        let client = client.build()?;
        
        let resp = client.get(req_string).query(&[("q", selected_text.to_lowercase())]).send().await?.text().await?;
        Ok(resp)
    });

    match result {
        Ok(r) => {
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





fn parse_resp_json_to_dsl(json_text: String) -> Result<(String, Option<String>)> {
    let json_data: Value = serde_json::from_str(json_text.as_str())?;
    let mut response = String::new();
    let mut src_lng_suggested = None;

    if let Some(sentences) = json_data.pointer("/sentences") {
        if let Some(sentences_arr) = sentences.as_array() {
            if let Some(arr_first_sentence) = sentences_arr.get(0) 
            && let Some(sentence_obj) = arr_first_sentence.as_object() {
                if let Some(trans) = sentence_obj.get("trans")
                && let Some(orig) = sentence_obj.get("orig")
                && let Some(trans_str) = trans.as_str()
                && let Some(orig_str) = orig.as_str() {
                    response.push_str(&format!("[b]{orig_str}[/b] / {trans_str}"));
                }
            }

            if let Some(arr_second_sentence) = sentences_arr.get(1) 
            && let Some(sentence_obj) = arr_second_sentence.as_object() {
                if let Some(src_translit) = sentence_obj.get("src_translit")
                && let Some(src_translit_str) = src_translit.as_str() {
                    response.push_str(&format!(" [i]({src_translit_str})[/i]"));
                }
            }
        }
    };

    if let Some(src) = json_data.pointer("/src") {
        if let Some(src_str) = src.as_str() {
            src_lng_suggested = Some(src_str.to_string());
        }
    };

    if let Some(dict) = json_data.pointer("/dict") {
        if let Some(dict_arr) = dict.as_array() {
            response.push_str("\n\n[c indigo]DICTIONARY[/c]");
            for item_value in dict_arr {
                if let Some(dict_obj) = item_value.as_object() {
                    if let Some(pos) = dict_obj.get("pos")
                    && let Some(pos_str) = pos.as_str() { 
                        response.push_str("\n[c teal]");
                        response.push_str(pos_str);
                        response.push_str("[/c]");
                        //response.push('\n');
                        //response.push_str("  ");
                    }
                    if let Some(entry) = dict_obj.get("entry")
                    && let Some(entry_arr) = entry.as_array() { 
                        for (i, entry_value) in entry_arr.iter().enumerate() {
                            if let Some(entry_obj) = entry_value.as_object()
                            && let Some(word) = entry_obj.get("word") 
                            && let Some(word_str) = word.as_str() {
                                response.push('\n');
                                response.push_str(&format!("  {}) {}", i + 1, word_str));
                                if let Some(reverse_translation) = entry_obj.get("reverse_translation")
                                && let Some(rt_arr) = reverse_translation.as_array() {
                                    response.push_str(" [c blue](");
                                    let len = rt_arr.len();
                                    for (i, rt_value) in rt_arr.iter().enumerate() {
                                        if let Some(rt_str) = rt_value.as_str() {
                                            response.push_str(rt_str);
                                            if i != len - 1 {
                                                response.push_str(", ");
                                            }
                                        }
                                    }
                                    response.push_str(")[/c]");
                                }
                            }
                        }
                    }
                }
                /*if item_value.get(0).is_some() && let Some(text) = item_value[0].as_str() {
                    response.push_str(text);
                    //dprintln!("{}", text);
                }*/
            }
        }
    };

    if let Some(synsets) = json_data.pointer("/synsets") {
        if let Some(synsets_arr) = synsets.as_array() {
            response.push_str("\n\n[c indigo]SYNONYM SETS[/c]");
            for item_value in synsets_arr {
                if let Some(dict_obj) = item_value.as_object() {
                    if let Some(pos) = dict_obj.get("pos")
                    && let Some(pos_str) = pos.as_str() { 
                        response.push('\n');
                        response.push_str("[c teal]");
                        response.push_str(pos_str);
                        response.push_str("[/c]");
                    }
                    if let Some(entry) = dict_obj.get("entry")
                    && let Some(entry_arr) = entry.as_array() { 
                        for (i, entry_value) in entry_arr.iter().enumerate() {
                            if let Some(entry_obj) = entry_value.as_object() {
                                response.push('\n');
                                response.push_str(&format!("  {}) ", i + 1));
                                if let Some(label_info) = entry_obj.get("label_info")
                                && let Some(label_info_obj) = label_info.as_object()
                                && let Some(register_value) = label_info_obj.get("register")
                                && let Some(register_arr) = register_value.as_array() {
                                    response.push_str("[c green]");
                                    let len = register_arr.len();
                                    for (i, register) in register_arr.iter().enumerate() {
                                        if let Some(r_str) = register.as_str() {
                                            response.push_str(r_str);
                                            if i != len - 1 {
                                                response.push_str(", ");
                                            }
                                        }
                                    }
                                    response.push_str(": ");
                                    response.push_str("[/c]");
                                }

                                if let Some(synonym) = entry_obj.get("synonym")
                                && let Some(synonym_arr) = synonym.as_array() {
                                    let len = synonym_arr.len();
                                    for (i, s_value) in synonym_arr.iter().enumerate() {
                                        if let Some(s_str) = s_value.as_str() {
                                            response.push_str(s_str);
                                            if i != len - 1 {
                                                response.push_str(", ");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                /*if item_value.get(0).is_some() && let Some(text) = item_value[0].as_str() {
                    response.push_str(text);
                    //dprintln!("{}", text);
                }*/
            }
        }
    };

    if let Some(definitions) = json_data.pointer("/definitions") {
        if let Some(definitions_arr) = definitions.as_array() {
            response.push_str("\n\n[c indigo]DEFINITIONS[/c]");
            for item_value in definitions_arr {
                if let Some(definition_obj) = item_value.as_object() {
                    if let Some(pos) = definition_obj.get("pos")
                    && let Some(pos_str) = pos.as_str() { 
                        response.push_str("\n[c teal]");
                        response.push_str(pos_str);
                        response.push_str("[/c]");
                    }
                    if let Some(entry) = definition_obj.get("entry")
                    && let Some(entry_arr) = entry.as_array() { 
                        for (i, entry_value) in entry_arr.iter().enumerate() {
                            if let Some(entry_obj) = entry_value.as_object()
                            && let Some(gloss) = entry_obj.get("gloss")
                            && let Some(gloss_str) = gloss.as_str() {
                                response.push_str(&format!("\n  {}) ", i + 1));

                                if let Some(label_info) = entry_obj.get("label_info")
                                && let Some(label_info_obj) = label_info.as_object()
                                && let Some(register_value) = label_info_obj.get("register")
                                && let Some(register_arr) = register_value.as_array() {
                                    response.push_str("[c green]");
                                    let len = register_arr.len();
                                    for (i, register) in register_arr.iter().enumerate() {
                                        if let Some(r_str) = register.as_str() {
                                            response.push_str(r_str);
                                            if i != len - 1 {
                                                response.push_str(", ");
                                            }
                                        }
                                    }
                                    response.push_str(": ");
                                    response.push_str("[/c]");
                                }

                                response.push_str(gloss_str);
                                if let Some(example) = entry_obj.get("example")
                                && let Some(example_str) = example.as_str() {
                                    response.push_str(" (\"");
                                    response.push_str(example_str);
                                    response.push_str("\")");
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    if let Some(examples) = json_data.pointer("/examples/example") {
        if let Some(examples_arr) = examples.as_array() {
            response.push_str("\n\n[b][c indigo]EXAMPLES[/c][/b]");
            for (i, item_value) in examples_arr.iter().enumerate() {
                if let Some(example_obj) = item_value.as_object() {
                    if let Some(text) = example_obj.get("text")
                    && let Some(text_str) = text.as_str() {
                        response.push_str(&format!("\n  {}) ", i + 1));
                        response.push('"');
                        response.push_str(text_str);
                        response.push('"');
                    }
                }
            }
        }
    };

    response.push('\n');
    let response = response.replace("<b>", "[b]").replace("</b>", "[/b]");

    Ok((response, src_lng_suggested))
     /*if let Some(lng) = json_data.pointer("/result/lang") && let Some(lng_str) = lng.as_str()  {
        src_lng_suggested = Some(lng_str.to_lowercase());
    }*/
}