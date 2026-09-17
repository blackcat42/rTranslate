#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_return)]

use debug_print::{debug_println as dprintln};
use crate::types::{AppEvent, TTService, TTServiceOption};
use std::env;
use std::io::Write;
use serde::Serialize;
use std::fs::File;
use super::GLOBAL_SETTINGS;
use std::{time::Duration};
use crate::utils::rt_request;
use std::sync::{Arc };
use std::sync::atomic::{AtomicBool, Ordering};

#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub struct OATTS {
    is_running: Arc<AtomicBool>, 
    s: fltk::app::Sender<AppEvent>,
    base_url: String,
    model: String,
    response_format: String,
    options: TTServiceOption
}

use anyhow::{anyhow, Result};


impl OATTS {
    pub fn new(
        s: fltk::app::Sender<AppEvent>, 
        options: TTServiceOption
    ) -> Result<Self> {
        let response_format = if let Some(f) = &options.openai_response_format {
            f
        } else {
            "mp3"
        };

        if let Some(base_url) = &options.openai_url 
        && let Some(model) = &options.openai_model {
            let is_running = Arc::new(AtomicBool::new(false));
            Ok(Self { 
                is_running,
                s,
                base_url: base_url.clone(),
                model: model.clone(),
                response_format: response_format.to_string(),
                options
            })
        } else {
            Err(anyhow!("error: "))
        }
        
    }
}


#[derive(Serialize)]
struct TTSRequest {
    model: String,
    voice: String,
    input: String,
    response_format: String,
}

impl TTService for OATTS {
    fn get_name(&self) -> &str {
        &self.options.name
    }
    fn generate(&self, text: String, src_id: i64, voice: String) -> Result<()> {
        if self.is_running.load(Ordering::Relaxed) {
            self.s.send(AppEvent::Message("tts service is still running".into()));
            //self.s.send(AppEvent::SetStatus("error: tts service is still running".into(), false, false));
            return Err(anyhow!("tts service is still running"));
        }

        if self.options.openai_api_key.is_empty() && self.options.api_key_requied {
            let msg = format!("{} service requires an API key. Get one at {}", self.options.name, self.options.api_key_url);
            self.s.send(AppEvent::Message(msg.clone().into()));
            self.s.send(AppEvent::SetReady(None, false));
            return Err(anyhow!(msg));
        }
        
        let s = self.s;
        let engine_uid = self.options.uid.clone();

        let body = TTSRequest {
            model: self.model.clone(),
            voice: voice.clone(),
            input: text,
            response_format: self.response_format.clone(),
        };
        let body = serde_json::to_string(&body)?;


        let working_dir = env::current_dir().unwrap();
        let filename = format!("{src_id}_{engine_uid}_{voice}.{}", &self.response_format);
        let audio_path = format!(r"tts_cache\{filename}");
        let audio_path_full = working_dir.join(&audio_path);


        

        let mut headers = std::collections::HashMap::new();
        headers.insert("Authorization".into(), format!("Bearer {}", self.options.openai_api_key));
        headers.insert("Content-Type".into(), "application/json".into());
        headers.insert("User-Agent".into(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.104 Safari/537.36".into());

        let mut client = rt_request::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(GLOBAL_SETTINGS.openai_api_request_timeout))
            .gzip(true)
            .proxy(self.options.use_proxy);
        if let Some(e) = &self.options.emulation {
            client = client.emulation(e);
        }
        let client = client.build()?;

        let client = client
            .post(format!("{}/audio/speech", self.base_url))
            .body(&body)
            .expect_binary(true);

        let is_running = Arc::clone(&self.is_running);
        dbg!(&client);
        std::thread::spawn(move || {
            is_running.store(true, Ordering::Relaxed);

            let audio_resp = client.send();

            let res: Result<()> = (|| {match audio_resp {
                Ok(audio_resp) => {
                    if let Some(ref hdrs) = audio_resp.headers {
                        for (key, val) in hdrs {
                            dprintln!("{}: {}", key, val);
                        }
                        if let Some(remaining) = hdrs.get("x-ratelimit-remaining") {
                            println!("remaining: {}", remaining);
                            
                        }
                        if let Some(reset) = hdrs.get("x-ratelimit-reset") {
                            println!("ratelimit-reset: {}", reset);
                        }
                    }
                    dbg!(&audio_resp.status());
                    if audio_resp.status().is_success() {
                        let audio_bytes = audio_resp.bytes()?;
                        let mut file = File::create(&audio_path_full)?;
                        file.write_all(&audio_bytes)?;
                        s.send(AppEvent::TTSave(src_id, engine_uid, voice, filename));
                        return Ok(());
                    } else {
                        let code = audio_resp.status().to_u16();
                        return Err(anyhow!(code.to_string())) 
                        //return Err(anyhow!(audio_resp.status().to_string()));
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }})();

            is_running.store(false, Ordering::Relaxed);
            match res {
                Ok(_) => {},
                Err(e) => {
                    s.send(AppEvent::SetReady(Some(e.to_string()), false));
                }
            }
            //Ok(())
        });
        Ok(())
    }

}
