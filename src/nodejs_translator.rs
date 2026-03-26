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

use super::GLOBAL_SETTINGS;

//TODO: catch thread panics

#[allow(clippy::type_complexity)]
pub struct NT {
    tx: mpsc::Sender<Option<(String, i64)>>,
    shared_receiver: Arc<Mutex<Receiver<Option<(String, i64)>>>>,
    is_running: Arc<AtomicBool>,
    current_src_id: Arc<AtomicI64>,
    current_src_text: Arc<RwLock<String>>,
    s: fltk::app::Sender<AppEvent>,
    uid: String,
    name: String,
    entry_point: String,
    src_lang: Lang,
    target_lang: Lang,
}

impl NT {
    pub fn new(s: fltk::app::Sender<AppEvent>, uid: String, name: String, entry_point: String) -> Self {
        let (tx, rx) = mpsc::channel::<Option<(String, i64)>>();
        let shared_receiver = Arc::new(Mutex::new(rx));
        let is_running = Arc::new(AtomicBool::new(false));
        let current_src_id = Arc::new(AtomicI64::new(0));
        let current_src_text = Arc::new(RwLock::new(String::from("")));
        //let uid = "brgmt".to_string();
        let src_lang = Lang::En;
        let target_lang = Lang::Ru;
        Self { tx, shared_receiver, is_running, current_src_id, current_src_text, s, uid, name, entry_point, src_lang, target_lang}
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
        let is_reload_needed = false;
        if (self.src_lang != src_lang || self.target_lang != target_lang) 
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
            let selected_text2 = selected_text.clone();
            let s2 = self.s;
            let tx2 = self.tx.clone();

            let src_lang = self.src_lang.clone();
            let target_lang = self.target_lang.clone();
            let entry_point = self.entry_point.clone();
            let uid = self.uid.clone();
            let service_name = self.get_name();

            std::thread::spawn(
                move || {
                    //println!("---BRGMT OUTER LOOP---");
                    let brgmt_thread = run_node_thread(
                        Arc::clone(&is_running), 
                        s2, 
                        tx2,
                        selected_text2, 
                        shared_receiver, 
                        Arc::clone(&current_src_id), 
                        Arc::clone(&current_src_text),
                        src_lang, 
                        target_lang, 
                        entry_point, 
                        uid,
                        service_name
                    );
                    match brgmt_thread.join() {
                        Ok(_value) => {
                            println!("Thread returned");
                            s2.send(AppEvent::SetReady());
                        },
                        Err(_e) => {
                            s2.send(AppEvent::SetReady());
                            s2.send(AppEvent::SetStatus("Error: nodejs thread panic".into(), true, false));
                            is_running.store(false, Ordering::Relaxed);
                        },
                    };
                }
            );
            let _ = self.tx.send(Some((selected_text.clone(), src_id)));
        } else {
            //println!("is_brgmt_running");
            if !is_reload_needed {
                let _ = self.tx.send(Some((selected_text.clone(), src_id)));
            }
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
    tx: mpsc::Sender<Option<(String, i64)>>,
    _text: String,
    cloned_receiver: Arc<Mutex<Receiver<Option<(String, i64)>>>>,
    current_src_id: Arc<AtomicI64>,
    current_src_text:Arc<RwLock<String>>,
    src_lang: Lang,
    target_lang: Lang,
    entry_point: String,
    service_uid: String,
    service_name: String
) -> thread::JoinHandle<()> {
    //TODO: catch thread panics
    
    let working_dir = env::current_dir().unwrap();

    std::thread::spawn({
        let service_uid = service_uid.clone();
        is_running.store(true, Ordering::Relaxed);
        //let tx = tx;

        move || {
            let full_path = working_dir.join(entry_point.as_str());
            let directory = &full_path.parent().unwrap();
            let mut child;
            let src_lang_str = src_lang.as_ref();
            let target_lang_str = target_lang.as_ref();

            if which(r".\deno").is_ok() {
                child = Command::new(working_dir.join(r".\deno"))
                    .arg("--allow-read=.")
                    .arg("--deny-net")
                    //.arg("--allow-write=.")
                    .arg(&full_path)
                    .arg(format!("--src={src_lang_str}"))
                    .arg(format!("--target={target_lang_str}"))
                    .creation_flags(CREATE_NO_WINDOW)
                    .current_dir(directory)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn().expect("Failed to spawn child process");
            } else if which("deno").is_ok() {
                child = Command::new("deno")
                    .arg("--allow-read=.")
                    .arg("--deny-net")
                    //.arg("--allow-write=.")
                    .arg(&full_path)
                    .arg(format!("--src={src_lang_str}"))
                    .arg(format!("--target={target_lang_str}"))
                    .creation_flags(CREATE_NO_WINDOW)
                    .current_dir(directory)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn().expect("Failed to spawn child process");
            } /*TODO: else if which("node").is_ok() {
                child = Command::new("node")
                    .arg(working_dir.join(r"bergamot\app.cjs"))
                    .arg(format!("--src={src_lang_str}"))
                    .arg(format!("--target={target_lang2}"))
                    .creation_flags(CREATE_NO_WINDOW)
                    .current_dir(working_dir.join("bergamot"))
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn().expect("Failed to spawn child process");
            } */else {
                s.send(AppEvent::SetReady());
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
                move || {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        let service_uid = service_uid.clone();
                        if let Ok(l) = line {
                            println!("Child says: {}", l.len());
                            if l.len() > 2 {
                                let src_text = current_src_text.read().unwrap();
                                //let src_text = *src_text;
                                let src_id = current_src_id.load(Ordering::Relaxed);
                                //one line - one response; inner newlines have been temporarily converted into <ENDOFLINE> tokens
                                let l2 = l.replace("<ENDOFLINE>", "\n");
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
                                    Some((text, src_id)) => {
                                        current_src_id.store(src_id, Ordering::Relaxed);
                                        let mut data = current_src_text.write().unwrap();
                                        *data = text.clone();

                                        let text = text.replace("\r", "").replace("\n", "<ENDOFLINE>");
                                        if let Err(_event) = stdin.write_all(text.as_bytes()) {
                                            is_running.store(false, Ordering::Relaxed);
                                            s.send(AppEvent::SetReady());
                                            s.send(AppEvent::SetStatus("Error: Failed to write to stdin".into(), true, false));
                                            //child.kill();
                                        }
                                        if let Err(_event) = stdin.write_all(b"\n") {
                                            is_running.store(false, Ordering::Relaxed);
                                            s.send(AppEvent::SetReady());
                                            s.send(AppEvent::SetStatus("Error: Failed to write to stdin".into(), true, false));
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
            if status.code().expect("nodejs error") == 1 {
                s.send(AppEvent::SetReady());
                panic!("nodejs error");    
            }
            println!("Child process exited with status: {}", status);
            println!("nodejs_thread stopping");
        }
    })
}

