//use anyhow::Result;
use serde::{Deserialize, Serialize};
use regex::Regex;
use crate::utils::helpers::{
    app_message, 
    //screen_center
};

//SETTINGS
fn default_as_true() -> bool { true }
fn default_as_false() -> bool { false } //explicit is better
fn default_as_one() -> i32 { 1 }
fn default_as_minus_one() -> i32 { -1 }
fn default_as_float_one() -> f32 { 1.0 }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UIConfig {
    //main_window_x: i32,
    //main_window_y: i32,
    pub main_window_w: i32,
    pub main_window_h: i32,
    pub popup_w: i32,
    pub popup_h: i32,
    pub popup_dict_w: i32,
    pub popup_dict_h: i32,

    pub selected_translator: String,
    pub selected_dict: String,
    pub selected_tts_voice: String,
    pub selected_tts_service: String,
    pub selected_prnn_service: String,

    pub selected_src: String,
    pub selected_target: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub translators: Vec<TranslatorOption>,
    pub dictionaries: Vec<DictOption>,
    pub tts_services: Vec<TTServiceOption>,
    pub prnn_services: Vec<PRNNSourceOption>,

    pub ocr_models: Vec<OCRModelOption>,

    pub download_all_pronunciations: bool,
    pub eng_accents: Vec<String>,

    #[serde(default = "default_as_false")]
    pub use_google_token: bool,
    pub google_translate_api_key: Option<String>,

    pub pinned_src_languages: Vec<String>,
    pub pinned_target_languages: Vec<String>,
    pub switch_target_lang: bool,

    pub ui_lang: String,
    pub ui_font_size: i32,
    pub text_font_size: i32,
    pub win_bg_color: String,
    pub text_bg_color_popup: String,
    pub text_bg_color_main: String,
    pub popup_opacity: f64,
    #[serde(default = "default_as_false")]
    pub no_tooltips: bool,

    pub translate_hotkey: Option<String>,
    pub dict_hotkey: Option<String>,
    pub ocr_hotkey: Option<String>,
    pub single_word_to_dict: bool,

    pub ext_service_unload_timeout: u64,
    pub http_throttling: f64,
    pub http_request_timeout: u64,
    pub openai_api_request_timeout: u64,
    pub openai_api_stream_request_timeout: u64,
    pub proxy: Option<ProxyOption>,
    
    #[serde(default = "default_as_false")]
    pub use_proxy_global: bool,

    pub source_text_max_length: usize, //TODO: chunking

    pub re_str_find: Option<String>,
    pub re_str_replace: Option<String>,
    
    pub source_text_min_length: usize,
    pub dict_request_max_length: usize,

    #[serde(default = "default_as_minus_one")]
    pub history_max_entries: i32,

    #[serde(default = "default_as_true")]
    pub ocr_fullscreen: bool,
    
    #[serde(default = "default_as_true")]
    pub use_db: bool,

    #[serde(default = "default_as_false")]
    pub sqlite_vacuum: bool,

    #[serde(default = "default_as_true")]
    pub qtranslate_autoload: bool,

    #[serde(default = "default_as_false")]
    pub enable_fltk_dpi_scaling: bool,

    pub ui_scaling: f32,

    #[serde(default = "default_as_true")]
    pub tts_button_dropdown: bool,
    #[serde(default = "default_as_true")]
    pub prnn_button_dropdown: bool,

    #[serde(default = "default_as_one")]
    pub copy_button_action: i32,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceType {
    #[default]
    Native,
    Sidecar,
    OpenAI,
    QTranslate,
    DSLDict,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TranslatorOption {
    #[serde(default)]
    pub service_type: ServiceType,
    pub uid: String,
    pub name: String,

    #[serde(default = "default_as_false")]
    pub use_proxy: bool,

    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub reload_if_lang_changed: Option<bool>,
    pub emulation: Option<String>,

    pub openai_url: Option<String>,
    #[serde(default)]
    pub openai_api_key: String,
    pub openai_model: Option<String>,
    pub openai_prompt: Option<String>,
    #[serde(default = "default_as_true")]
    pub stream: bool,
    #[serde(default = "default_as_false")]
    pub api_key_requied: bool,
    #[serde(default)]
    pub api_key_url: String,
    #[serde(default = "default_as_false")]
    pub markdown: bool,
    #[serde(default = "default_as_false")]
    pub cookies: bool,
    #[serde(default = "default_as_false")]
    pub force_split_view: bool,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DictOption {
    #[serde(default)]
    pub service_type: ServiceType,
    pub uid: String,
    pub name: String,

    #[serde(default = "default_as_false")]
    pub use_proxy: bool,

    pub command: Option<String>,
    pub path: Option<String>,
    pub dict_path: Option<String>,
    pub emulation: Option<String>,
    #[serde(default = "default_as_false")]
    pub cookies: bool,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TTServiceOption {
    #[serde(default)]
    pub service_type: ServiceType,
    pub uid: String,
    pub name: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub voices: Vec<String>,

    #[serde(default = "default_as_false")]
    pub use_proxy: bool,
    pub emulation: Option<String>,

    #[serde(default = "default_as_float_one")]
    pub speed: f32,
    pub openai_url: Option<String>,
    #[serde(default)]
    pub openai_api_key: String,
    pub openai_model: Option<String>,
    pub openai_response_format: Option<String>,
    #[serde(default = "default_as_false")]
    pub api_key_requied: bool,
    #[serde(default)]
    pub api_key_url: String,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PRNNSourceOption {
    #[serde(default)]
    pub service_type: ServiceType,
    pub uid: String,
    pub name: String,
    #[serde(default = "default_as_false")]
    pub use_proxy: bool,
    pub emulation: Option<String>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProxyOption {
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct OCRModelOption {
    pub name: String,
    pub det_model: String,
    pub rec_model: String,
    pub charset: String
}

pub mod app_settings {
    use super::*;
    pub fn from_file(file_path: &str) -> Settings {
        let settings_json = std::fs::read_to_string(file_path).unwrap_or_else(|e| {
                app_message("Failed to open settings.json");
                panic!("Error: {}", e);
            });
        let mut settings: Settings = json5::from_str(&settings_json).unwrap_or_else(|e| {
                app_message("Failed to parse settings.json");
                panic!("Error: {}", e);
            });

        if !settings.qtranslate_autoload {
            return settings;
        }
        let re_uid = Regex::new(r"^[\w ]+$").unwrap();
        let mut seen_uids_tr: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut seen_uids_dict: std::collections::HashSet<String> = std::collections::HashSet::new();
        let target_dir = "./extensions/qtranslate/Services"; 
        let path = std::path::Path::new(target_dir);
        if path.exists() && let Ok(paths) = std::fs::read_dir(path) {
            for entry in paths {
                let entry = entry.unwrap();
                let entry_path = entry.path();

                #[allow(clippy::collapsible_if)]
                if entry_path.is_dir() {
                    if let Some(folder_name_os) = entry_path.file_name() {
                        let folder_name = folder_name_os.to_string_lossy().into_owned();
                        let contents = std::fs::read_to_string(entry_path.join("Service.js"));

                        if let Ok(ref c) = contents 
                        && c.contains("Capability.TRANSLATE") 
                        && re_uid.is_match(&folder_name) 
                        && seen_uids_tr.insert(folder_name.clone()) {
                            let new_tr = TranslatorOption {
                                service_type: ServiceType::QTranslate,
                                uid: folder_name.clone(),
                                name: folder_name.clone(),
                                use_proxy: false,
                                command: None,
                                args: None,
                                reload_if_lang_changed: None,
                                emulation: None,

                                openai_url: None,
                                openai_api_key: "".to_string(),
                                openai_model: None,
                                openai_prompt: None,
                                stream: false,
                                api_key_requied: false,
                                api_key_url: "".to_string(),
                                markdown: false,
                                cookies: false,
                                force_split_view: false,
                            };
                            
                            if !settings.translators.iter().any(|item| item.uid == new_tr.uid) {
                                settings.translators.push(new_tr);
                            }
                        }
                        if let Ok(ref c) = contents 
                        && c.contains("Capability.DICTIONARY") 
                        && re_uid.is_match(&folder_name) 
                        && seen_uids_dict.insert(folder_name.clone()) {
                            let new_dict = DictOption {
                                service_type: ServiceType::QTranslate,
                                uid: folder_name.clone(),
                                name: folder_name,
                                use_proxy: false,
                                command: None,
                                path: None,
                                dict_path: None,
                                emulation: None,
                                cookies: false,
                            };
                            
                            if !settings.dictionaries.iter().any(|item| item.uid == new_dict.uid) {
                                settings.dictionaries.push(new_dict);
                            }
                        }
                        //if let Ok(ref c) = contents 
                        //&& c.contains("Capability.LISTEN") 
                        //&& re_uid.is_match(&folder_name) 
                        //&& seen_uids_tts.insert(folder_name.clone()) {
                            /*let new_dict = DictOption {
                                uid: folder_name.clone(),
                                name: folder_name,
                                use_proxy: false,
                                command: Some("QTRANSLATE".to_string()),
                                path: None,
                                dict_path: None,
                                emulation: None,
                            };
                            
                            if !settings.dictionaries.iter().any(|item| item.uid == new_dict.uid) {
                                settings.dictionaries.push(new_dict);
                            }*/
                        //}
                    }
                }
            }
        }
        settings
    }
}

pub mod ui_config {
    use super::*;
    pub fn from_file(file_path: &str) -> UIConfig{
        if !std::path::Path::new(file_path).exists() {
            let config_default = UIConfig {
                main_window_w: 800,
                main_window_h: 600,

                popup_w: 550,
                popup_h: 200,

                popup_dict_w: 450,
                popup_dict_h: 200,

                selected_translator: "".to_string(),
                selected_dict: "".to_string(),
                selected_tts_voice: "".to_string(),
                selected_tts_service: "".to_string(),
                selected_prnn_service: "".to_string(),

                selected_src: "".to_string(),
                selected_target: "".to_string()
            };

            let ui_config = serde_json::to_string(&config_default).unwrap_or_else(|e| {
                app_message("Failed to serialize UICONFIG");
                panic!("Error: {}", e);
            });
            std::fs::write("ui_config.json", ui_config).unwrap_or_else(|e| {
                app_message("Failed to write ui_config.json");
                panic!("Error: {}", e);
            });
        }

        let ui_config_json = std::fs::read_to_string("ui_config.json").unwrap_or_else(|e| {
            app_message("Failed to open ui_config.json");
            panic!("Error: {}", e);
        });
        let ui_config: UIConfig = json5::from_str(&ui_config_json).unwrap_or_else(|e| {
            app_message("Failed to parse ui_config.json");
            panic!("Error: {}", e);
        });
        ui_config
    }
}
