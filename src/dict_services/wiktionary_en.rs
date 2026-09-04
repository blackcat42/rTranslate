use debug_print::{debug_println as dprintln};
use crate::types::{AppEvent, Dictionary, Lang, UIStateDict, DictResult};
use crate::utils::rt_request::{
    Client,
    //Version
};
use crate::utils::html_to_bbcode::html_to_bbcode;
use crate::utils::html_to_bbcode::HTMLSelectorType;
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

fn tag_handler(tag: &tl::HTMLTag, inner_text: String) -> String {
    //The "$RT_NEWLINE$" token preserves line breaks during trim()
    
    let tag_name = tag.name().as_utf8_str();
    if let Some(class_name) = tag.attributes().class() {
        let class_name = class_name.as_utf8_str();
        if class_name.contains("thumbcaption") 
        || class_name.contains("thumbimage") 
        || class_name.contains("noprint") {
            return String::new();
        }
        if class_name.contains("etyl") {
            return format!("{}:", inner_text);
        }
        if class_name.contains("h-quotation") {
            return format!(" ❝ {}❞", inner_text);
        }
        if class_name.contains("e-translation") || class_name.contains("e-transliteration") {
            return format!(" // {}", inner_text);
        }

        if class_name.contains("extiw") {
            return inner_text;
        }
        if class_name.contains("citation-whole") {
            let t = format!("\n[quote]{}[/quote]\n", inner_text);
            return t;
        }
        if class_name.contains("e-example") {
            let t = format!("[quote]  {}[/quote]", inner_text);
            return t;
        }

        if class_name.contains("etytree-block") {
            let t = format!("{}", inner_text.trim());
            return t;
        }

        if class_name.contains("etytree-connector-vertical") {
            return " -> ".to_string();
        }

        if class_name.contains("NavHead") {
            let t = format!("\n[c green]{}:[/c]", inner_text);
            return t;
        }
        
    }
    if let Some(Some(title)) = tag.attributes().get("title") {
        if title == "Appendix:Glossary" {
            return inner_text;
        }
    }

    match tag_name.as_ref() {
        "h1" => format!("\n$RT_NEWLINE$[c red]{}[/c]", inner_text.trim().to_uppercase()),
        "h2" => {
            let inner_text = inner_text.trim().to_uppercase();
            let qwe = "-".repeat(inner_text.chars().count());
            format!("\n$RT_NEWLINE$[c indigo]----------{qwe}----------\n          {inner_text}\n----------{qwe}----------[/c]")
        },
        "h3" => format!("\n$RT_NEWLINE$[c green]{}[/c]", inner_text.trim().to_uppercase()),
        "h4" => format!("\n$RT_NEWLINE$[b]{}[/b]", inner_text.trim().to_uppercase()),
        "h5" => format!("\n$RT_NEWLINE${}", inner_text.trim().to_uppercase()),
        "p" => format!("\n{}", inner_text),
        "strong" | "b" => format!("[b]{}[/b]", inner_text),
        "em" | "i" => format!("[i]{}[/i]", inner_text),
        "code" => format!("`{}`", inner_text),
        "br" => "\n".to_string(),
        "li" => format!("\n  • {}", inner_text.trim()),
        "a" => {
            format!("[c blue]{}[/c]", inner_text)
        },
        "div" => {
            format!("\n{}", inner_text)
        },
        "span" => {
            format!("{}", inner_text)
        },
        "script" | "style" | "head" | "nav" | "sup" => {
            "".to_string() 
        },
        _ => format!("{}", inner_text),    
    }
}

impl WDEn {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>, name: String, uid: String, use_proxy: bool, emulation: Option<String>) -> Self {
        let is_running = Arc::new(AtomicBool::new(false));
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
                            let t_text = html_to_bbcode(&t_text, HTMLSelectorType::Id("mw-content-text".to_string()), tag_handler);
                            match t_text {
                                Ok(res) => {
                                    let res = res.replace("([c blue]edit[/c])", "");
                                    let res = res.replace("  • [quote]", "[quote]  ");

                                    app_sender.send(AppEvent::SaveDictEntry(DictResult {
                                        src_id,
                                        dict_uid: uid.clone(),
                                        text: res.clone(),
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
                                        dict_text: Some(res),
                                        is_fav: None
                                    }, false));
                                },
                                Err(e) => {
                                    app_sender.send(AppEvent::SetReady(Some(e.to_string()), true));
                                }
                            }
                        }
                        Err(e) => {
                            app_sender.send(AppEvent::SetReady(Some(e.to_string()), true));
                        }
                    }
                    thread::sleep(Duration::from_millis((GLOBAL_SETTINGS.http_throttling * 1000.0) as u64));
                    is_running.store(false, Ordering::SeqCst);
                }
            });
        } else {
            self.app_sender.send(AppEvent::SetReady(Some("error: rate limit".to_string()), true));
        }
    }
}

#[allow(unused_variables)]
fn send_tr_request(selected_text: String, proxy: bool, emulation: Option<String>) -> Result<String> {
    //let req_string = "https://en.wiktionary.org/w/index.php?action=raw".to_string();
    let req_string = format!("https://en.wiktionary.org/wiki/{}", selected_text.to_lowercase());

    let mut headers = std::collections::HashMap::new();
    headers.insert("User-Agent".into(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.104 Safari/537.36".into());

    let mut client = Client::builder()
        .timeout(Duration::from_secs(GLOBAL_SETTINGS.http_request_timeout))
        .default_headers(headers)
        .proxy(proxy);
    if let Some(e) = emulation {
        client = client.emulation(e);
    }
    let client = client.build()?;
    
    //let resp = client.get(req_string).query([("title", selected_text.to_lowercase())]).send()?.text()?;
    let resp = client.get(req_string).send()?.text()?;
    let result = Ok(resp);

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
}
