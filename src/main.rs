#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#![allow(clippy::get_first)]

use fltk::{
    app,
    prelude::*,
};

use std::collections::HashMap;
use std::env;
use std::fs::File; 
use std::io::BufReader;
use std::rc::Rc;
use std::cell::RefCell;

use anyhow::{anyhow, Result};

use serde::{Deserialize, Serialize};
use regex::Regex;

use get_selected_text::get_selected_text; //todo: high ram usage

use global_hotkey::{
    GlobalHotKeyManager, 
    GlobalHotKeyEvent, 
    HotKeyState, 
    hotkey::{HotKey}
    //hotkey::{HotKey, Modifiers, Code}
};

use std::str::FromStr;

use rusqlite::{params, params_from_iter, Connection};

use tray_icon::{
    menu::{
        Menu, MenuItem,
    },
    TrayIconBuilder, TrayIconEvent,
};
/*use tray_icon::{
    menu::{
        AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem
    },
    TrayIcon, TrayIconBuilder, TrayIconEvent, TrayIconEventReceiver,
};*/
//use anyhow::{anyhow};

mod nodejs_translator;
mod google_translate;
mod google_translate2;
mod wiktionary_en;
mod prnn_wiki;
mod nodejs_tts;
mod user_dict;
mod types;
mod bbcode;
mod app_state;
mod app_view;
use types::{AppEvent, UIState, UIStateDict, TranslSource, LangNames};
use app_state::{AppState};
use app_view::{AppView};
use std::sync::{LazyLock};
use std::sync::{OnceLock};



//SETTINGS
fn default_as_true() -> bool { true }
fn default_as_false() -> bool { false } //explicit is better
fn default_as_minus_one() -> i32 { -1 }

#[derive(Debug, Deserialize, Serialize)]
struct Settings {
    pub translators: Vec<TranslatorOption>,
    pub dictionaries: Vec<DictOption>,
    pub tts_engines: Vec<TTSEngineOption>,
    pub prnn_sources: Vec<PRNNSourceOption>,

    pub google_translate_api_key: Option<String>,

    pub src_language: String,
    pub target_language: String,

    pub default_translator: String,
    pub default_dict: String,

    pub ui_font_size: i32,
    pub win_bg_color: String,
    pub text_bg_color: String,
    pub popup_opacity: f64,

    pub translate_hotkey: Option<String>,
    pub dict_hotkey: Option<String>,
    pub single_word_to_dict: bool,

    pub nodejs_unload_timeout: u64,
    pub http_throttling: f64,
    pub http_request_timeout: u64,

    pub source_text_max_length: usize, //TODO: chunking
    pub transl_request_min_length: usize,
    pub dict_request_max_length: usize,

    #[serde(default = "default_as_minus_one")]
    pub history_max_entries: i32,
    #[serde(default = "default_as_false")]
    pub clear_audio_cache_at_startup: bool,
    #[serde(default = "default_as_minus_one")]
    pub audio_max_entries: i32,

    pub lang_autodetect: bool,

    #[serde(default = "default_as_true")]
    pub use_db: bool,
    #[serde(default = "default_as_true")]
    pub use_db_dict: bool,
}
#[derive(Debug, Deserialize, Serialize)]
struct TranslatorOption {
    pub uid: String,
    pub name: String,
    pub path: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
struct DictOption {
    pub uid: String,
    pub name: String,
    pub path: Option<String>,
    pub dict_path: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
struct TTSEngineOption {
    pub uid: String,
    pub name: String,
    pub path: Option<String>,
    pub voices: Vec<String>
}
#[derive(Debug, Deserialize, Serialize)]
struct PRNNSourceOption {
    pub uid: String,
    pub name: String,
}

static GLOBAL_SETTINGS: LazyLock<Settings> = LazyLock::new(|| {
    if !std::path::Path::new("settings.json").exists() {
        std::fs::copy("settings.json.default", "settings.json").unwrap_or_else(|e| {
            app_panic_message("Failed to open settings.json");
            panic!("Error: {}", e);
        });
    }

    let settings_json = std::fs::read_to_string("settings.json").unwrap_or_else(|e| {
            app_panic_message("Failed to open settings.json");
            panic!("Error: {}", e);
        });
    let settings: Settings = json5::from_str(&settings_json).unwrap_or_else(|e| {
            app_panic_message("Failed to parse settings.json");
            panic!("Error: {}", e);
        });
    settings
});

static TOKIO_RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn main() {

    let app = app::App::default().with_scheme(app::Scheme::Base);

    //COLORS
    //app::set_frame_color();
    //app::set_background2_color(); //Set the background color for input and text widgets
    let str_color = &GLOBAL_SETTINGS.win_bg_color.strip_prefix('#').unwrap_or(&GLOBAL_SETTINGS.win_bg_color);
    if let Ok(bg_color) = u32::from_str_radix(str_color, 16) {
        let bg_color = fltk::utils::hex2rgb(bg_color);
        app::set_background_color(bg_color.0, bg_color.1, bg_color.2);
    } else {
        app::set_background_color(214, 207, 198);
    };

    //PATHS
    let working_dir = std::env::current_dir().unwrap_or_else(|e| {
            app_panic_message("current_dir");
            panic!("Error: {}", e);
        });

    //TRAY
    let tray_menu = Menu::new();

    let tray_menu_main_window = MenuItem::new("rTranslate", true, None); //TODO: make bold
    if let Err(err) = tray_menu.append(&tray_menu_main_window) {
        println!("{err:?}");
    }
    let tray_menu_popup_window = MenuItem::new("Show popup window", true, None);
    if let Err(err) = tray_menu.append(&tray_menu_popup_window) {
        println!("{err:?}");
    }
    let tray_menu_popup_dict_window = MenuItem::new("Show dict. popup window", true, None);
    if let Err(err) = tray_menu.append(&tray_menu_popup_dict_window) {
        println!("{err:?}");
    }

    let tray_menu_exit = MenuItem::new("Exit", true, None);
    if let Err(err) = tray_menu.append(&tray_menu_exit) {
        println!("{err:?}");
    }

    let icon = load_icon(working_dir.join(r"icons\tray_icon.png").as_path());
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_menu_on_left_click(false)
        .with_tooltip("rTranslate")
        .with_icon(icon)
        .build()
        .unwrap_or_else(|e| {
            app_panic_message("Tray icon builder error");
            panic!("Error: {}", e);
        });

    let conn = if GLOBAL_SETTINGS.use_db {
        Connection::open("history.db").ok()
    } else {
        None
    };
    let conn_dict = if GLOBAL_SETTINGS.use_db_dict {
        Connection::open("dictionary_index.db").ok()
    } else {
        None
    };
    let conn_dict_wrapper = Rc::new(RefCell::new(conn_dict));

    if GLOBAL_SETTINGS.history_max_entries >= 0 && conn.is_some() {
        let _ = clear_history(&conn);
    }
    if GLOBAL_SETTINGS.clear_audio_cache_at_startup && conn.is_some() {
        let _ = clear_audio_cache(&conn);
    }
    //HOTKEYS
    let manager = GlobalHotKeyManager::new().unwrap_or_else(|e| {
        app_panic_message("GlobalHotKeyManager");
        panic!("Error: {}", e);
    });
    let tr_hotkey_id: Option<u32> = if let Some(translate_hotkey) = &GLOBAL_SETTINGS.translate_hotkey {
        if let Ok(hotkey) = translate_hotkey.parse::<HotKey>() {
            let _ = manager.register(hotkey);
            Some(hotkey.id())
        } else {
            app_panic_message("Failed to parse translate hotkey");
            None
        }
    } else {
        None
    };
    let dict_hotkey_id: Option<u32> = if let Some(dict_hotkey) = &GLOBAL_SETTINGS.dict_hotkey {
        if let Ok(hotkey_dict) = dict_hotkey.parse::<HotKey>() {
            let _ = manager.register(hotkey_dict);
            Some(hotkey_dict.id())
        } else {
            app_panic_message("Failed to parse dict hotkey");
            None
        }
    } else {
        None
    };

    let (app_sender, app_receiver) = app::channel::<AppEvent>();

    let mut app_state = AppState {
        app_sender,
        src_id: 0,
        src_text: "".to_string(),
        src_text_dict: "".to_string(),
        selected_translator: "".to_string(),
        selected_dict: "".to_string(),
        selected_tts_voice: "".to_string(),
        selected_tts_engine: "".to_string(),
        selected_prnn_source: "".to_string(),
        selected_src: types::Lang::from_str(&GLOBAL_SETTINGS.src_language).unwrap_or(types::Lang::En),
        selected_target: types::Lang::from_str(&GLOBAL_SETTINGS.target_language).unwrap_or(types::Lang::Ru),

        db: conn,

        translators: HashMap::new(),
        dictionaries: HashMap::new(),
        tts_engines: HashMap::new(),
        prnn_sources: HashMap::new(),
    };
    let _ = app_state.init_db();

    let mut app_view = AppView::new(app_sender);

    let re_uid = Regex::new(r"^\w+$").unwrap();
    for value in GLOBAL_SETTINGS.translators.iter() {
        if !re_uid.is_match(&value.uid) {
            app_panic_message("settings.json: Failed to parse uid");
            panic!("Error");
        }
        if let Some(path) = &value.path && path.chars().count() > 0 {
            app_state.translators.insert(value.uid.clone(), Box::new(nodejs_translator::NT::new(app_sender, value.uid.clone(), value.name.clone(), path.clone())));
        } else if value.uid == "tr_google" {
            app_state.translators.insert(value.uid.clone(), Box::new(google_translate::GT::new(app_sender, value.name.clone(), value.uid.clone())));
        } else if value.uid == "tr_google2" {
            app_state.translators.insert(value.uid.clone(), Box::new(google_translate2::GT::new(app_sender, value.name.clone(), value.uid.clone())));
        }
    }
    //app_state.translators.entry(String::from("tr_google")).or_insert_with(|| Box::new(google_translate::GT::new(app_sender)));

    for value in GLOBAL_SETTINGS.dictionaries.iter() {
        if !re_uid.is_match(&value.uid) {
            app_panic_message("settings.json: Failed to parse uid");
            panic!("Error");
        }
        if let Some(dict_path) = &value.dict_path && dict_path.chars().count() > 0 {
            let conn_dict_clone = Rc::clone(&conn_dict_wrapper);
            app_state.dictionaries.insert(value.uid.clone(), Box::new(user_dict::DSLDict::new(app_sender, value.uid.clone(), value.name.clone(), dict_path.clone(), conn_dict_clone)));
        } else if value.uid == "dict_wiktionary_en" {
            app_state.dictionaries.insert(value.uid.clone(), Box::new(wiktionary_en::WDEn::new(app_sender, value.name.clone(), value.uid.clone())));
        }
    }
    //app_state.dictionaries.entry(String::from("dict_wiktionary_en")).or_insert_with(|| Box::new(wiktionary_en::WDEn::new(app_sender)));

    for value in GLOBAL_SETTINGS.tts_engines.iter() {
        if !re_uid.is_match(&value.uid) {
            app_panic_message("settings.json: Failed to parse uid");
            panic!("Error");
        }

        if let Some(path) = &value.path && path.chars().count() > 0 {
            dbg!(value);
            app_state.tts_engines.insert(value.uid.clone(), Box::new(nodejs_tts::NTTS::new(app_sender, value.uid.clone(), value.name.clone(), path.clone())));
        }
    }
    
    for value in GLOBAL_SETTINGS.prnn_sources.iter() {
        /*if let Some(path) = &value.path && path.chars().count() > 0 {
            //
        } else*/ 
        if value.uid == "prnn_wiki" {
            app_state.prnn_sources.insert(value.uid.clone(), Box::new(prnn_wiki::WP::new(app_sender, value.name.clone())));
        }
    }
    //app_state.prnn_sources.entry(String::from("prnn_wiki")).or_insert_with(|| Box::new(prnn_wiki::WP::new(app_sender, "Wiktionary Pronunciations".to_string())));

    let _ = app_state.update_history_browser();
    let _ = app_state.update_fav_browser();



    app_view.set_lang(
        LangNames::from_str(app_state.selected_src.as_ref()).unwrap_or(LangNames::En).as_ref(), 
        LangNames::from_str(app_state.selected_target.as_ref()).unwrap_or(LangNames::En).as_ref()
    );
    app_sender.send(AppEvent::SetTranslator(GLOBAL_SETTINGS.default_translator.clone()));
    app_sender.send(AppEvent::SetDict(GLOBAL_SETTINGS.default_dict.clone()));
    app_sender.send(AppEvent::SetTTSEngine(
        GLOBAL_SETTINGS.tts_engines.get(0).unwrap_or_else(|| {
                app_panic_message("Failed to parse selected_tts_engine");
                panic!("Error");
            }).uid.clone(),
        GLOBAL_SETTINGS.tts_engines.get(0).unwrap_or_else(|| {
            app_panic_message("Failed to parse selected_tts_voice");
            panic!("Error");
        }).voices.get(0).unwrap_or_else(|| {
            app_panic_message("Failed to parse selected_tts_voice");
            panic!("Error");
        }).clone()
    ));
    app_sender.send(AppEvent::SetPRNNEngine(
        GLOBAL_SETTINGS.prnn_sources.get(0).unwrap_or_else(|| {
            app_panic_message("Failed to parse selected_prnn_source");
            panic!("Error");
        }).uid.clone()
    ));

    //HOTKEYS
    //TODO: github.com/iholston/win-hotkeys; github.com/obv-mikhail/InputBot
    std::thread::spawn(move || loop {
        //println!("hotkeys event loop");
        if let Ok(event) = GlobalHotKeyEvent::receiver().recv() { 
            app_sender.send(AppEvent::HotKey(event));
        }
    });
    /*
    while app.wait() {
        if app::event() == fltk::enums::Event::KeyUp {
            let key = app::event_key();
            match key {
                fltk::enums::Key::Left => println!("ArrowLeft"),
                _ => ()
            };
            //handle_key(key);
        }
    }*/



    while app.wait() {
        //println!("app main loop");
        match app_receiver.recv() {

            //TODO: get ui_state from database by given src- and translation id's (single source of truth)
            //or use global object (?) if db is not supported
            Some(AppEvent::UpdateUi(state, is_new_source)) => {
                app_view.update_ui(state, is_new_source);
            }
            Some(AppEvent::UpdateUiDict(state, is_new_source)) => {
                app_view.update_ui_dict(state, is_new_source);
            }
            Some(AppEvent::ClearUi(is_dict)) => {
                app_view.clear_ui(is_dict);
            }

            Some(AppEvent::SetWaiting()) => {
                app_view.set_waiting();
            }
            Some(AppEvent::SetReady()) => {
                app_view.set_ready();
            }
            Some(AppEvent::UpdateHistoryBrowserView(state)) => {
                app_view.update_history_browser(state);
            }
            Some(AppEvent::UpdateFavBrowserView(state)) => {
                app_view.update_fav_browser(state);
            }
            Some(AppEvent::UpdateTTSBrowser(src_text, tts_arr, prnn_arr)) => {
                let _ = app_state.set_src_text(&src_text, false);
                let _ = app_state.set_src_text(&src_text, true);
                let _ = app_state.translate(true, false);
                let _ = app_state.request_dict_entry(true, false);
                app_view.set_tts_browser_data(tts_arr);
                app_view.set_dict_assets_browser_data(prnn_arr);
            }
            Some(AppEvent::UpdateTTState(id)) => {
                if let Err(e) = app_state.update_tts_list(id) {
                    app_view.set_status(e.to_string().as_str(), true, false);
                };
            }
            Some(AppEvent::SetSrcLang(lng)) => {
                app_state.selected_src = lng;
                //TODO: update views
            }
            Some(AppEvent::SetTargetLang(lng)) => {
                app_state.selected_target = lng;
            }
            Some(AppEvent::SetTranslator(translator)) => {
                app_state.selected_translator = translator.clone();
                if let Some(tr_struct) = app_state.translators.get(translator.as_str()) {
                    let tr_name = tr_struct.get_name();
                    app_view.set_translator(&tr_name, translator.as_str());
                }
            }
            Some(AppEvent::SetDict(dict)) => {
                app_state.selected_dict = dict.clone();
                if let Some(dict_struct) = app_state.dictionaries.get(dict.as_str()) {
                    let dict_name = dict_struct.get_name();
                    app_view.set_dict(&dict_name, dict.as_str());
                }
            }
            Some(AppEvent::SetTTSEngine(tts, voice)) => {
                app_state.selected_tts_engine = tts.clone();
                app_state.selected_tts_voice = voice.clone();

                if let Some(tts_struct) = app_state.tts_engines.get(tts.as_str()) {
                    let tts_name = tts_struct.get_name();
                    app_view.set_tts_engine(&tts_name, &voice);
                }
                
            }
            Some(AppEvent::SetPRNNEngine(prnn)) => {
                app_state.selected_prnn_source = prnn.clone();

                if let Some(prnn_struct) = app_state.prnn_sources.get(prnn.as_str()) {
                    let prnn_name = prnn_struct.get_name();
                    app_view.set_prnn_service(&prnn_name);
                }
                
            }

            Some(AppEvent::ToggleFav(text, is_dict)) => {
                //let _ = app_state.set_fav(text);
                match text {
                    Some(t) => {
                        let _ = app_state.toggle_fav(&t, is_dict);
                    },
                    None => {
                        if is_dict {
                            let _ = app_state.toggle_fav(&app_view.src_dict, is_dict);
                        } else {
                            let _ = app_state.toggle_fav(&app_view.src, is_dict);
                        }
                    }
                };
                let _ = app_state.update_fav_browser();
            }

            Some(AppEvent::SaveTranslation((src_id, _src_text, translator, src, target, translation_text))) => {
                let _ = app_state.insert_transl(src_id, translator.as_str(), src.as_ref(), target.as_ref(), translation_text.as_str());
                let _ = app_state.update_history_browser();
                
            }
            Some(AppEvent::SaveDictEntry((src_id, _src_text, dict, dict_text))) => {
                let _ = app_state.insert_dict_entry(src_id, dict.as_str(), dict_text.as_str());
                let _ = app_state.update_history_browser(); 
            }
            Some(AppEvent::SavePRNN((src_id, prnn_source_uid, filename))) => {
                let _ = app_state.insert_prnn(src_id, &prnn_source_uid, &filename);
                //let _ = app_state.update_browser(); 
            }

            Some(AppEvent::SetStatus(text, is_error, is_dict)) => {
                app_view.set_status(&text, is_error, is_dict);
            }

            Some(AppEvent::TrayIcon(e)) => {
                match e {
                    TrayIconEvent::DoubleClick{..} => {
                        app_view.main_win.show();
                    }
                    TrayIconEvent::Click{..} => {
                        //...
                    }
                    _ =>  {}
                }
            }
            Some(AppEvent::TrayMenuEvent(e)) => {
                println!("{:?}", e);
                let exit_id = tray_menu_exit.id();
                let main_window_id = tray_menu_main_window.id();

                if e.id == exit_id {
                    std::process::exit(0);
                } else if e.id == main_window_id {
                    app_view.main_win.show();
                } else if e.id == tray_menu_popup_window.id() {
                    app_view.show_popup(false, false);
                } else if e.id ==  tray_menu_popup_dict_window.id() {
                    app_view.show_popup(true, false);
                }
            }
            Some(AppEvent::HotKey(e)) => 'hotkey_arm: {
                //dbg!(e);
                if e.state == HotKeyState::Released {
                    let mut is_dict: bool;
                    if Some(e.id) == tr_hotkey_id {
                        is_dict = false;
                    } else if Some(e.id) == dict_hotkey_id {
                        is_dict = true;
                    } else {
                        break 'hotkey_arm;
                    }
                    match get_selected_text() {
                        Ok(selected_text) => {
                            if !selected_text.trim().contains(' ') 
                               && !is_dict 
                               && GLOBAL_SETTINGS.single_word_to_dict {
                                is_dict = true;
                            }
                            if let Err(set_src_error) = app_state.set_src_text(&selected_text, is_dict) {
                                app_view.set_status(set_src_error.to_string().as_str(), true, is_dict);
                            } else {
                                app_view.show_popup(is_dict, true);
                                //app_view.clear_ui(is_dict); //clear status, title and translation buffer

                                if !is_dict {
                                    if let Err(tr_error) = app_state.translate(false, false) {
                                        app_sender.send(AppEvent::SetReady());
                                        app_view.set_status(tr_error.to_string().as_str(), true, false);
                                    }
                                } else if let Err(dict_error) = app_state.request_dict_entry(false, false) {
                                    app_sender.send(AppEvent::SetReady());
                                    app_view.set_status(dict_error.to_string().as_str(), true, true);
                                }
                            }
                        },
                        Err(_) => {
                            app_view.set_status("An error occurred while getting the selected text", true, false);
                            println!("An error occurred while getting the selected text");
                        }
                    }
                }
            }
            Some(AppEvent::Translate(fail_if_not_exist, force, check_buf)) => 'translate_arm: {
                //app_view.clear_ui(false);
                if check_buf && (app_view.src_buf.text() != app_view.src) {
                    if let Err(set_src_error) = app_state.set_src_text(&app_view.src_buf.text(), false) {
                        app_view.set_status(set_src_error.to_string().as_str(), true, false);
                        break 'translate_arm;
                    }
                }
                if let Err(error) = app_state.translate(fail_if_not_exist, force) {
                    app_sender.send(AppEvent::SetReady());
                    app_sender.send(AppEvent::SetStatus(error.to_string().as_str().into(), false, false));
                    //app_view.set_status(error.to_string().as_str(), true, false);
                }
            }
            Some(AppEvent::RequestDictEntry(fail_if_not_exist, force, check_buf)) => 'request_dict_arm: {
                //app_view.clear_ui(true);
                if check_buf && (app_view.src_buf.text() != app_view.src_dict) { //only src_buf in main_window is editable
                    if let Err(set_src_error) = app_state.set_src_text(&app_view.src_dict_buf.text(), true) {
                        app_view.set_status(set_src_error.to_string().as_str(), true, true);
                        break 'request_dict_arm;
                    }
                }
                if let Err(error) = app_state.request_dict_entry(fail_if_not_exist, force) {
                    app_sender.send(AppEvent::SetReady());
                    app_sender.send(AppEvent::SetStatus(error.to_string().as_str().into(), false, true));
                    //app_view.set_status(error.to_string().as_str(), true, true);
                }
            }
            Some(AppEvent::SendToDict()) => {
                //app_view.clear_ui(true);
                if let Err(set_src_error) = app_state.set_src_text(&app_view.src, true) {
                    app_sender.send(AppEvent::SetReady());
                    app_view.set_status(set_src_error.to_string().as_str(), true, true);
                } else if let Err(dict_error) = app_state.request_dict_entry(false, false) {
                    app_sender.send(AppEvent::SetReady());
                    app_view.set_status(dict_error.to_string().as_str(), true, true);
                }
            }

            Some(AppEvent::TTString()) => {
                let _ = app_state.run_tts();
            }
            Some(AppEvent::PRNNString()) => {
                let _ = app_state.run_prnn();
            }
            Some(AppEvent::TTSPlay(filename)) => {
                app_view.set_ready();

                let audio_path = format!(r"tts_cache\{filename}.ogg");

                std::thread::spawn({
                    let working_dir = env::current_dir().expect("current_dir");
                    let audio_path = working_dir.join(audio_path);

                    move || {
                        let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
                            .expect("open default audio stream");
                        let file = BufReader::new(File::open(audio_path)
                            .expect("BufReader"));
                        // Note that the playback stops when the sink is dropped
                        let sink = rodio::play(stream_handle.mixer(), file)
                            .expect("rodio::play");
                        sink.sleep_until_end();
                        //TODO: statusbar
                    }
                });
                app::awake();
            }
            _other =>  {
            }
        }


    }

    // #[cfg(not(target_os = "windows"))]
    // app.run().unwrap();
}




fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
    //TODO fallback
    let (icon_rgba, icon_width, icon_height) = {
        let image1 = ::image::open(path)
            .unwrap_or_else(|e| {
                app_panic_message("Failed to open icon path");
                panic!("Error: {}", e);
                })
            .into_rgba8();
        let (width, height) = image1.dimensions();
        let rgba = image1.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap_or_else(|e| {
        app_panic_message("Failed to load tray icon");
        panic!("Error: {}", e);
    })
}



fn app_panic_message(e: &str) {
    let pos = screen_center();
    fltk::dialog::alert(pos.0 - 210, pos.1 - 40, e);
    
}
pub fn screen_center() -> (i32, i32) {
    (
        (app::screen_size().0 / 2.0) as i32,
        (app::screen_size().1 / 2.0) as i32,
    )
}

fn clear_audio_cache(conn: &Option<Connection>) -> Result<()> {
    let audio_max_entries = GLOBAL_SETTINGS.audio_max_entries;
    if !GLOBAL_SETTINGS.clear_audio_cache_at_startup || audio_max_entries < 0 {
        return Ok(());
    }
    if let Some(db) = conn {
        let count: i32 = db.query_row(
            "SELECT COUNT(*) FROM tts 
             INNER JOIN src 
             ON tts.src_id = src.id 
             WHERE src.fav = FALSE",
            params![],
            |row| row.get(0),
        )?;
        if audio_max_entries >= count {
            return Ok(());
        }
        let limit_del = count - audio_max_entries;
    
        let mut data_pr = db.prepare(
            "SELECT tts.src_id, tts.path FROM tts 
             INNER JOIN src 
             ON tts.src_id = src.id 
             WHERE src.fav = FALSE 
             ORDER BY tts.src_id ASC 
             LIMIT :limit
             
            "
        )?;

        let mut ids: Vec<u32> = vec![];
        let mut data = data_pr.query(&[(":limit", &limit_del)])?;

        while let Some(row) = data.next()? {
            let id: u32 = row.get(0)?;
            let path: String = row.get(1)?;
            //println!("ID: {}", &id);
            //println!("PATH: {}", &path);
            ids.push(id);
        }
        let _ = delete_audio_files_by_ids(conn, ids);
    }
    Ok(())
}


fn clear_history(conn: &Option<Connection>) -> Result<()> {
    
    let history_max_entries = GLOBAL_SETTINGS.history_max_entries;
    if let Some(db) = conn {
        //println!("clear_history");

        let count: i32 = db.query_row(
            "SELECT COUNT(*) FROM src WHERE src.fav = FALSE",
            params![],
            |row| row.get(0),
        )?;
        if history_max_entries >= count || history_max_entries < 0 {
            return Ok(());
        }

        let limit_del = count - history_max_entries;
        //println!("limit_del {}", limit_del);

        let mut data_pr = db.prepare(
            "SELECT id FROM src 
             WHERE src.fav = FALSE ORDER BY id ASC LIMIT :limit"
        )?;

        let mut ids: Vec<u32> = vec![];
        let mut data = data_pr.query(&[(":limit", &limit_del)])?;
        while let Some(row) = data.next()? {
            let id: u32 = row.get(0)?;
            ids.push(id);
        }

        let _ = delete_audio_files_by_ids(conn, ids.clone());

        let placeholders: String = std::iter::repeat_n("?", ids.len()).collect::<Vec<_>>().join(",");
        let query = format!("DELETE FROM src WHERE id IN ({})", placeholders);
        let rows = db.execute(
            &query,
            params_from_iter(ids.iter()),
        )?;
        println!("clear_history; rows deleted {}", rows);

        Ok(())

        /*let rows = db.execute(
            "DELETE FROM src 
             WHERE id IN (
               SELECT id 
               FROM src 
               WHERE src.fav = FALSE 
               ORDER BY id ASC 
               LIMIT ?1
             )
            ",
            params![limit_del],
        )?;*/           
    } else {
        Err(anyhow!("db"))
    }
}


fn delete_audio_files_by_ids(conn: &Option<Connection>, ids: Vec<u32>) -> Result<()> {
    let placeholders: String = std::iter::repeat_n("?", ids.len()).collect::<Vec<_>>().join(",");
    if let Some(db) = conn {
        if GLOBAL_SETTINGS.clear_audio_cache_at_startup {
            let paths: Vec<Option<String>> = db
                .prepare(&format!("SELECT path FROM tts WHERE src_id IN ({})", placeholders))?
                .query_map(params_from_iter(ids.iter()), |row| row.get(0))?
                .map(|path| path.ok())
                .collect();

            for path in paths.into_iter().flatten() {
                let audio_path = format!(r"tts_cache\{path}.ogg");
                let working_dir = std::env::current_dir()?;
                let file = working_dir.join(&audio_path);
                if let Ok(exist) = file.try_exists() && exist {
                    //println!("File to delete: {}", &file.display());
                    match std::fs::remove_file(&file) {
                        Ok(_) => {
                            println!("File deleted: {}", &file.display());
                        },
                        Err(e) => {
                            eprintln!("Error deleting file: {} ({})", &file.display(), e);
                        }
                    }
                }
            }
            //println!("IDs: {:?}", ids);
            //println!("paths: {:?}", paths);
        }
        Ok(())
    } else {
        Err(anyhow!("db"))
    }
}