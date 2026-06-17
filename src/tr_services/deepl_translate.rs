use debug_print::{debug_println as dprintln};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::types::{AppEvent, Translator, Lang, UIState};
//use ureq::Agent;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time::Duration};
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{anyhow, Result};
use super::GLOBAL_SETTINGS;
use super::TOKIO_RT;
use std::str::FromStr;

use wreq::{
    Client,
    Version,
    header,
    StatusCode
};
use wreq_util::{
    Emulation
};

pub struct DL {
    is_running: Arc<AtomicBool>,
    app_sender: fltk::app::Sender<AppEvent>,
    name: String,
    uid: String,
    use_proxy: bool
}

impl DL {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>, name: String, uid: String, use_proxy: bool) -> Self {
        let is_running = Arc::new(AtomicBool::new(false));
        Self {is_running, app_sender, name, uid, use_proxy}
    }
}
impl Translator for DL {
    fn terminate(&mut self) {
        
    }
    fn get_uid(&self) -> String {
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
                    let transl_result = send_tr_request(text.clone(), src_lang.clone(), target_lang.clone(), is_lang_detected, proxy);
                    match transl_result {
                        Ok(t_text) => {
                            //dprintln!("lng: {}", t_text.1.unwrap_or("".to_string())); //TODO!
                            app_sender.send(AppEvent::SaveTranslation((src_id, text.clone(), uid.clone(), t_text.1.clone(), target_lang.clone(), t_text.0.clone())));
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












// Original source (golang): github.com/OwO-Network/DeepLX/blob/8d145722eba578fac10d4ef22959cb92044b1673/translate/translate.go
// https://github.com/OwO-Network/DeepLX/blob/8d145722eba578fac10d4ef22959cb92044b1673/translate/types.go
// Author: Vincent Young
#[derive(Serialize)]
struct PostData {
    jsonrpc: String,
    method: String,
    id: u32,
    params: PostDataParams,
}

#[derive(Serialize)]
struct PostDataParams {
    splitting: String,
    lang: DLang,
    texts: Vec<TextItem>,
    timestamp: u32 //it's not a timestamp, actually
}

#[derive(Serialize)]
struct TextItem {
    text: String,
    #[serde(rename = "requestAlternatives")]
    request_alternatives: u32
}

#[derive(Serialize)]
struct DLang {
    source_lang_user_selected: String, // Can be "auto"
    target_lang: String
}

// DeepLXTranslationResult represents the final translation result
/*struct DeepLXTranslationResult {
    code: i64,
    id: i64,
    message: String,
    data: String, // The primary translated text
    alternatives: Vec<String>, // Other possible translations
    source_lang: String,
    target_lang: String,
    method: String
}*/

fn send_tr_request(selected_text: String, src_lang: Lang, target_lang: Lang, is_lang_detected: bool, proxy: Option<wreq::Proxy>) -> Result<(String, Lang)> {
    let mut response = "".to_string();

    let src_lang_ref = if is_lang_detected {
        src_lang.as_ref().to_uppercase()
    } else {
        "auto".to_string()
    };

    let rt = TOKIO_RT.get_or_init(|| {
        tokio::runtime::Runtime::new().expect("Tokio Runtime Error")
    });

    let result = rt.block_on(async {

        // Prepare translation request using new LMT_handle_texts method
        let id = get_random_number();
        let i_count = get_i_count(&selected_text);
        let timestamp = get_time_stamp(i_count);

        let post_data = PostData {
            jsonrpc: "2.0".to_string(),
            method: "LMT_handle_texts".to_string(),
            id,
            params: PostDataParams {
                splitting: "newlines".to_string(),
                lang: DLang {
                    source_lang_user_selected: src_lang_ref.to_string(),
                    target_lang: target_lang.as_ref().to_uppercase(),
                },
                texts: vec![TextItem {
                    text: selected_text.clone(),
                    request_alternatives: 3,
                }],
                timestamp,
            },
        };

        // Format and apply body manipulation method like TypeScript
        let post_str = format_post_string(post_data);
        let post_str = handler_body_method(id, &post_str);

        // Make translation request
        let url_full = "https://www2.deepl.com/jsonrpc";
        //ita-free.www.deepl.com

        // Set headers to simulate browser request
        let mut headers = header::HeaderMap::new();
        headers.insert("Content-Type", header::HeaderValue::from_static("application/json"));
        headers.insert("Accept", header::HeaderValue::from_static("*/*"));
        headers.insert("Accept-Language", header::HeaderValue::from_static("en-US,en;q=0.9"));
        headers.insert("Accept-Encoding", header::HeaderValue::from_static("gzip"));
        //headers.insert("Accept-Encoding", header::HeaderValue::from_static("gzip, deflate, br, zstd"));
        headers.insert("Origin", header::HeaderValue::from_static("https://www.deepl.com"));
        headers.insert("Referer", header::HeaderValue::from_static("https://www.deepl.com/"));
        headers.insert("Sec-Fetch-Dest", header::HeaderValue::from_static("empty"));
        headers.insert("Sec-Fetch-Mode", header::HeaderValue::from_static("cors"));
        headers.insert("Sec-Fetch-Site", header::HeaderValue::from_static("same-site"));
        headers.insert("User-Agent", header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36"));

        // Create a new req client
        let mut client = wreq::Client::builder()
            .emulation(Emulation::Chrome137)
            .default_headers(headers)
            .timeout(Duration::from_secs(GLOBAL_SETTINGS.http_request_timeout))
            .gzip(true);
        client = if let Some(proxy) = proxy {
            client.proxy(proxy)
        } else {
            client
        };
        
        let client = client.build()?;
        //client := req.C().SetTLSFingerprintRandomized()
        let resp = client.post(url_full)
            //.version(Version::HTTP_11)
            .body(post_str).send().await?;
        let status = resp.status();
        if status.is_success() {
            let resp = resp.text().await?;
            return Ok(resp);
        }
        match status {
            StatusCode::OK => {
                let resp = resp.text().await?;
                Ok(resp)
            }
            code => {
                Err(anyhow!(code.to_string()))
            }
        } 
    });

    match result {
        Ok(json_data) => {
            dprintln!("{}", &json_data);
            let json_data: Value = serde_json::from_str(json_data.as_str())?;
            let mut src_lng_suggested = src_lang.clone();
            if let Some(texts) = json_data.pointer("/result/texts") {

                if let Some(texts_arr) = texts.as_array() 
                && let Some(arr_first_item) = texts_arr.get(0) 
                && let Some(text_obj) = arr_first_item.as_object() {

                    if let Some(text) = text_obj.get("text") 
                    && let Some(text_str) = text.as_str() 
                    && text_str.len() > 0 {
                        response.push_str(text_str);
                    }

                    if let Some(alt) = text_obj.get("alternatives") && let Some(alt_arr) = alt.as_array() {
                        dbg!(alt_arr);
                    }
                } else {
                    return Err(anyhow!("response error: empty array (/result/texts)"));
                }
            } else {
                dprintln!("response error");
            }

            if let Some(lng) = json_data.pointer("/result/lang") {
                src_lng_suggested = Lang::from_str(lng.as_str().unwrap_or("auto").to_lowercase().as_str()).unwrap_or(src_lang);
            }
            // if let Some(lng) = data.pointer("/result/lang_is_confident") {
            // }
            // if let Some(lng_detected) = data.pointer("/result/detectedLanguages") {
            //     if let Some(lng_arr) = lng_detected.as_array() {
            //     }
            // }
            /*
            // Get alternatives
            var alternatives []string
            alternativesArray := textsArray[0].Get("alternatives").Array()
            for _, alt := range alternativesArray {
                altText := alt.Get("text").String()
                if altText != "" {
                    alternatives = append(alternatives, altText)
                }
            }
            */

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
}





// Original src (golang): github.com/OwO-Network/DeepLX/blob/8d145722eba578fac10d4ef22959cb92044b1673/translate/utils.go
// Author: Vincent Young

// UTILS

// getICount returns the number of 'i' characters in the text
fn get_i_count(translate_text: &str) -> u32 {
    let count = translate_text.chars().filter(|&c| c == 'i').count();
    count as u32
}

// getRandomNumber generates a random number for request ID
fn get_random_number() -> u32 {
    if let Ok(nanos) = SystemTime::now().duration_since(UNIX_EPOCH) {
        let nanos = nanos.subsec_nanos();
        (rand(nanos, 0, 99999) + 100000) * 1000
    } else {
        999999999_u32
    }    
}

// getTimeStamp generates timestamp for request based on i count
fn get_time_stamp(i_count: u32) -> u32 {
    if let Ok(ts) = SystemTime::now().duration_since(UNIX_EPOCH) {
        let ts = ts.subsec_millis();
        if i_count > 0 {
            let i_count = i_count + 1;
            ts - (ts % i_count) + i_count
        } else {
            ts
        }
    } else {
        999_u32
    } 
}

// formatPostString formats the request JSON string with specific spacing rules
fn format_post_string(post_data: PostData) -> String {
    serde_json::to_string(&post_data).unwrap()
}

// handlerBodyMethod manipulates the request body based on random number calculation
fn handler_body_method(random: u32, body: &str) -> String {
    let calc = ((random + 5) % 29 == 0) || ((random + 3) % 13 == 0);
    if calc {
        body.replacen("\"method\":\"", "\"method\" : \"", 1)
    } else {
        body.replacen("\"method\":\"", "\"method\": \"", 1)
    }
}

fn rand(mut seed: u32, min:u32, max: u32) -> u32 {
    seed ^= seed << 13;
    seed ^= seed >> 7;
    seed ^= seed << 17;

    min + (seed % (max - min))
}

