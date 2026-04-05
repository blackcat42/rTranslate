use fltk::{
    app,
};

//TODO: rewrite, split into separate modules and traits, remove is_dict flags, etc...

use rusqlite::{params, Connection, ToSql};
const SEED: u32 = 42;

use std::collections::HashMap;
use std::convert::AsRef;
use std::str::FromStr;

use fancy_regex::Regex;
use twox_hash::XxHash32;
use anyhow::{anyhow, Result};
use unicode_truncate::UnicodeTruncateStr;

use crate::types::{
    AppEvent, 
    Lang, 
    Translator, 
    Dictionary, 
    TTSEngine, 
    PRNNService, 
    UIState, 
    UIStateDict, 
    TTSource,
    PRNNSource,
    TranslSource,
};

use super::GLOBAL_SETTINGS;

pub struct AppState {
    pub app_sender: fltk::app::Sender<AppEvent>,

    pub src_id: i64, //todo u32
    pub src_text: String,
    pub src_text_dict: String,

    pub selected_translator: String,
    pub selected_dict: String,
    pub selected_tts_voice: String,
    pub selected_tts_engine: String,
    pub selected_prnn_source: String,

    pub selected_src: Lang,
    pub selected_target: Lang,

    pub translators: HashMap<String, Box<dyn Translator>>,
    pub dictionaries: HashMap<String, Box<dyn Dictionary>>,
    pub tts_engines: HashMap<String, Box<dyn TTSEngine>>,
    pub prnn_sources: HashMap<String, Box<dyn PRNNService>>,

    pub db: Option<Connection>,
}

impl AppState {

    pub fn update_fav_browser(&mut self) -> Result<()> {
        let db_ref = &self.db;
        if !GLOBAL_SETTINGS.use_db || db_ref.is_none() {
            return Ok(());
        }

        if let Some(db) = db_ref {
            let mut data_pr = db.prepare(
                "SELECT id, text FROM src WHERE src.fav = TRUE ORDER BY id DESC LIMIT 50", //TODO: limit from settings
            )?;

            let data = data_pr.query_map([], |row| {
                Ok(TranslSource {
                    id: row.get(0)?,
                    text: row.get(1)?,
                })
            })?;

            let mut transl_arr: Vec<TranslSource> = Vec::new();

            for item in data {
                let item = item?;
                let mut row = item.text.chars().take(55).collect::<String>().replace("\n", " ");
                if row.chars().count() > 50 {
                    row.push_str("...");
                }
                transl_arr.push(TranslSource {id: item.id, text: row});
            }

            //dbg!(&transl_arr);
            self.app_sender.send(AppEvent::UpdateFavBrowserView(transl_arr));
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn update_history_browser(&mut self) -> Result<()> {
        let db_ref = &self.db;
        if !GLOBAL_SETTINGS.use_db || db_ref.is_none() {
            return Ok(());
        }

        if let Some(db) = db_ref {
            let mut data_pr = db.prepare(
                "SELECT id, text FROM src ORDER BY id DESC LIMIT 50", //TODO: limit from settings
            )?;

            let data = data_pr.query_map([], |row| {
                Ok(TranslSource {
                    id: row.get(0)?,
                    text: row.get(1)?,
                })
            })?;

            let mut transl_arr: Vec<TranslSource> = Vec::new();

            for item in data {
                let item = item?;
                let mut row = item.text.chars().take(55).collect::<String>().replace("\n", " ");
                if row.chars().count() > 50 {
                    row.push_str("...");
                }
                transl_arr.push(TranslSource {id: item.id, text: row});
            }

            self.app_sender.send(AppEvent::UpdateHistoryBrowserView(transl_arr));
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn update_tts_list(&mut self, src_id: i32) -> Result<()> {
        let db_ref = &self.db;
        if !GLOBAL_SETTINGS.use_db || db_ref.is_none() {
            return Ok(());
        }
        
        if let Some(db) = db_ref {
            let src_text: String = db.query_row(
                "SELECT src.id, src.text FROM src 
                 WHERE src.id = ?1",
                params![src_id],
                |row| {
                    row.get(1)
                },
            )?;

            let mut data_pr = db.prepare(
                "SELECT path, tts_engine_uid, tts_voice_uid FROM tts
                 WHERE src_id = :id"
            )?;
            let mut data_pr_prnn = db.prepare(
                "SELECT path, prnn_source_uid FROM prnn
                 WHERE src_id = :id"
            )?;

            let data = data_pr.query_map(&[(":id", &src_id)], |row| {
                Ok(TTSource {
                    path: row.get(0)?,
                    engine: row.get(1)?,
                    voice: row.get(2)?,
                })
            })?;
            let data_prnn = data_pr_prnn.query_map(&[(":id", &src_id)], |row| {
                Ok(PRNNSource {
                    path: row.get(0)?,
                    service: row.get(1)?,
                })
            })?;

            let mut tts_arr: Vec<TTSource> = Vec::new();
            for item in data {
                let item = item?;
                tts_arr.push(item);
            }
            let mut prnn_arr: Vec<PRNNSource> = Vec::new();
            for item in data_prnn {
                let item = item?;
                prnn_arr.push(item);
            }

            self.app_sender.send(AppEvent::UpdateTTSBrowser(src_text, tts_arr, prnn_arr));
        };
        Ok(())
    }

    pub fn set_src_text(&mut self, text: &str, is_dict: bool) -> Result<()> {

        self.app_sender.send(AppEvent::ClearUi(is_dict));
        let (text, src_id, is_fav) = self.insert_src(text)?;
        self.src_id = src_id;

        if is_dict {
            self.src_text_dict = text;
            let state = UIStateDict {
                src_id: None,
                src_text_dict: self.src_text_dict.clone(),
                dict_uid: None,
                dict_name: None,
                //src: src_lang.clone(), 
                //target: target_lang.clone(), 
                dict_text: None,
                is_fav: Some(is_fav),
            };
            self.app_sender.send(AppEvent::UpdateUiDict(state, true));
        } else {
            self.src_text = text;
            let state = UIState {
                src_text: self.src_text.clone(),
                tr_uid: None,
                translator: None, 
                src: None, 
                target: None, 
                translation_text: None,
                is_fav: Some(is_fav)
            };
            self.app_sender.send(AppEvent::UpdateUi(state, true));
        }
        

        Ok(())
    }

    pub fn translate(&mut self, fail_if_not_exist: bool, force: bool) -> Result<()> {

        self.app_sender.send(AppEvent::ClearUi(false));

        //let lang_detect = isolang::Language::from_639_3(lang_detect_result.three_letter_code()).unwrap().to_639_1().unwrap();
        if !fail_if_not_exist {
            self.app_sender.send(AppEvent::SetWaiting(Some("translate".to_string()), false));
        }
        let selected_text = self.src_text.clone();
        if selected_text.chars().count() < GLOBAL_SETTINGS.transl_request_min_length {
            return Err(anyhow!("source text is too short"));
        }
        //let (selected_text, src_id, is_fav) = self.insert_src(selected_text.as_str())?; //TODO: remove this call; id and is_fav to self props

        let mut src_lang = self.selected_src.clone();
        let mut target_lang = self.selected_target.clone();

        let mut is_lang_detected = true;
        if GLOBAL_SETTINGS.lang_autodetect {
            //DETECT LANGUAGE
            let info = whatlang::detect(selected_text.as_str()).ok_or(anyhow!("whatlang error"))?;
            //println!("{:?}", info.lang().code()); 
            //println!("{:?}", info.is_reliable());
            if info.is_reliable() {
                let lng = Lang::from_str(info.lang().code()).unwrap_or(Lang::En);
                if lng == target_lang {
                    target_lang = src_lang;
                    src_lang = lng;
                } else {
                    src_lang = lng;
                }
                
                let qwe = format!("Language detected as {}", info.lang().eng_name());
                self.app_sender.send(AppEvent::SetStatus(qwe.into_boxed_str(), false, false));
                is_lang_detected = true;
            } else {
                is_lang_detected = false;
                //TODO!: if not forced, check cache for any existing translation; get last entry

                //self.app_sender.send(AppEvent::SetStatus("selected text is too short to detect the language".into(), false, false));
            }

            /*if selected_text.chars().count() > 55 {
                let lang_detect_result = whichlang::detect_language(selected_text.clone().as_str());
                src_lang = Lang::from_str(lang_detect_result.three_letter_code()).unwrap_or(Lang::En);
                //src_lang = lng;

                let qwe = format!("src lang detected as {}", lang_detect_result.three_letter_code());
                self.app_sender.send(AppEvent::SetStatus(qwe.into_boxed_str()));
            } else {
                self.app_sender.send(AppEvent::SetStatus("selected text is too short to detect the language; previously selected lang-pair was used".into()));
            }*/
        }

        

        //self.app_sender.send(AppEvent::SetWaiting());

        let cached_transl = if force {
            None
        } else {
            self.check_transl_cache(
                self.src_id, 
                &self.selected_translator, 
                src_lang.as_ref(), 
                target_lang.as_ref()
            )
        };

        match cached_transl {
            Some(t) => {
                let tr_uid = self.selected_translator.clone();
                let mut tr_name = "".to_string();
                if let Some(tr) = self.translators.get(tr_uid.as_str()) {
                    tr_name = tr.get_name();
                }

                self.app_sender.send(AppEvent::UpdateUi(UIState {
                    src_text: selected_text,
                    tr_uid: Some(tr_uid), 
                    translator: Some(tr_name), 
                    src: Some(src_lang.clone()), 
                    target: Some(target_lang.clone()), 
                    translation_text: Some(t),
                    is_fav: None
                }, false));
            }
            None => {
                if fail_if_not_exist {
                    //self.app_sender.send(AppEvent::SetStatus("no cached results".into(), true, false));
                    return Err(anyhow!("no cached results"));
                } else if let Some(translator) = self.translators.get_mut(self.selected_translator.as_str()) {
                    translator.translate(
                        self.src_id, 
                        selected_text, 
                        src_lang.clone(), 
                        target_lang.clone(),
                        is_lang_detected
                    );
                } else {
                    //self.app_sender.send(AppEvent::SetStatus("selected translation service is not exist".into(), true));
                    return Err(anyhow!("selected translation service is not exist"));
                }
            }
        };

        Ok(())
    }

    pub fn request_dict_entry(&mut self, fail_if_not_exist: bool, force: bool) -> Result<()> {
        
        self.app_sender.send(AppEvent::ClearUi(true));
        //let lang_detect = isolang::Language::from_639_3(lang_detect_result.three_letter_code()).unwrap().to_639_1().unwrap();
        if !fail_if_not_exist {
            self.app_sender.send(AppEvent::SetWaiting(None, true));
        }
        
        let selected_text = self.src_text_dict.clone();
        if selected_text.chars().count() > GLOBAL_SETTINGS.dict_request_max_length {
            return Err(anyhow!("source text is too long"));
        }
        //let (selected_text, src_id, is_fav) = self.insert_src(selected_text.as_str())?;

        let mut src_lang = self.selected_src.clone();
        let mut target_lang = self.selected_target.clone();

        if GLOBAL_SETTINGS.lang_autodetect {
            let info = whatlang::detect(selected_text.as_str()).ok_or(anyhow!("whatlang error"))?;
            //println!("{:?}", info.lang().code()); println!("{:?}", info.is_reliable());
            if info.is_reliable() {
                let lng = Lang::from_str(info.lang().code()).unwrap_or(Lang::En);
                if lng == target_lang {
                    target_lang = src_lang;
                    src_lang = lng;
                } else {
                    src_lang = lng;
                }

                //let lng = Lang::from_str(info.lang().code()).unwrap_or(Lang::En);
                //self.selected_src = lng;

                let qwe = format!("Language detected as {}", info.lang().code());
                self.app_sender.send(AppEvent::SetStatus(qwe.into_boxed_str(), false, true));
            } else {
                self.app_sender.send(AppEvent::SetStatus("selected text is too short to detect the language".into(), false, true));
            }
        }


        let dict_entry = if force {
            None
        } else {
            self.check_dict_cache(self.src_id, &self.selected_dict)
        };

        match dict_entry {
            Some(t) => {
                let dict_uid = self.selected_dict.clone();
                let mut dict_name = "".to_string();
                if let Some(d) = self.dictionaries.get(dict_uid.as_str()) {
                    dict_name = d.get_name();
                }

                self.app_sender.send(AppEvent::UpdateUiDict(UIStateDict {
                    src_id: Some(self.src_id),
                    src_text_dict: selected_text,
                    dict_uid: Some(dict_uid), 
                    dict_name: Some(dict_name), 
                    //src: src_lang.clone(), 
                    //target: target_lang.clone(), 
                    dict_text: Some(t),
                    is_fav: None,
                }, false));
            }
            None => {
                if fail_if_not_exist && !force {
                    //self.app_sender.send(AppEvent::SetStatus("no cached results".into(), true, true));
                    return Err(anyhow!("no cached results for selected dictionary"));
                } else if let Some(dict) = self.dictionaries.get_mut(self.selected_dict.as_str()) {
                    dict.translate(
                        self.src_id,
                        selected_text,
                        src_lang.clone(),
                        target_lang.clone()
                    );
                } else {
                    return Err(anyhow!("selected dict service is not exist"));
                }
            }
        };

        Ok(())
    }
    
    pub fn run_tts(&mut self) -> Result<()> {
        //let text = self.src_text.clone();
        let (text, src_id, is_fav) = self.insert_src(&self.src_text)?;
        let tts_file = self.check_tts_cache(src_id, &self.selected_tts_engine, &self.selected_tts_voice);
        //15_kkr_af-heart.ogg

        match tts_file {
            Ok(tr) => {
                let filename = format!("{}.ogg", tr);
                self.app_sender.send(AppEvent::TTSPlay(filename));
            }
            Err(_) => {
                if text.chars().count() < 2 {
                    self.app_sender.send(AppEvent::SetStatus("TTS error: text is too short".into(), false, false));
                    return Err(anyhow!("TTS error: text is too short"));
                }

                //self.set_waiting();
                self.app_sender.send(AppEvent::SetWaiting(None, false));
                if let Some(engine) = self.tts_engines.get_mut(self.selected_tts_engine.clone().as_str()) {
                    let a = engine.generate(
                        text.clone(), 
                        src_id, 
                        self.selected_tts_voice.clone()
                    );
                } else {
                    println!("error");
                }
            }
        }
        app::awake();
        Ok(())
    }
    
    pub fn check_tts_cache(&self, src_id: i64, selected_tts_engine: &str, selected_tts_voice: &str) -> Result<String> {
        let db_ref = &self.db;
        if !GLOBAL_SETTINGS.use_db || db_ref.is_none() {
            return Err(anyhow!("db support is off"));
        }
        if let Some(db) = db_ref {
            let tts = db.query_row(
                "SELECT path FROM tts 
                 WHERE src_id = ?1 AND tts_engine_uid = ?2 AND tts_voice_uid = ?3",
                params![src_id, selected_tts_engine, selected_tts_voice],
                |row| {
                    let text = row.get(0)?;
                    Ok(text)
                },
            );

            match tts {
                Ok(t) => {
                    println!("tts found");
                    let audio_path = format!(r"tts_cache\{t}.ogg");
                    let working_dir = std::env::current_dir()?;
                    match working_dir.join(audio_path).try_exists() {
                        Ok(true) => Ok(t),
                        Ok(false) => Err(anyhow!("tts not found (fs erroe)")),
                        Err(_e) => Err(anyhow!("tts not found")),
                    }
                }
                Err(_) => {
                    Err(anyhow!("tts db-entry not found"))
                }
            }
        } else {
            Err(anyhow!("db"))
        }
    }

    pub fn run_prnn(&mut self, index: i32) -> Result<()> {
        //let text = self.src_text_dict.clone();
        let (text, src_id, is_fav) = self.insert_src(&self.src_text_dict)?;
        let tts_file = self.check_prnn_cache(src_id, &self.selected_prnn_source, index);

        match tts_file {
            Ok(tr) => {
                self.app_sender.send(AppEvent::TTSPlay(tr));
            }
            Err(_) => {
                if text.chars().count() < 1 {
                    self.app_sender.send(AppEvent::SetStatus("TTS error: text is too short".into(), false, true));
                    return Err(anyhow!("TTS error: text is too short"));
                }

                //self.set_waiting();
                self.app_sender.send(AppEvent::SetWaiting(None, true));
                if let Some(engine) = self.prnn_sources.get_mut(self.selected_prnn_source.clone().as_str()) {
                    let _ = engine.generate(
                        text.clone(), 
                        src_id, 
                    );
                } else {
                    println!("error");
                }
            }
        }
        app::awake();
        Ok(())
    }

    pub fn check_prnn_cache(&self, src_id: i64, selected_prnn_source: &str, index: i32) -> Result<String> {
        let db_ref = &self.db;
        if !GLOBAL_SETTINGS.use_db || db_ref.is_none() {
            return Err(anyhow!("db support is off"));
        }

        if let Some(db) = db_ref {
            let mut data_pr_prnn = db.prepare(
                "SELECT path, prnn_source_uid FROM prnn
                 WHERE src_id = :id AND prnn_source_uid = :prnn_src_uid"
            )?;
            let data_prnn = data_pr_prnn.query_map(&[(":id", &src_id as &dyn ToSql), (":prnn_src_uid", &selected_prnn_source as &dyn ToSql)], |row| {
                Ok(PRNNSource {
                    path: row.get(0)?,
                    service: row.get(1)?,
                })
            })?;
            let mut prnn_arr: Vec<PRNNSource> = Vec::new();
            for item in data_prnn {
                let item = item?;
                prnn_arr.push(item);
            }
            let index = if index < 0 {
                0_usize
            } else {
                index as usize
            };
            if prnn_arr.is_empty() {
                return Err(anyhow!("no pronunciation found"));
            }
            let current_index = index % prnn_arr.len();
            let tts = prnn_arr.get(current_index).ok_or(anyhow!("error"))?;
            let tts = tts.path.clone();
            let audio_path = format!(r"tts_cache\{tts}");
            let working_dir = std::env::current_dir()?;
            match working_dir.join(audio_path).try_exists() {
                Ok(true) => Ok(tts),
                Ok(false) => Err(anyhow!("pronunciation not found (fs error)")),
                Err(e) => Err(e.into()),
            }
        } else {
            Err(anyhow!("db"))
        }
    }
    
    pub fn check_transl_cache(&self, src_id: i64, selected_translator: &str, src_lang: &str, target_lang: &str) -> Option<String> {
        let db_ref = &self.db;
        if !GLOBAL_SETTINGS.use_db || db_ref.is_none() {
            return None;
        }
        if let Some(db) = db_ref {
            let transl = db.query_row(
                "SELECT text FROM transl 
                 WHERE src_id = ?1 AND transl_engine_uid = ?2 AND src = ?3 AND target = ?4",
                params![src_id, selected_translator, src_lang, target_lang],
                |row| {
                    let text = row.get(0)?;
                    Ok(text)
                },
            );

            match transl {
                Ok(t) => {
                    Some(t)
                }
                Err(_) => {
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn check_dict_cache(&self, src_id: i64, selected_dict: &str) -> Option<String> {
        let db_ref = &self.db;
        if !GLOBAL_SETTINGS.use_db || db_ref.is_none() {
            return None;
        }
        if let Some(db) = db_ref {
            let transl = db.query_row(
                "SELECT text FROM dict 
                 WHERE src_id = ?1 AND dict_uid = ?2",
                params![src_id, selected_dict],
                |row| {
                    let text = row.get(0)?;
                    Ok(text)
                },
            );

            match transl {
                Ok(t) => {
                    Some(t)
                }
                Err(_) => {
                    None
                }
            }
        } else {
            None
        }
    } 

    pub fn insert_tts(&self, src_id: i64, tts_engine: &str, tts_voice: &str, filename: &str) -> Result<String> {
        let db_ref = &self.db;
        if !GLOBAL_SETTINGS.use_db || db_ref.is_none() {
            return Ok(filename.to_string());
        }
        if let Some(db) = db_ref {
            let zxc = src_id.clone().to_string();
            let path = format!(r"{zxc}_{tts_engine}_{tts_voice}");
            db.execute(
                "REPLACE INTO tts (src_id, path, tts_engine_uid, tts_voice_uid) VALUES (?1, ?2, ?3, ?4)",
                params![src_id, path, tts_engine, tts_voice],
            )?;
            println!("tts inserted/replaced");
            //Ok(db.last_insert_rowid())//TODO: RETURNING clause
            Ok(filename.to_string())
        } else {
            Ok(filename.to_string())
        }
    }
    pub fn insert_prnn(&self, src_id: i64, prnn_source: &str, filename: &str) -> Result<String> {
        let db_ref = &self.db;
        if !GLOBAL_SETTINGS.use_db || db_ref.is_none() {
            return Ok(filename.to_string());
        }
        if let Some(db) = db_ref {
            db.execute(
                "REPLACE INTO prnn (src_id, path, prnn_source_uid) VALUES (?1, ?2, ?3)",
                params![src_id, filename, prnn_source],
            )?;
            println!("prnn inserted/replaced");
            //Ok(db.last_insert_rowid())//TODO: RETURNING clause
            Ok(filename.to_string())
        } else {
            Ok(filename.to_string())
        }
    }

    pub fn insert_transl(&self, src_id: i64, selected_translator: &str, src: &str, target: &str, text: &str) -> Result<i64> {
        let db_ref = &self.db;
        if !GLOBAL_SETTINGS.use_db || db_ref.is_none() {
            return Ok(0);
        }
        if let Some(db) = db_ref {
            db.execute(
                "REPLACE INTO transl (src_id, transl_engine_uid, src, target, text) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![src_id, selected_translator, src, target, text],
            )?;
            println!("transl inserted/replaced");
            Ok(db.last_insert_rowid()) //TODO: RETURNING clause
        } else {
            Ok(0)
        }
    }
    pub fn insert_dict_entry(&self, src_id: i64, selected_dict: &str, text: &str) -> Result<i64> {
        let db_ref = &self.db;
        if !GLOBAL_SETTINGS.use_db || db_ref.is_none() {
            return Ok(0);
        }
        if let Some(db) = db_ref {
            //let selected_translator = selected_translator;
            //let src = src.as_ref();
            //let target = target.as_ref();
            db.execute(
                "REPLACE INTO dict (src_id, dict_uid, text) VALUES (?1, ?2, ?3)",
                params![src_id, selected_dict, text],
            )?;
            println!("dict inserted/replaced");
            Ok(db.last_insert_rowid())//TODO: RETURNING clause
        } else {
            Ok(0)
        }
    }

    pub fn insert_src(&self, text: &str) -> Result<(String, i64, bool)> {
        let text = text.replace("\r\n", "\n");

        //remove hardcoded line-breaks
        //TODO more accurate??? Optionable
        //TODO custom regex?
        let request_limit = GLOBAL_SETTINGS.source_text_max_length; //TODO: chunking, statusbar
        let re = Regex::new(r"\n(?=[a-z])")?;
        let text = re.replace_all(&text, "").to_string();
        let text = text.unicode_truncate(request_limit).0.trim().to_string();

        if text.chars().count() < 1 {
            return Err(anyhow!("source text is too short"));
        }

        let db_ref = &self.db;
        if !GLOBAL_SETTINGS.use_db || db_ref.is_none() {
            return Ok((text, 0, false));
        }

        if let Some(db) = db_ref {
            let hash = XxHash32::oneshot(SEED, text.as_bytes());

            let src_id = db.query_row(
                "SELECT id, fav FROM src 
                 WHERE hash = ?1 AND text = ?2",
                params![hash, text],
                |row| {
                    let id = row.get(0)?;
                    let fav = row.get(1)?;
                    Ok((id, fav))
                },
            );

            match src_id {
                Ok((id, fav)) => {
                    println!("cached src found");
                    Ok((text, id, fav))
                }
                Err(_) => {
                    db.execute(
                        "INSERT INTO src (text, hash) VALUES (?1, ?2)",
                        params![text, hash],
                    )?;
                    println!("src inserted");
                    Ok((text, db.last_insert_rowid(), false)) //TODO: RETURNING clause
                }
            } 
        } else {
            Ok((text, 0, false))
        }
    }

    pub fn toggle_fav(&self, text: &str, is_dict: bool) -> Result<()> {
        //let mut src_id = 0;
        /*if let Some(t) = text {
            src_id = self.insert_src(t.as_str())?.0;
        } else {
            src_id = self.insert_src(self.src_text.as_str())?.0;
        }*/
        let src_id = self.insert_src(text)?.1;

        let db_ref = &self.db;
        if !GLOBAL_SETTINGS.use_db || db_ref.is_none() {
            return Ok(());
        }

        if let Some(db) = db_ref {
            let is_fav: bool = db.query_row(
                "SELECT fav FROM src 
                 WHERE id = ?1",
                params![src_id],
                |row| {
                    let text = row.get(0)?;
                    Ok(text)
                },
            ).unwrap_or(false);

            db.execute(
                "UPDATE src SET fav = ?2 WHERE id = ?1",
                params![src_id, !is_fav],
            )?;

            if !is_dict {
                let state = UIState {
                    src_text: text.to_string(),
                    tr_uid: None,
                    translator: None, 
                    src: None, 
                    target: None, 
                    translation_text: None,
                    is_fav: Some(!is_fav)
                };
                self.app_sender.send(AppEvent::UpdateUi(state, false));
            } else {
                let state = UIStateDict {
                    src_id: None,
                    src_text_dict: text.to_string(),
                    dict_uid: None,
                    dict_name: None,
                    //src: src_lang.clone(), 
                    //target: target_lang.clone(), 
                    dict_text: None,
                    is_fav: Some(!is_fav),
                };
                self.app_sender.send(AppEvent::UpdateUiDict(state, false));
            }
            
            //self.app_sender.send(AppEvent::SetUiFavState(!is_fav));

            Ok(())
        } else {
            Ok(())
        }
    }


    pub fn init_db(&self) -> Result<()> {
        //println!("open_db");
        let db_ref = &self.db;
        if !GLOBAL_SETTINGS.use_db || db_ref.is_none() {
            self.app_sender.send(AppEvent::SetStatus("no db support, caching will not work".into(), false, false));
            return Ok(());
        }

        if let Some(db) = db_ref {
            db.execute(
                "CREATE TABLE IF NOT EXISTS src (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    hash INTEGER NOT NULL,
                    fav INTEGER DEFAULT 0,
                    text TEXT NOT NULL,
                    comment TEXT
                )",
                (),
            )?;

            db.execute(
                "CREATE INDEX IF NOT EXISTS src_hash_index ON src (hash)",
                (),
            )?;

            db.execute(
                "CREATE TABLE IF NOT EXISTS transl (
                    src_id INTEGER NOT NULL,
                    text TEXT NOT NULL,
                    src TEXT NOT NULL,
                    target TEXT NOT NULL,
                    transl_engine_uid TEXT NOT NULL,
                    PRIMARY KEY (src_id, transl_engine_uid, src, target)
                    FOREIGN KEY (src_id) REFERENCES src (id) ON DELETE CASCADE
                )",
                (),
            )?;

            db.execute(
                "CREATE TABLE IF NOT EXISTS tts (
                    src_id INTEGER NOT NULL,
                    path TEXT NOT NULL,
                    tts_engine_uid TEXT NOT NULL,
                    tts_voice_uid TEXT NOT NULL,
                    PRIMARY KEY (src_id, tts_engine_uid, tts_voice_uid)
                    FOREIGN KEY (src_id) REFERENCES src (id) ON DELETE CASCADE
                )",
                (),
            )?;

            db.execute(
                "CREATE TABLE IF NOT EXISTS prnn (
                    src_id INTEGER NOT NULL,
                    path TEXT NOT NULL,
                    prnn_source_uid TEXT NOT NULL,
                    PRIMARY KEY (src_id, path, prnn_source_uid)
                    FOREIGN KEY (src_id) REFERENCES src (id) ON DELETE CASCADE
                )",
                (),
            )?;

            db.execute(
                "CREATE TABLE IF NOT EXISTS dict (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    text TEXT NOT NULL,
                    dict_uid TEXT NOT NULL,
                    src_id INTEGER NOT NULL REFERENCES src(id) ON DELETE CASCADE
                )",
                (),
            )?;
        };
        Ok(())
    }

}