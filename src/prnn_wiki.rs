use crate::types::{AppEvent, PRNNService};
use std::{thread, time::Duration};
use std::sync::{Arc };
use std::sync::atomic::{AtomicBool, Ordering};
use std::fs::File;
use std::io::Write;
use anyhow::{anyhow, Result};
use serde_json::Value;
use wreq::{
    Client,
    Version,
    header
};
use wreq_util::{
    Emulation
};
use super::GLOBAL_SETTINGS;
use super::TOKIO_RT;
use serde::{Deserialize, Serialize};

//#[allow(dead_code)]
//#[allow(clippy::upper_case_acronyms)]
pub struct WP {
    is_running: Arc<AtomicBool>,
    app_sender: fltk::app::Sender<AppEvent>,
    name: String,
}

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

                    let filenames = send_pr_request(text.clone(), src_id);
                    match filenames {
                        Ok(p_files) => {
                            //dbg!(&p_file);
                            for item in p_files.iter() {
                                dbg!(&item);
                                app_sender.send(AppEvent::PRNNSave((src_id, "prnn_wiki".to_string(), item.clone() )));
                            }
                            if let Some (str1) = p_files.get(0) {
                                //dbg!(&str1);
                                app_sender.send(AppEvent::TTSPlay(str1.to_string()));
                            }
                        }
                        Err(_e) => {
                            app_sender.send(AppEvent::Message("tts error".into()));
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

/*#[derive(Deserialize, Serialize, Debug)]
struct MediaList {
    items: Vec<MediaItem>
}

#[derive(Deserialize, Serialize, Debug)]
struct MediaItem {
    title: String,
    type: String
}*/

fn send_pr_request(selected_text: String, src_id: i64) -> Result<Vec<String>> {
    
    //let req_string = format!("https://en.wiktionary.org/api/rest_v1/page/media-list/{}", selected_text.to_lowercase());

    let rt = TOKIO_RT.get_or_init(|| {
        tokio::runtime::Runtime::new().expect("Tokio Runtime Error")
    });

    let mut headers = header::HeaderMap::new();
    headers.insert("Accept-Encoding", header::HeaderValue::from_static("gzip"));
    headers.insert("Host", header::HeaderValue::from_static("en.wiktionary.org"));

    let result = rt.block_on(async {
        let mut arr: Vec<String> = vec![];
        let mut arr_filenames: Vec<String> = vec![];

        let client = Client::builder()
            //.emulation(Emulation::Chrome137)
            .default_headers(headers)
            .timeout(Duration::from_secs(GLOBAL_SETTINGS.http_request_timeout))
            .user_agent("User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.104 Safari/537.36")
            //.cookie_store(true)
            .gzip(true)
            .build()?;


        /*let resp = client.get(req_string).send().await?.text().await?;
        println!("{}", resp);
        let v: Value = serde_json::from_str(resp.as_str())?;
        let items = v.pointer("/items").and_then(|v| v.as_array());
        dbg!(items);
        //let value: MediaList = serde_json::from_str(json_data.as_str())?;
        //let mut src_lng_suggested = None;
        if let Some(items) = items {
            for item_value in items {
                //let item = item_value.as_object();
                dbg!(item_value);
                if let Some(i) = item_value.as_object().ok_or(anyhow!("json parse error"))?.get("type").ok_or(anyhow!("json parse error"))?.as_str() && i == "audio" {
                    if let Some(ii) = item_value.get("title") {
                        //dbg!(ii);
                        let ii = ii.to_string().replace("File:", "").trim_matches('"').to_string();
                        arr.push(ii);
                    }
                     //println!("{}", text);
                }
            };
        }*/



        
        let req_string2 = format!("https://en.wiktionary.org/wiki/{}", selected_text.to_lowercase());
        let resp_full_text = client.get(req_string2).send().await?.text().await?;


        let working_dir = std::env::current_dir()?;
        
        let item0 = regex::escape("upload.wikimedia.org/wikipedia/commons/");
        let regex_string_audio = format!(r"(?i){item0}(./../?)([^/]+\.wav|[^/]+\.ogg)");
        let re = regex::Regex::new(&regex_string_audio)?;

        for caps in re.captures_iter(&resp_full_text) {
            dbg!(&caps);
            if let Some(inner_text) = caps.get(1) && let Some(text_filename) = caps.get(2) {
                //adr
                let infix = inner_text.as_str();
                let flnm = text_filename.as_str();
                let full_url = format!("https://upload.wikimedia.org/wikipedia/commons/{infix}{flnm}");
                let filename = sanitize_filename::sanitize(flnm);
                println!("{}", filename);
                println!("{}", filename.ends_with(".ogg"));
                if !filename.ends_with(".ogg") 
                && !filename.ends_with(".mp3") 
                && !filename.ends_with(".wav") {
                    continue;
                }

                let audio_path = format!(r"tts_cache\{filename}");
                let audio_path_full = working_dir.join(&audio_path);
                if let Ok(exist) = audio_path_full.try_exists() && exist {
                    continue;
                } else {
                    //https://wikitech.wikimedia.org/wiki/Robot_policy
                    tokio::time::sleep(tokio::time::Duration::from_millis(1300)).await;
                    let audio_resp = client.get(full_url).send().await?;
                    println!("{}", audio_resp.status());
                    if audio_resp.status().is_success() {
                        let audio_bytes = audio_resp.bytes().await?;
                        let mut file = File::create(&audio_path_full)?;
                        file.write_all(&audio_bytes)?;
                        arr_filenames.push(filename);
                    } else {
                        return Err(anyhow!("https error"));
                    }
                }
            }
        }

        /*if arr.len() > 0 {
            for item in arr {
                /*tokio::time::sleep(tokio::time::Duration::from_secs(GLOBAL_SETTINGS.http_request_timeout)).await;
                let req_string = format!("https://en.wiktionary.org/w/api.php");
                let resp = client.get(req_string).query(&[
                    ("action", "query".to_string()),
                    ("format", "json".to_string()),
                    ("prop", "imageinfo".to_string()),
                    ("iiprop", "url".to_string()),
                    ("titles", item)
                ]).send().await?.text().await?;
                let v: Value = serde_json::from_str(resp.as_str())?;
                let val = v.pointer("/query/pages/-1/imageinfo").and_then(|v| v.as_array());
                if let Some(value) = val && let Some(url) = value.get(0) {
                    arr_url.push(url.to_string());
                    //println!("{}", url);
                }*/


                
                let working_dir = std::env::current_dir()?;
                let filename = sanitize_filename::sanitize(&item);
                let audio_path = format!(r"tts_cache\{filename}");
                let audio_path_full = working_dir.join(&audio_path);

                if let Ok(exist) = audio_path_full.try_exists() && exist {
                    continue;
                }

                let item0 = regex::escape("upload.wikimedia.org/wikipedia/commons/");
                let item1 = regex::escape(&urlencoding::encode(&item));
                let regex_string = format!("(?i){item0}(./../?)({item1})");
                println!("{}", regex_string);
                let re = regex::Regex::new(&regex_string)?;

                //for caps in re.captures_iter(&resp_full_text) {
                if let Some(caps) = re.captures(&resp_full_text) {
                    //dbg!(&caps);
                    if let Some(inner_text) = caps.get(1) 
                    && let Some(text_filename) = caps.get(2) 
                    && (
                        filename.ends_with(".ogg") 
                        || filename.ends_with(".mp3") //???
                        || filename.ends_with(".wav")
                    ) {
                        //adr
                        let infix = inner_text.as_str();
                        let flnm = text_filename.as_str();
                        let full_url = format!("https://upload.wikimedia.org/wikipedia/commons/{infix}{flnm}");

                        tokio::time::sleep(tokio::time::Duration::from_secs(GLOBAL_SETTINGS.http_request_timeout)).await;
                        let audio_resp = client.get(full_url).send().await?;
                        if audio_resp.status().is_success() {
                            let audio_bytes = audio_resp.bytes().await?;
                            let mut file = File::create(&audio_path_full)?;
                            file.write_all(&audio_bytes)?;
                            arr_url.push(filename);
                        } else {
                            return Err(anyhow!("https error"));
                        }

                        /*while let Some(item) = audio_bytes.next().await {
                            let chunk = item?;
                            file.write_all(&chunk)?;
                        }*/
                    }
                }
            };
        } else {
            return Err(anyhow!("error"));
        }*/
        Ok(arr_filenames)
    });


    /*
    "https://en.wiktionary.org/api/rest_v1/page/media-list/maybe"
    items.0.type == "audio"
    items.0.title == "File:En-uk-maybe.ogg"

    "https://en.wiktionary.org/w/api.php?action=query&format=json&prop=imageinfo&titles=File:en-us-maybe.ogg&iiprop=url"

    query.pages.-1.imageinfo.0.url == "https://upload.wikimedia.org/wikipedia/commons/4/4d/En-us-maybe.ogg"*/

    let result = result?;
    Ok(result)
}