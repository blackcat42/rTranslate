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

use rusqlite::{Connection};

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
mod wiktionary_en;
mod prnn_wiki;
mod nodejs_tts;
mod user_dict;
mod types;
mod bbcode;
mod app_state;
mod app_view;
use types::{AppEvent, UIState, UIStateDict};
use app_state::{AppState};
use app_view::{AppView};
use std::sync::{LazyLock};



#[derive(Debug, Deserialize, Serialize)]
struct Settings {
    pub translators: Vec<TranslatorOption>,
    pub dictionaries: Vec<DictOption>,
    pub tts_engines: Vec<TTSEngineOption>,
    pub prnn_sources: Vec<PRNNSourceOption>,

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

    pub nodejs_unload_timeout: u64,
    pub lang_autodetect: bool,
    pub use_db: bool,
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
        src_text: "".to_string(),
        src_text_dict: "".to_string(),
        selected_translator: GLOBAL_SETTINGS.default_translator.clone(),
        selected_dict: GLOBAL_SETTINGS.default_dict.clone(),
        selected_tts_voice: GLOBAL_SETTINGS.tts_engines.get(0).unwrap_or_else(|| {
                app_panic_message("Failed to parse selected_tts_voice");
                panic!("Error");
            }).voices.get(0).unwrap_or_else(|| {
                app_panic_message("Failed to parse selected_tts_voice");
                panic!("Error");
            }).clone(),
        selected_tts_engine: GLOBAL_SETTINGS.tts_engines.get(0).unwrap_or_else(|| {
                app_panic_message("Failed to parse selected_tts_engine");
                panic!("Error");
            }).uid.clone(),
        selected_prnn_source: GLOBAL_SETTINGS.prnn_sources.get(0).unwrap_or_else(|| {
                app_panic_message("Failed to parse selected_prnn_source");
                panic!("Error");
            }).uid.clone(),
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
        }
    }
    app_state.translators.entry(String::from("tr_google")).or_insert_with(|| Box::new(google_translate::GT::new(app_sender)));

    for value in GLOBAL_SETTINGS.dictionaries.iter() {
        if !re_uid.is_match(&value.uid) {
            app_panic_message("settings.json: Failed to parse uid");
            panic!("Error");
        }
        if let Some(dict_path) = &value.dict_path && dict_path.chars().count() > 0 {
            let conn_dict_clone = Rc::clone(&conn_dict_wrapper);
            app_state.dictionaries.insert(value.uid.clone(), Box::new(user_dict::DSLDict::new(app_sender, value.uid.clone(), value.name.clone(), dict_path.clone(), conn_dict_clone)));
        }
    }
    app_state.dictionaries.entry(String::from("dict_wiktionary_en")).or_insert_with(|| Box::new(wiktionary_en::WDEn::new(app_sender)));

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
    
    //for value in GLOBAL_SETTINGS.prnn_sources.iter() {
        /*if value.path.chars().count() > 0 {
            dbg!(value);
            app_state.prnn_sources.insert(value.uid.clone(), Box::new(nodejs_tts::NTTS::new(app_sender, value.uid.clone(), value.name.clone(), value.path.clone())));
        }*/
    //}
    app_state.prnn_sources.entry(String::from("prnn_wiki")).or_insert_with(|| Box::new(prnn_wiki::WP::new(app_sender)));

    let _ = app_state.update_history_browser();
    let _ = app_state.update_fav_browser();

    let lang_from = types::LangNames::from_str(&GLOBAL_SETTINGS.src_language).unwrap_or(types::LangNames::En);
    let lang_to = types::LangNames::from_str(&GLOBAL_SETTINGS.target_language).unwrap_or(types::LangNames::Ru);
    app_view.set_lang(lang_from.as_ref(), lang_to.as_ref());

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
            Some(AppEvent::UpdateUi(state)) => {
                //let UIState {src_text, tr_uid, translator, src, target, translation_text, is_fav} = state;
                app_view.update_ui(state);
            }
            /*Some(AppEvent::UpdateUiSrc(text, is_fav)) => {
                app_view.update_ui_src(text, is_fav);
            }*/
            Some(AppEvent::UpdateUiDict(state)) => {
                //let UIStateDict {src_id, src_text_dict, dict_uid, dict_name, dict_text, is_fav} = state;
                app_view.update_ui_dict(state);
            }
            /*Some(AppEvent::SetUiFavState(f)) => {
                app_view.set_fav(f);
            }*/

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
                app_view.clear_ui(false);
                app_view.clear_ui(true);
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
                app_state.selected_tts_engine = tts;
                app_state.selected_tts_voice = voice;
            }
            Some(AppEvent::SetPRNNEngine(prnn)) => {
                app_state.selected_prnn_source = prnn;
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
            Some(AppEvent::HotKey(e)) => {
                //dbg!(e);
                if e.state == HotKeyState::Released && Some(e.id) == tr_hotkey_id {
                    match get_selected_text() {
                        Ok(selected_text) => {
                            //TODO: if single word --> to dict
                            app_view.show_popup(false, true);
                            app_view.clear_ui(false); //clear status, title and translation buffer

                            //app_view.set_waiting();
                            if let Err(set_src_error) = app_state.set_src_text(&selected_text, false) {
                                app_view.set_ready();
                                app_view.set_status(set_src_error.to_string().as_str(), true, false);
                            } else if let Err(tr_error) = app_state.translate(false, false) {
                                app_view.set_ready();
                                app_view.set_status(tr_error.to_string().as_str(), true, false);
                            }
                        },
                        Err(_) => {
                            app_view.set_status("An error occurred while getting the selected text", true, false);
                            println!("An error occurred while getting the selected text");
                        }
                    }
                } else if e.state == HotKeyState::Released && Some(e.id) == dict_hotkey_id {
                    match get_selected_text() {
                        Ok(selected_text) => {
                            app_view.clear_ui(true);
                            app_view.show_popup(true, true);
                            app_view.set_waiting();
                            if let Err(set_src_error) = app_state.set_src_text(&selected_text, true) {
                                app_view.set_ready();
                                app_view.set_status(set_src_error.to_string().as_str(), true, true);
                            } else if let Err(dict_error) = app_state.request_dict_entry(false, false) {
                                app_view.set_ready();
                                app_view.set_status(dict_error.to_string().as_str(), true, true);
                            }                           
                        },
                        Err(_) => {
                            app_view.set_status("An error occurred while getting the selected text", true, true);
                            println!("An error occurred while getting the selected text");
                        }
                    }
                }
            }
            Some(AppEvent::Translate(force)) => {
                if let Err(error) = app_state.translate(false, force) {
                    println!("{}", error);
                }
            }
            Some(AppEvent::RequestDictEntry(force)) => {
                if let Err(error) = app_state.request_dict_entry(false, force) {
                    println!("{}", error);
                }
            }
            Some(AppEvent::SendToDict()) => {
                app_view.clear_ui(true);

                app_view.set_waiting();
                    if let Err(set_src_error) = app_state.set_src_text(&app_view.src, true) {
                        app_view.set_ready();
                        app_view.set_status(set_src_error.to_string().as_str(), true, true);
                    } else if let Err(dict_error) = app_state.request_dict_entry(false, false) {
                        app_view.set_ready();
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
