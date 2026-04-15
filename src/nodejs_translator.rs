use std::env;
use std::io::Write;
use std::process::{Command, Stdio};
use which::which;
//use anyhow::{anyhow, Result};

use std::{thread, time::Duration};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use crate::types::{AppEvent, Translator, Lang, UIState};

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, };

use std::io::{BufRead, BufReader};
use std::convert::AsRef;
use anyhow::{anyhow, Result};
use super::GLOBAL_SETTINGS;
use std::str::FromStr;

//TODO: catch thread panics

#[allow(clippy::type_complexity)]
pub struct NT {
    tx: mpsc::Sender<Option<(String, i64, Lang, Lang)>>,
    shared_receiver: Arc<Mutex<Receiver<Option<(String, i64, Lang, Lang)>>>>,
    is_running: Arc<AtomicBool>,
    current_src_id: Arc<AtomicI64>,
    current_src_text: Arc<RwLock<String>>,
    s: fltk::app::Sender<AppEvent>,
    uid: String,
    name: String,
    command: String,
    args: Vec<String>,
    src_lang: Lang,
    target_lang: Lang,
    reload_if_lang_changed: bool,
}

impl NT {
    pub fn new(s: fltk::app::Sender<AppEvent>, uid: String, name: String, command: String, args: Vec<String>, reload_if_lang_changed: bool) -> Self {
        let (tx, rx) = mpsc::channel::<Option<(String, i64, Lang, Lang)>>();
        let shared_receiver = Arc::new(Mutex::new(rx));
        let is_running = Arc::new(AtomicBool::new(false));
        let current_src_id = Arc::new(AtomicI64::new(0));
        let current_src_text = Arc::new(RwLock::new(String::from("")));
        let src_lang = Lang::En;
        let target_lang = Lang::Ru;
        Self { tx, shared_receiver, is_running, current_src_id, current_src_text, s, uid, name, command, args, src_lang, target_lang, reload_if_lang_changed}
    }
}

impl Translator for NT {
    fn terminate(&mut self) {
        if self.is_running.load(Ordering::Relaxed) {
            self.is_running.store(false, Ordering::Relaxed);
            let _ = self.tx.send(None);
        }
    }
    fn translate(&mut self, src_id: i64, selected_text: String, src_lang: Lang, target_lang: Lang, _is_lang_detected: bool) {
        println!("new src or target lang: {}", (self.src_lang != src_lang || self.target_lang != target_lang));
        println!("old lng: {} new lng: {}", self.src_lang.as_ref(), src_lang.as_ref());

        //fallback if src language changed, but process with specific language model is still running

        if self.reload_if_lang_changed
        && (self.src_lang != src_lang || self.target_lang != target_lang) 
        && self.is_running.load(Ordering::Relaxed) {
            self.terminate();
        }
        self.src_lang = src_lang;
        self.target_lang = target_lang;

        if !self.is_running.load(Ordering::Relaxed) {
            //println!("!is_brgmt_running");
            let shared_receiver = Arc::clone(&self.shared_receiver);
            let is_running = Arc::clone(&self.is_running);
            let current_src_id = Arc::clone(&self.current_src_id);
            let current_src_text = Arc::clone(&self.current_src_text);
            let s2 = self.s;
            let tx2 = self.tx.clone();

            let src_lang = self.src_lang.clone();
            let target_lang = self.target_lang.clone();
            let command = self.command.clone();
            let args = self.args.clone();
            let uid = self.uid.clone();
            let service_name = self.get_name();

            std::thread::spawn(
                move || {
                    //println!("---BRGMT OUTER LOOP---");
                    let brgmt_thread = run_node_thread(
                        Arc::clone(&is_running), 
                        s2, 
                        tx2,
                        shared_receiver, 
                        Arc::clone(&current_src_id), 
                        Arc::clone(&current_src_text),
                        src_lang, 
                        target_lang, 
                        command, 
                        args,
                        uid,
                        service_name
                    );
                    match brgmt_thread.join() {
                        Ok(value) => {
                            println!("Thread returned");
                            match value {
                                Ok(_) => {
                                    s2.send(AppEvent::SetReady(None, false));
                                },
                                Err(e) => {
                                    s2.send(AppEvent::SetStatus(e.to_string().into(), true, false));
                                }
                            }
                        },
                        Err(_e) => {
                            s2.send(AppEvent::SetReady(Some("Error: nodejs thread panic".to_string()), false));
                            is_running.store(false, Ordering::Relaxed);
                        },
                    };
                }
            );
            let _ = self.tx.send(Some((selected_text.clone(), src_id, self.src_lang.clone(), self.target_lang.clone() )));
        } else {
            let _ = self.tx.send(Some((selected_text.clone(), src_id, self.src_lang.clone(), self.target_lang.clone() )));
        }
    }

    fn get_uid(&self) -> String {
        self.uid.clone()
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
}

//TODO: platform-specific
use std::os::windows::process::CommandExt;
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
fn run_node_thread(
    is_running: Arc<AtomicBool>,
    s: fltk::app::Sender<AppEvent>,
    tx: mpsc::Sender<Option<(String, i64, Lang, Lang)>>,
    cloned_receiver: Arc<Mutex<Receiver<Option<(String, i64, Lang, Lang)>>>>,
    current_src_id: Arc<AtomicI64>,
    current_src_text:Arc<RwLock<String>>,
    src_lang: Lang,
    target_lang: Lang,
    command: String,
    args: Vec<String>,
    service_uid: String,
    service_name: String
) -> thread::JoinHandle<Result<()>> {
    //TODO: catch thread panics
    
    let working_dir = env::current_dir().unwrap();

    std::thread::spawn({
        let service_uid = service_uid.clone();
        is_running.store(true, Ordering::Relaxed);
        //let tx = tx;

        move || {
            //let full_path = working_dir.join(entry_point.as_str());
            let directory = working_dir.join(&format!("extensions\\{service_uid}"));
            let mut child;
            let src_lang_str = src_lang.as_ref();
            let target_lang_str = target_lang.as_ref();

            if which(&command).is_ok() {
                if command.starts_with(".\\") {
                    child = Command::new(working_dir.join(&command))
                        .args(args)
                        .arg(format!("--src={src_lang_str}"))
                        .arg(format!("--target={target_lang_str}"))
                        .creation_flags(CREATE_NO_WINDOW)
                        .current_dir(directory)
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn().expect("Failed to spawn child process");
                } else {
                    child = Command::new(&command)
                        .args(args)
                        .arg(format!("--src={src_lang_str}"))
                        .arg(format!("--target={target_lang_str}"))
                        .creation_flags(CREATE_NO_WINDOW)
                        .current_dir(directory)
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn().expect("Failed to spawn child process");
                };  
            } else {
                s.send(AppEvent::SetReady(Some("error".to_string()), false));
                panic!("");
            }

            let mut stdin = child.stdin.take().expect("Failed to open stdin");

            let stdout = child.stdout.take().expect("Failed to get stdout handle");

            thread::spawn({
                //let service_uid = service_uid.clone();
                let current_src_id: Arc<AtomicI64> = Arc::clone(&current_src_id);
                let current_src_text: Arc<RwLock<String>> = Arc::clone(&current_src_text);
                let name = service_name.clone();
                let is_running = is_running.clone();
                let mut src_lang = src_lang.clone();
                let mut target_lang = target_lang.clone();
                move || {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        let service_uid = service_uid.clone();
                        if let Ok(l) = line {
                            println!("Child says: {}", l.len());
                            println!("Child says: {}", l.clone());
                            if l.len() > 2 {
                                let src_text = current_src_text.read().unwrap();
                                //let src_text = *src_text;
                                let src_id = current_src_id.load(Ordering::Relaxed);
                                //one line - one response; inner newlines have been temporarily converted into <ENDOFLINE> tokens
                                let mut l2 = l.replace("<ENDOFLINE>", "\n");

                                let mut response_src_id: Option<i64> = None;
                                let regex_string = format!(r"<SRC_ID=(\d+)>");
                                let re = regex::Regex::new(&regex_string).unwrap();
                                if let Some(caps) = re.captures(&l2) {
                                    if let Some(matched_group) = caps.get(1) {
                                        response_src_id = Some(matched_group.as_str().parse::<i64>().unwrap());
                                        let full_match = caps.get(0).unwrap().as_str();
                                        l2 = l2.replacen(full_match, "", 1);
                                    }
                                }
                                

                                let mut src_lang_detected: Option<String> = None;
                                let regex_string = format!(r"<SRC_LANG_DETECTED=(..|auto|null|undefined)>");
                                let re = regex::Regex::new(&regex_string).unwrap();
                                if let Some(caps) = re.captures(&l2) {
                                    if let Some(matched_group) = caps.get(1) {
                                        src_lang_detected = Some(matched_group.as_str().to_string());
                                        let full_match = caps.get(0).unwrap().as_str();
                                        l2 = l2.replacen(full_match, "", 1);
                                    }
                                }
                                if let Some(lng) = src_lang_detected && let Ok(detected_lng) = Lang::from_str(&lng) {
                                    src_lang = detected_lng;
                                }

                                if let Some(id) = response_src_id && id == src_id {
                                    s.send(AppEvent::SaveTranslation((src_id, src_text.clone(), service_uid.clone(), src_lang.clone(), target_lang.clone(), l2.to_string())));
                                    s.send(AppEvent::UpdateUi(UIState {
                                        src_text: src_text.clone(),
                                        tr_uid: Some(service_uid), 
                                        translator: Some(name.clone()), 
                                        src: Some(src_lang.clone()), 
                                        target: Some(target_lang.clone()), 
                                        translation_text: Some(l2.to_string()),
                                        is_fav: None
                                    }, false));
                                    // + "\n" 
                                }
                            }        
                        }
                    }
                    println!("brgmt_thread_reader stopping");
                    is_running.store(false, Ordering::Relaxed);
                    let _ = tx.send(None);
                }
            });

            while is_running.load(Ordering::Relaxed) {
                //println!("---BRGMT INNER LOOP---");
                let receiver = cloned_receiver.lock();
                match receiver {
                    Ok(r) => {
                        let transl_request = r.recv_timeout(Duration::from_secs(GLOBAL_SETTINGS.nodejs_unload_timeout));
                        match transl_request {
                            Ok(res) => {
                                match res {
                                    Some((text, src_id, src_lng, target_lng)) => {
                                        current_src_id.store(src_id.clone(), Ordering::Relaxed);
                                        let mut data = current_src_text.write().unwrap();
                                        *data = text.clone();
                                        let src_lng = src_lng.as_ref();
                                        let target_lng = target_lng.as_ref();

                                        let text = format!("<SRC_ID={src_id}><SRC_LANG={src_lng}><TARGET_LANG={target_lng}>{text}");
                                        let text = text.replace("\r", "").replace("\n", "<ENDOFLINE>");
                                        if let Err(e) = stdin.write_all(text.as_bytes()) {
                                            is_running.store(false, Ordering::Relaxed);
                                            s.send(AppEvent::SetReady(Some(e.to_string()), false));
                                            //s.send(AppEvent::SetStatus("Error: Failed to write to stdin".into(), true, false));
                                            //child.kill();
                                        }
                                        if let Err(e) = stdin.write_all(b"\n") {
                                            is_running.store(false, Ordering::Relaxed);
                                            s.send(AppEvent::SetReady(Some(e.to_string()), false));
                                            //s.send(AppEvent::SetStatus("Error: Failed to write to stdin".into(), true, false));
                                            //child.kill();
                                        }
                                    },
                                    None => {
                                        is_running.store(false, Ordering::Relaxed);
                                    }
                                }
                            },
                            Err(_err) => {
                                is_running.store(false, Ordering::Relaxed);
                            }
                        };
                    },
                    Err(_e) => {
                        is_running.store(false, Ordering::Relaxed);
                    }
                }               
            };
            drop(stdin);
            let status = child.wait().expect("failed to wait on child");
            //stdout_thread.join().expect("failed to join stdout thread");
            let exit_code = status.code().unwrap();
            if exit_code == 1 {
                //s.send(AppEvent::SetReady());
                panic!("nodejs thread panic");
            } else if exit_code == 73 {
                return Err(anyhow!("Error: unsupported language"));
            } else if exit_code == 53 {
                return Err(anyhow!("Service error"));
            }
            println!("Child process exited with status: {}", status);
            println!("nodejs_thread stopping");
            Ok(())
        }
    })
}

