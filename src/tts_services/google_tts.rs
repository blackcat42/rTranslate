//use debug_print::{debug_println as dprintln};
#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_return)]

use crate::types::{AppEvent, TTService};
use crate::settings::TTServiceOption;
use std::env;
use std::io::Write;
use std::fs::File;
use super::GLOBAL_SETTINGS;
use std::time::Duration;
use crate::utils::rt_request;
use std::sync::{Arc };
use std::sync::atomic::{AtomicBool, Ordering};

use crate::utils::helpers::google_tk;

#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub struct GTTS {
    is_running: Arc<AtomicBool>, 
    s: fltk::app::Sender<AppEvent>,
    options: TTServiceOption
}

use anyhow::{anyhow, Result};

impl GTTS {
    pub fn new(
        s: fltk::app::Sender<AppEvent>, 
        options: TTServiceOption
    ) -> Result<Self> {
        let is_running = Arc::new(AtomicBool::new(false));
        Ok(Self { 
            is_running, 
            s,  
            options
        })
    }
}

impl TTService for GTTS {
    fn get_name(&self) -> &str {
        &self.options.name
    }
    fn generate(&self, text: String, src_id: i64, voice: String) -> Result<()> {
        if self.is_running.load(Ordering::Relaxed) {
            self.s.send(AppEvent::Message("tts service is still running".into()));
            return Err(anyhow!("tts service is still running"));
        }
        
        let s = self.s;
        let engine_uid = self.options.uid.clone();

        let tk = if let Ok(token) = google_tk(&text, "") {
            format!("&tk={}", token)
        } else {
            "".to_string()
        };
        let url = format!("https://translate.google.com/translate_tts?ie=UTF-8&tl={}&client=gtx{}", "en", tk);
        //TODO: LNG DETECT!!! (from db)

        let working_dir = env::current_dir().unwrap();
        let filename = format!("{src_id}_{engine_uid}.mp3");
        let audio_path = format!(r"tts_cache\{filename}");
        let audio_path_full = working_dir.join(&audio_path);


        
        let mut headers = std::collections::HashMap::new();
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
            .get(url)
            .expect_binary(true)
            .query([("q", text)]);

        let is_running = Arc::clone(&self.is_running);
        dbg!(&client);
        std::thread::spawn(move || {
            is_running.store(true, Ordering::Relaxed);

            let audio_resp = client.send();

            let res: Result<()> = (|| {match audio_resp {
                Ok(audio_resp) => {
                    dbg!(&audio_resp.status());
                    if audio_resp.status().is_success() {
                        let audio_bytes = audio_resp.bytes()?;
                        let mut file = File::create(&audio_path_full)?;
                        file.write_all(&audio_bytes)?;
                        s.send(AppEvent::TTSave(src_id, engine_uid, voice, filename));
                        return Ok(());
                    } else {
                        let code = audio_resp.status().to_u16();
                        return Err(anyhow!(code.to_string()));
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
