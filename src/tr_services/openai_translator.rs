#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_return)]

use debug_print::{debug_println as dprintln};
use crate::types::{AppEvent, Translator, Lang, UIState, TranslResult};
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time::Duration};
use std::io::{BufRead, BufReader};
use anyhow::Result;
use super::GLOBAL_SETTINGS;
use crate::utils::helpers::extract_detected_lang;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use crate::utils::rt_request;


pub struct OA {
    is_running: Arc<AtomicBool>,
    is_processing: Arc<AtomicBool>,
    app_sender: fltk::app::Sender<AppEvent>,
    name: String,
    uid: String,
    use_proxy: bool,
    emulation: Option<String>,
    kill_sender: Option<std::sync::mpsc::Sender<()>>,
    base_url: String, 
    api_key: String, 
    model: String, 
    prompt: String,
    stream: bool,
    api_key_requied: bool, 
    api_key_url: String
}

impl OA {
    pub fn new(
        app_sender: fltk::app::Sender<AppEvent>, 
        name: String, 
        uid: String, 
        use_proxy: bool, 
        emulation: Option<String>, 
        base_url: String, 
        api_key: String, 
        model: String, 
        prompt: String, 
        stream: bool, 
        api_key_requied: bool, 
        api_key_url: String
    ) -> Self {
        let is_running = Arc::new(AtomicBool::new(false));
        let is_processing = Arc::new(AtomicBool::new(false));
        Self {
            is_running, 
            is_processing, 
            app_sender, 
            name, 
            uid, 
            use_proxy, 
            emulation, 
            kill_sender: None, 
            base_url, 
            api_key, 
            model, 
            prompt, 
            stream, 
            api_key_requied, 
            api_key_url
        }
    }
}
impl Translator for OA {
    fn terminate(&mut self) {
        if let Some(s) = self.kill_sender.take() {
            let _ = s.send(());
        }
    }
    fn is_processing(&self) -> bool {
        self.is_processing.load(Ordering::SeqCst)
    }
    fn get_uid(&self) -> &str {
        &self.uid
    }
    fn get_name(&self) -> &str {
        &self.name
    }

    fn translate(&mut self, src_id: i64, text: String, src_lang: Lang, target_lang: Lang, is_lang_detected: bool) {

        if !self.is_running.load(Ordering::SeqCst) {
            if self.api_key.is_empty() && self.api_key_requied {
                let msg = format!("{} service requires an API key. Get one at {}", self.name, self.api_key_url);
                self.app_sender.send(AppEvent::Message(msg.clone().into()));
                self.app_sender.send(AppEvent::SetReady(Some(msg), false));
                return;
            }

            let (kill_tx, kill_rx) = std::sync::mpsc::channel();
            self.kill_sender = Some(kill_tx);

            thread::spawn({
                let app_sender = self.app_sender;
                let is_running = Arc::clone(&self.is_running);
                let is_processing = Arc::clone(&self.is_processing);
                let name = self.get_name().to_string();
                let uid = self.get_uid().to_string();
                let use_proxy = self.use_proxy;
                let emulation = self.emulation.clone();

                let base_url = self.base_url.clone();
                let api_key = self.api_key.clone();
                let model = self.model.clone();
                let prompt = self.prompt.clone();
                let stream = self.stream;

                move || {
                    is_running.store(true, Ordering::SeqCst);
                    is_processing.store(true, Ordering::SeqCst);
                    let transl_result = send_tr_request(
                        app_sender, 
                        kill_rx, 
                        text.clone(), 
                        src_lang.clone(), 
                        target_lang.clone(), 
                        is_lang_detected, 
                        use_proxy, 
                        emulation, 
                        &base_url, 
                        &api_key, 
                        &model, 
                        &prompt
                    );
                    is_processing.store(false, Ordering::SeqCst);
                    match transl_result {
                        Ok(t_text) => {
                             app_sender.send(AppEvent::SaveTranslation(TranslResult {
                                src_id, 
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
                        }
                    }
                    thread::sleep(Duration::from_millis((GLOBAL_SETTINGS.http_throttling * 1000.0) as u64));
                    is_running.store(false, Ordering::SeqCst);
                }
            });
        } else {
            self.app_sender.send(AppEvent::SetReady(Some("error: rate limit".to_string()), false));
        }
    }
}



#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Deserialize, Debug)]
struct ChatChunk {
    choices: Vec<ChunkChoice>,
}

#[derive(Deserialize, Debug)]
struct ChunkChoice {
    delta: ChunkDelta,
}

#[derive(Deserialize, Debug)]
struct ChunkDelta {
    content: Option<String>,
}


fn send_tr_request(
    app_sender: fltk::app::Sender<AppEvent>, 
    kill_rec: std::sync::mpsc::Receiver<()>, 
    selected_text: String, 
    src_lang: Lang, 
    target_lang: Lang, 
    is_lang_detected: bool, 
    proxy: bool, 
    emulation: Option<String>, 
    base_url: &str, 
    api_key: &str, 
    model: &str, 
    prompt: &str
) -> Result<(String, Lang)> {

    let src_lang_ref = if is_lang_detected {
        src_lang.as_ref()
    } else {
        "auto"
    };
    app_sender.send(AppEvent::SetWaitingWithStream(false));

    //let src_lang_ref = serde_json::to_string(src_lang_ref)?;
    //let target_lang = serde_json::to_string(target_lang.as_ref())?;

    let prompt = prompt
        .replace("\r\n", "\n")
        .replace("<RT_TARGET_LANG>", target_lang.name())
        .replace("<RT_TEXT>", &selected_text);

    dprintln!("{}", &prompt);

    let body = ChatRequest {
        model: model.to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt,
        }],
        stream: true,
    };
    let body = serde_json::to_string(&body)?;


    let mut headers = std::collections::HashMap::new();
    headers.insert("Authorization".into(), format!("Bearer {}", api_key));
    headers.insert("Content-Type".into(), "application/json".into());
    headers.insert("Accept".into(), "text/event-stream".into());
    headers.insert("User-Agent".into(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.104 Safari/537.36".into());

    let mut client = rt_request::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(GLOBAL_SETTINGS.openai_api_stream_request_timeout))
        .proxy(proxy);
    if let Some(e) = emulation {
        client = client.emulation(e);
    }
    let client = client.build()?;

    let reader = client.post(format!("{}/chat/completions", base_url)).body(&body).expect_raw(true).send()?;
    let reader = BufReader::new(reader.raw()?.into_body().into_reader());
    let mut result: String = "".to_string();

    for line in reader.lines() {
        if kill_rec.try_recv().is_ok() {
            dprintln!("Kill signal received");
            break;
        }

        let line = line?;
        if line.starts_with("data: ") {
            let data_str = line.trim_start_matches("data: ").trim();
            if data_str == "[DONE]" {
                break;
            }
            if data_str.is_empty() {
                continue;
            }

            if let Ok(chunk) = serde_json::from_str::<ChatChunk>(data_str) 
            && let Some(choice) = chunk.choices.first() 
            && let Some(content) = &choice.delta.content {
                result.push_str(content);
                app_sender.send(AppEvent::AppendToStreamBuf(content.to_string()));                
            }
        }
    }

    dprintln!("{}", result);
    let (detected_lang, result) = extract_detected_lang(result);
    let src_lang = if let Some(lang) = detected_lang && src_lang_ref == "auto" {
        dprintln!("DETECTED_LANG tag found");
        Lang::from_str(&lang).unwrap_or(src_lang)
    } else {
        src_lang
    };
    Ok((result, src_lang))
}
