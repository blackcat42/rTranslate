use debug_print::{debug_println as dprintln};
use crate::types::{AppEvent, PRNNService, Lang};
use std::{thread, time::Duration};
use std::sync::{Arc };
use std::sync::atomic::{AtomicBool, Ordering};
use std::fs::File;
use std::io::Write;
use anyhow::{anyhow, Result};
//use serde_json::Value;
use wreq::{
    Client,
    //Version,
    header,
    StatusCode
};
//use wreq_util::{
//    Emulation
//};
use super::GLOBAL_SETTINGS;
use super::TOKIO_RT;
//use serde::{Deserialize, Serialize};

//#[allow(dead_code)]
//#[allow(clippy::upper_case_acronyms)]
pub struct GP {
    is_running: Arc<AtomicBool>,
    app_sender: fltk::app::Sender<AppEvent>,
    name: String,
    use_proxy: bool
}

impl GP {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>, name: String, use_proxy: bool) -> Self {
        let is_running = Arc::new(AtomicBool::new(false));
        Self { is_running, app_sender, name, use_proxy}
    }
}
//TODO! language detect
impl PRNNService for GP {
    fn get_name(&self) -> &str {
        &self.name
    }
    
    fn generate(&self, text: String, src_lang: Lang, src_id: i64) -> Result<()> {
        if !self.is_running.load(Ordering::SeqCst) {
            thread::spawn({
                let app_sender = self.app_sender;
                let is_running = Arc::clone(&self.is_running);
                let src_lang = src_lang.clone();
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
                    let filenames = send_pr_request(app_sender, text.clone(), src_lang.as_ref(), src_id, proxy);
                    match filenames {
                        Ok(p_files) => {
                            //dbg!(&p_file);
                            for item in p_files.iter() {
                                dbg!(&item);
                                app_sender.send(AppEvent::PRNNSave((src_id, "prnn_google".to_string(), item.clone() )));
                            }
                            if let Some (str1) = p_files.get(0) {
                                //dbg!(&str1);
                                app_sender.send(AppEvent::TTSPlay(str1.to_string()));
                            }
                        }
                        Err(e) => {
                            app_sender.send(AppEvent::Message(e.to_string().into()));
                            //app_sender.send(AppEvent::SetStatus("tts error".into(), false, true));
                        }
                    }
                    app_sender.send(AppEvent::SetReady(None, true));
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


fn send_pr_request(app_sender: fltk::app::Sender<AppEvent>, selected_text: String, src_lang: &str, _src_id: i64, proxy: Option<wreq::Proxy>) -> Result<Vec<String>> {

    let selected_text = selected_text.to_lowercase();
    let first_two_chars: String = selected_text.chars().take(2).collect();
    //let date1 = "2021-03-01";
    //let date2 = "2021-06-17";
    let date3 = "2024-04-19";
    //TODO: settings
    let mut arr_urls: Vec<String> = vec![];
    if src_lang == "en" {
        if GLOBAL_SETTINGS.download_all_pronunciations || GLOBAL_SETTINGS.eng_accents.contains(&String::from("us")) {
            arr_urls.push(format!("https://ssl.gstatic.com/dictionary/static/sounds/oxford/{selected_text}--_us_1.mp3"));
            arr_urls.push(format!("https://ssl.gstatic.com/dictionary/static/pronunciation/{date3}/audio/{first_two_chars}/{selected_text}_en_us_1.mp3"));
        }
        if GLOBAL_SETTINGS.download_all_pronunciations || GLOBAL_SETTINGS.eng_accents.contains(&String::from("gb")) {
            arr_urls.push(format!("https://ssl.gstatic.com/dictionary/static/sounds/oxford/{selected_text}--_gb_1.mp3"));
            arr_urls.push(format!("https://ssl.gstatic.com/dictionary/static/pronunciation/{date3}/audio/{first_two_chars}/{selected_text}_en_gb_1.mp3"));
        }
        if GLOBAL_SETTINGS.download_all_pronunciations || GLOBAL_SETTINGS.eng_accents.contains(&String::from("in")) {
            arr_urls.push(format!("https://ssl.gstatic.com/dictionary/static/pronunciation/{date3}/audio/{first_two_chars}/{selected_text}_en_in_1.mp3"));
        }
    } else {
        arr_urls.push(format!("https://ssl.gstatic.com/dictionary/static/pronunciation/{date3}/audio/{first_two_chars}/{selected_text}_{src_lang}_{src_lang}_1.mp3"));
    }


    let rt = TOKIO_RT.get_or_init(|| {
        tokio::runtime::Runtime::new().expect("Tokio Runtime Error")
    });

    let mut headers = header::HeaderMap::new();
    headers.insert("Accept-Encoding", header::HeaderValue::from_static("gzip"));
    //headers.insert("Host", header::HeaderValue::from_static("en.wiktionary.org"));
    headers.insert("User-Agent", header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36"));

    let result = rt.block_on(async {
        let mut arr_filenames: Vec<String> = vec![];
        let working_dir = std::env::current_dir()?;

        let mut client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(GLOBAL_SETTINGS.http_request_timeout))
            .gzip(true);
        client = if let Some(proxy) = proxy {
            client.proxy(proxy)
        } else {
            client
        };
        let client = client.build()?;

        let mut count = 1;
        let urls_len = arr_urls.len();
        for url in arr_urls {
            #[allow(clippy::double_ended_iterator_last)]
            let filename = url.trim_end_matches('/').split('/').last().unwrap_or("tmp.mp3");
            let filename = sanitize_filename::sanitize(filename);

            let status_str = format!("Downloading pronunciation files ({}/{})...", count, urls_len);
            count += 1;
            app_sender.send(AppEvent::SetStatus(status_str.as_str().into(), true, true));
            
            let audio_path = format!(r"tts_cache\{filename}");
            let audio_path_full = working_dir.join(&audio_path);
            if let Ok(exist) = audio_path_full.try_exists() && exist {
                arr_filenames.push(filename);
                continue;
            } else {
                tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
                let audio_resp = client.get(url).send().await?;
                dprintln!("{}", audio_resp.status());
                let status = audio_resp.status();
                if status.is_success() {
                    let audio_bytes = audio_resp.bytes().await?;
                    let mut file = File::create(&audio_path_full)?;
                    file.write_all(&audio_bytes)?;
                    arr_filenames.push(filename);
                } else {
                    match status {
                        StatusCode::NOT_FOUND => {
                            continue;
                        }
                        code => {
                            return Err(anyhow!(code.to_string()))
                        }
                    } 
                    //return Err(anyhow!(audio_resp.status().to_string()));
                }
            }
        }
        if !arr_filenames.is_empty() {
            Ok(arr_filenames)
        } else {
            Err(anyhow!(format!("no pronunciations were found for the selected language ({src_lang})")))
        }
        
    });

    let result = result?;
    Ok(result)
}