//TODO: async, non-blocking

use crate::types::{AppEvent, TTSEngine};
use std::env;
use std::io::Write;
use std::process::{Command, Stdio};
use which::which;

use std::sync::{Arc };
use std::sync::atomic::{AtomicBool, Ordering};

#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub struct NTTS {
    //tx: mpsc::Sender<(String, i64)>,
    //shared_receiver: Arc<Mutex<Receiver<(String, i64)>>>,
    is_running: Arc<AtomicBool>, 
    s: fltk::app::Sender<AppEvent>,
    uid: String,
    name: String,
    entry_point: String,
}

use anyhow::{anyhow, Result};

//TODO: platform-specific
use std::os::windows::process::CommandExt;
const CREATE_NO_WINDOW: u32 = 0x08000000;

impl NTTS {
    pub fn new(s: fltk::app::Sender<AppEvent>, uid: String, name: String, entry_point: String) -> Self {
        //let (tx, rx) = mpsc::channel::<(String, i64)>();
        //let shared_receiver = Arc::new(Mutex::new(rx));
        let is_running = Arc::new(AtomicBool::new(false));
        Self { is_running, s, uid, name, entry_point}
    }
}

impl TTSEngine for NTTS {
    fn generate(&self, text: String, src_id: i64, speaker_uid: String) -> Result<String> {
        if self.is_running.load(Ordering::SeqCst) {
            self.s.send(AppEvent::SetStatus("error: tts service is still running".into(), true, false));
            return Err(anyhow!("tts db-entry not found"));
        }
        let working_dir = env::current_dir().unwrap();
        
        let voice = speaker_uid;
        let mut child;

        let full_path = working_dir.join(self.entry_point.as_str());
        let dir_str = &full_path.parent().unwrap();
        let engine_uid = self.uid.clone();
        let filename = format!("{src_id}_{engine_uid}_{voice}");

        if which(r".\deno").is_ok() {
                child = Command::new(working_dir.join(r".\deno"))
                    .arg("--allow-read=.")
                    .arg("--deny-net")
					.arg("--allow-ffi=.")
					.arg("--allow-env")
					.arg("--allow-write=.")
					.arg("--allow-run=./oggenc2")
                    .arg(&full_path)
                    .arg(format!("--voice={voice}"))
                    .arg(format!("--uid={filename}"))
                    .creation_flags(CREATE_NO_WINDOW)
                    .current_dir(dir_str)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn().expect("Failed to spawn child process");
            } else if which("deno").is_ok() {
                child = Command::new("deno")
                    .arg("--allow-read=.")
                    .arg("--deny-net")
                    .arg("--allow-ffi=.")
                    .arg("--allow-env")
                    .arg("--allow-write=.")
                    .arg("--allow-run=./oggenc2")
                    .arg(&full_path)
                    .arg(format!("--voice={voice}"))
                    .arg(format!("--uid={filename}"))
                    .creation_flags(CREATE_NO_WINDOW)
                    .current_dir(dir_str)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn().expect("Failed to spawn child process");
            }/*TODO: else if which("node").is_ok() {
                child = Command::new("node")
                    .arg(working_dir.join(r"kokoro\index.mjs"))
                    .arg(format!("--kkr-voice={voice2}"))
                    .creation_flags(CREATE_NO_WINDOW)
                    .current_dir(working_dir.join("kokoro"))
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn().expect("Failed to spawn child process");
            }*/ else {
                panic!("");
            }

        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        std::thread::spawn(move || {
            stdin.write_all(text.as_bytes()).expect("Failed to write to stdin");
        });

        child.wait().unwrap();
        Ok(filename)
    }

}
