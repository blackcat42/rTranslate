use debug_print::{debug_println as dprintln};
use serde_json::Value;
use crate::types::{AppEvent, Translator, Lang, UIState, TranslResult};
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time::Duration};
use anyhow::{anyhow, Result};
use super::GLOBAL_SETTINGS;
use crate::utils::helpers::is_win7_or_greater;
use base64::{prelude::BASE64_STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use crate::utils::rt_request;
use std::fs::File;
use std::io::Read;

thread_local! {
    static JS_CALLS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

pub struct QT {
    is_running: Arc<AtomicBool>,
    app_sender: fltk::app::Sender<AppEvent>,
    name: String,
    uid: String,
    use_proxy: bool,
    emulation: Option<String>
}

impl QT {
    pub fn new(app_sender: fltk::app::Sender<AppEvent>, name: String, uid: String, use_proxy: bool, emulation: Option<String>) -> Self {
        let is_running = Arc::new(AtomicBool::new(false));
        Self {is_running, app_sender, name, uid, use_proxy, emulation}
    }
}
impl Translator for QT {
    fn terminate(&mut self) {
        
    }
    fn get_uid(&self) -> &str {
        &self.uid
    }
    fn get_name(&self) -> &str {
        &self.name
    }

    fn translate(&mut self, src_id: i64, text: String, src_lang: Lang, target_lang: Lang, is_lang_detected: bool) {

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
                                        
                    let transl_result = send_tr_request(&uid, text.clone(), src_lang.clone(), target_lang.clone(), is_lang_detected, use_proxy, emulation);
                    match transl_result {
                        Ok(t_text) => {
                            //dprintln!("lng: {}", t_text.1.unwrap_or("".to_string())); //TODO!
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
                            //app_sender.send(AppEvent::SetStatus(e.to_string().as_str().into(), true, false));
                        }
                    }
                    thread::sleep(Duration::from_millis((GLOBAL_SETTINGS.http_throttling * 1000.0) as u64));
                    is_running.store(false, Ordering::SeqCst);
                }
            });
        } else {
            self.app_sender.send(AppEvent::SetReady(Some("error: rate limit".to_string()), false));
            //self.app_sender.send(AppEvent::SetStatus("error: rate limit".into(), true, false));
        }
    }
}


fn send_tr_request(srvc_id: &str, selected_text: String, src_lang: Lang, target_lang: Lang, is_lang_detected: bool, proxy: bool, emulation: Option<String>) -> Result<(String, Lang)> {
    let mut response = "".to_string();

    let src_lang_ref = if is_lang_detected {
        src_lang.as_ref()
    } else {
        "auto"
    };

    let src_lang = convert_lang_to_qt_isize(src_lang);
    let target_lang = convert_lang_to_qt_isize(target_lang);

    let selected_text = serde_json::to_string(&selected_text)?;


    let response = qt_service_get_request_data(srvc_id, &selected_text, src_lang, target_lang, None, proxy, emulation.as_deref())?;
    let src_lang = convert_qt_isize_to_lang(response.sourceLanguage.unwrap_or(-1)).unwrap_or(Lang::Auto);
    //let target_lang = convert_qt_isize_to_lang(response.translationLanguage.unwrap_or(-1)).unwrap_or(Lang::Ru);
    if let Some(t) = response.translation {
        Ok((t, src_lang))
    } else {
        Err(anyhow!("qtranslate service error"))
    }
}


fn qt_service_get_request_data(srvc_id: &str, src_text: &str, from: isize, to: isize, handler: Option<String>, proxy: bool, emulation: Option<&str>) -> Result<ResponseData> {

    let calls = JS_CALLS.with(|d| d.get());
    if calls >= 10 {
        return Err(anyhow!("JS_CALLS limit"));
    }
    JS_CALLS.with(|d| d.set(calls + 1));

    let mut eval_str = "".to_string();
    if let Some(h) = handler && !h.is_empty() && is_valid_js_function_name(&h) {
        eval_str = format!("{h}({src_text}, {from}, {to})");
    } else {
        eval_str = format!("serviceTranslateRequest({src_text}, {from}, {to})");
    }
    let request_data = run_qt_service(srvc_id, &eval_str)?;

    match request_data {
        QTData::RequestData(d) => {
            let r_resp = make_req(d.clone(), proxy, emulation)?;
            qt_service_process_response_data(srvc_id, src_text, &r_resp, from, to, d.responseHandler.clone(), proxy, emulation)
        }
        _ => {
            Err(anyhow!("qtranslate service error"))
        }
    }

}

fn make_req(d: RequestData, proxy: bool, emulation: Option<&str>) -> Result<String> {
    let mut headers_map = std::collections::HashMap::new();
    if let Some(hdrs) = d.headers {
        for line in hdrs.split("\r\n") {
            if line.is_empty() {
                continue;
            }
            if let Some((key, value)) = line.split_once(": ") {
                headers_map.insert(key.to_string(), value.to_string());
            }
        }
    }

    let mut client = rt_request::Client::builder()
        .default_headers(headers_map)
        .timeout(Duration::from_secs(GLOBAL_SETTINGS.http_request_timeout))
        .proxy(proxy);
    if let Some(e) = emulation {
        client = client.emulation(e);
    }
    let mut client = client.build()?;
    let resp = if d.method == 1 {
        client.get(d.uri).send()?.text()
    } else if d.method == 2 {
        client = client.post(d.uri);
        //.version("HTTP_11")
        if let Some(b) = d.data {
            client = client.body(b);
        }
        client.send()?.text()
    } else {
        Err(anyhow!("http method undefined"))
    };
    resp
}

fn qt_service_process_response_data(
    srvc_id: &str,
    src_text: &str, 
    response_text: &str, 
    from: isize, 
    to: isize,
    handler: Option<String>,
    proxy: bool, emulation: Option<&str>
) -> Result<ResponseData> {

    let mut eval_str = "".to_string();
    
    let response_text = serde_json::to_string(response_text)?;
    //let response_text = response_text.replace('\'', "\\\'");
    if let Some(h) = handler && !h.is_empty() && is_valid_js_function_name(&h) {
        eval_str = format!("{h}({src_text}, {response_text}, {from}, {to})");
    } else {
        eval_str = format!("serviceTranslateResponse({}, {}, {}, {})", src_text, response_text, from, to );
    }
    let response = run_qt_service(srvc_id, &eval_str)?;

    match response {
        QTData::ResponseData(d) => {
            if let Some(ref h) = d.nextRequestHandler && !h.is_empty() {
                qt_service_get_request_data(srvc_id, src_text, from, to, Some(h.to_string()), proxy, emulation)
            } else {
                Ok(d.clone())
            }
        }
        _ => {
            Err(anyhow!("qtranslate service error"))
        }
    }
    
}

fn run_qt_service(srvc_id: &str, eval_str: &str) -> Result<QTData> {
    let working_dir = std::env::current_dir().unwrap();
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let command = ".\\qjs".to_string();
    
    if which::which(&command).is_ok() {
        let mut child = std::process::Command::new(working_dir.join(&command));
        let eval_str = BASE64_STANDARD.encode(eval_str.as_bytes());
        //println!("{}", &eval_str);
        //let arg_str = format!("--qtranslate-eval {}", eval_str);
        child.arg("--std").arg(".\\extensions\\qtranslate\\index.js")
        .arg("--service-id").arg(srvc_id)
        .arg("--qtranslate-eval").arg(&eval_str)
        .creation_flags(CREATE_NO_WINDOW)
        .current_dir(working_dir);

        let mut child_output = String::new();
        let mut child_err = String::new();

        if !is_win7_or_greater() {
            let output_file = File::create("qjs_output.tmp")?;
            let output_err_file = File::create("qjs_output_err.tmp")?;

            {
                let mut child = child
                .stdin(std::process::Stdio::null()) 
                .stdout(std::process::Stdio::from(output_file)) 
                .stderr(std::process::Stdio::from(output_err_file));
                let mut child = child.spawn()?;
                let status = child.wait()?;
            }

            if let Ok(mut file) = File::open("qjs_output.tmp") {
                file.read_to_string(&mut child_output)?;
            }
            if let Ok(mut file) = File::open("qjs_output_err.tmp") {
                file.read_to_string(&mut child_err)?;
            }
            if let Err(e) = std::fs::remove_file("qjs_output.tmp") {
                println!("error remove file: {}", e);
            }
            if let Err(e) = std::fs::remove_file("qjs_output_err.tmp") {
                println!("error remove file: {}", e);
            }
        } else {
            let output = child.output()?;
            child_err = String::from_utf8_lossy(&output.stderr).into_owned();
            child_output = String::from_utf8_lossy(&output.stdout).into_owned();
        }

        //dprintln!("cmd: {:?}", child);
        
        dprintln!("{}", child_err);
        dprintln!("{}", child_output);

        let error = extract_text("<SERVICE_ERROR_BEGIN>", "<SERVICE_ERROR_END>", &child_output);
        if !error.is_empty() {
            return Err(anyhow!(error));
        }
        
        let response_text = extract_text("<SERVICE_RESPONSE_BEGIN>", "<SERVICE_RESPONSE_END>", &child_output);
        //let response_text = decode_base64_string(response_text);
        
        let r = response_text.trim();
        dprintln!("QTrespnse-{}-", r);

        let req: Result<RequestData> = serde_json::from_str(&r).map_err(|e| anyhow::anyhow!("{e}"));
        if let Ok(request_data) = req {
            return Ok(QTData::RequestData(request_data));
        }

        let res: Result<ResponseData> = serde_json::from_str(&r).map_err(|e| anyhow::anyhow!("{e}"));
        if let Ok(response_data) = res {
            return Ok(QTData::ResponseData(response_data));
        } else {
            return Err(anyhow!("error decode js service response"));
        }

    } else {
        return Err(anyhow!("qjs not found"));
    }
}


pub enum QTData {
    RequestData(RequestData),
    ResponseData(ResponseData)
}

#[derive(Deserialize, Serialize, Clone)]
pub struct RequestData {
  method: u8, //1-GET 2-POST
  uri: String,
  data: Option<String>,
  headers: Option<String>,
  codepage: Option<usize>, 
  responseHandler: Option<String>
}

#[derive(Deserialize, Clone, Serialize, Debug)]
pub struct ResponseData {
  translation: Option<String>,
  sourceLanguage: Option<isize>,
  translationLanguage: Option<isize>,
  data: Option<String>,
  nextRequestHandler: Option<String>
}


fn decode_base64_string(encoded: String) -> Result<String> {
    let encoded = encoded.trim();
    match BASE64_STANDARD.decode(encoded).map_err(|e| anyhow::anyhow!("{e}")) {
        Ok(decoded_bytes) => {
            String::from_utf8(decoded_bytes).map_err(|e| anyhow::anyhow!("{e}"))
        }
        Err(e) => Err(e)
    }
}

fn is_valid_js_function_name(name: &str) -> bool {
    let pattern = regex::Regex::new(r"^[a-zA-Z_$][a-zA-Z0-9_$]*$").unwrap();
    if !pattern.is_match(name) {
        return false;
    }
    let reserved_words: std::collections::HashSet<&str> = [
        "abstract", "boolean", "break", "byte", "case", "catch", "char", "class", 
        "const", "continue", "debugger", "default", "delete", "do", "double", "else", 
        "enum", "export", "extends", "false", "final", "finally", "float", "for", 
        "function", "goto", "if", "implements", "import", "in", "instanceof", "int", 
        "interface", "long", "native", "new", "null", "package", "private", "protected", 
        "public", "return", "short", "static", "super", "switch", "synchronized", "this", 
        "throw", "throws", "transient", "true", "try", "typeof", "var", "void", 
        "volatile", "while", "with"
    ].iter().cloned().collect();

    !reserved_words.contains(name)
}

fn extract_text(start_tag: &str, end_tag: &str, input: &str) -> String {
    //TODO: utils
    let start_idx = input.find(start_tag);
    let end_idx = input.rfind(end_tag).unwrap_or(input.len());
    let text_start = if let Some(start_idx) = start_idx { 
        start_idx + start_tag.len() 
    } else {
        return "".to_string();
    };

    if text_start <= end_idx {
        input[text_start..end_idx].to_string()
    } else {
        "".to_string()
    }
}


fn convert_lang_to_qt_isize(lng: Lang) -> isize {
    match lng {
        //TODO
        Lang::Auto => 1,
        Lang::Af => 2,
        Lang::Az => 3,
        Lang::Sq => 4,
        Lang::Ar => 5,
        Lang::Hy => 6,
        Lang::Eu => 7,
        Lang::Be => 8,
        Lang::Bg => 9,
        Lang::Ca => 10,

        Lang::Zh => 11, //"zh-CN",
        //"zh-TW" => 12,

        Lang::Hr => 13,
        Lang::Cs => 14,
        Lang::Da => 15,
        Lang::Nl => 16,
        Lang::En => 17,
        Lang::Et => 18,
        Lang::Fi => 19,
        Lang::Tl => 20,
        Lang::Fr => 21,
        Lang::Gl => 22,
        Lang::De => 23,
        Lang::El => 24,
        Lang::Ht => 25,
        Lang::He => 26, //iw
        Lang::Hi => 27,
        Lang::Hu => 28,
        Lang::Is => 29,
        Lang::Id => 30,
        Lang::It => 31,
        Lang::Ga => 32,
        Lang::Ja => 33,
        Lang::Ka => 34,
        Lang::Ko => 35,
        Lang::Lv => 36,
        Lang::Lt => 37,
        Lang::Mk => 38,
        Lang::Ms => 39,
        Lang::Mt => 40,
        Lang::No => 41,
        Lang::Fa => 42,
        Lang::Pl => 43,
        Lang::Pt => 44,
        Lang::Ro => 45,
        Lang::Ru => 46,
        Lang::Sr => 47,
        Lang::Sk => 48,
        Lang::Sl => 49,
        Lang::Es => 50,
        Lang::Sw => 51,
        Lang::Sv => 52,
        Lang::Th => 53,
        Lang::Tr => 54,
        Lang::Uk => 55,
        Lang::Ur => 56,
        Lang::Vi => 57,
        Lang::Cy => 58,
        Lang::Yi => 59,
        Lang::Epo => 60,
        Lang::Hmn => 61,
        Lang::La => 62,
        Lang::Lo => 63,
        Lang::Kk => 64,
        Lang::Uz => 65,
        Lang::Si => 66,
        Lang::Tg => 67,
        Lang::Te => 68,
        Lang::Km => 69,
        Lang::Mn => 70,
        Lang::Kn => 71,
        Lang::Ta => 72,
        Lang::Mr => 73,
        Lang::Bn => 74,
        Lang::Tt => 75,
        //Lang::Auto => ,
        _ => -1,
    }
}

fn convert_qt_isize_to_lang(value: isize) -> Result<Lang> {
    match value {
        1 => Ok(Lang::Auto),
        2 => Ok(Lang::Af),
        3 => Ok(Lang::Az),
        4 => Ok(Lang::Sq),
        5 => Ok(Lang::Ar),
        6 => Ok(Lang::Hy),
        7 => Ok(Lang::Eu),
        8 => Ok(Lang::Be),
        9 => Ok(Lang::Bg),
        10 => Ok(Lang::Ca),

        11 => Ok(Lang::Zh),
        12 => Ok(Lang::Zh),

        13 => Ok(Lang::Hr),
        14 => Ok(Lang::Cs),
        15 => Ok(Lang::Da),
        16 => Ok(Lang::Nl),
        17 => Ok(Lang::En),
        18 => Ok(Lang::Et),
        19 => Ok(Lang::Fi),
        20 => Ok(Lang::Tl),
        21 => Ok(Lang::Fr),
        22 => Ok(Lang::Gl),
        23 => Ok(Lang::De),
        24 => Ok(Lang::El),
        25 => Ok(Lang::Ht),
        26 => Ok(Lang::He),
        27 => Ok(Lang::Hi),
        28 => Ok(Lang::Hu),
        29 => Ok(Lang::Is),
        30 => Ok(Lang::Id),
        31 => Ok(Lang::It),
        32 => Ok(Lang::Ga),
        33 => Ok(Lang::Ja),
        34 => Ok(Lang::Ka),
        35 => Ok(Lang::Ko),
        36 => Ok(Lang::Lv),
        37 => Ok(Lang::Lt),
        38 => Ok(Lang::Mk),
        39 => Ok(Lang::Ms),
        40 => Ok(Lang::Mt),
        41 => Ok(Lang::No),
        42 => Ok(Lang::Fa),
        43 => Ok(Lang::Pl),
        44 => Ok(Lang::Pt),
        45 => Ok(Lang::Ro),
        46 => Ok(Lang::Ru),
        47 => Ok(Lang::Sr),
        48 => Ok(Lang::Sk),
        49 => Ok(Lang::Sl),
        50 => Ok(Lang::Es),
        51 => Ok(Lang::Sw),
        52 => Ok(Lang::Sv),
        53 => Ok(Lang::Th),
        54 => Ok(Lang::Tr),
        55 => Ok(Lang::Uk),
        56 => Ok(Lang::Ur),
        57 => Ok(Lang::Vi),
        58 => Ok(Lang::Cy),
        59 => Ok(Lang::Yi),
        60 => Ok(Lang::Epo),
        61 => Ok(Lang::Hmn),
        62 => Ok(Lang::La),
        63 => Ok(Lang::Lo),
        64 => Ok(Lang::Kk),
        65 => Ok(Lang::Uz),
        66 => Ok(Lang::Si),
        67 => Ok(Lang::Tg),
        68 => Ok(Lang::Te),
        69 => Ok(Lang::Km),
        70 => Ok(Lang::Mn),
        71 => Ok(Lang::Kn),
        72 => Ok(Lang::Ta),
        73 => Ok(Lang::Mr),
        74 => Ok(Lang::Bn),
        75 => Ok(Lang::Tt),
        _ => Err(anyhow!("Unsupported language ID")),
    }
}
