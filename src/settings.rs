//use std::fmt::Display;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use regex::Regex;
use crate::utils::helpers::{
    app_message, 
    //screen_center
};
//use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use std::time::{SystemTime, UNIX_EPOCH};

//SETTINGS
fn default_as_true() -> bool { true }
fn default_as_false() -> bool { false } //explicit is better
fn default_as_one() -> i32 { 1 }
fn default_as_minus_one() -> i32 { -1 }
fn default_as_float_one() -> f32 { 1.0 }
fn default_prompt() -> String { ".\\prompts\\default.txt".to_string()}

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



impl Default for Settings {
    fn default() -> Self {
        Self {
            translators: vec![],
            dictionaries: vec![],
            tts_services: vec![],
            prnn_services: vec![],
            ocr_models: vec![],
            download_all_pronunciations: false,
            eng_accents: vec![],

            use_google_token: false,
            google_translate_api_key: "".to_string(),

            pinned_src_languages: vec![],
            pinned_target_languages: vec![],
            switch_target_lang: true,

            ui_lang: "en".to_string(),
            ui_font_size: 12,
            text_font_size: 14,
            win_bg_color: "#C0C0C0".to_string(),
            text_bg_color_popup: "#F0F0F0".to_string(),
            text_bg_color_main: "#FFFFFF".to_string(),
            popup_opacity: 1.0,
            no_tooltips: false,
            translate_hotkey: "ctrl+KeyQ".to_string(),
            dict_hotkey: "ctrl+shift+KeyQ".to_string(),
            ocr_hotkey: "ctrl+Backquote".to_string(),
            single_word_to_dict: false,

            ext_service_unload_timeout: 120,
            http_throttling: 1.5,
            http_request_timeout: 30,
            openai_api_request_timeout: 60,
            openai_api_stream_request_timeout: 300,
            proxy: ProxyOption { url: "".to_string(), username: "".to_string(), password: "".to_string() },
            use_proxy_global: false,

            source_text_max_length: 4500,

            re_str_find: "".to_string(),
            re_str_replace: "".to_string(),
            
            source_text_min_length: 3,
            dict_request_max_length: 100,

            history_max_entries: -1,
            ocr_fullscreen: true,
            use_db: true,
            sqlite_vacuum: false,
            qtranslate_autoload: true,
            enable_fltk_dpi_scaling: false,

            ui_scaling: 1.0,
            tts_button_dropdown: true,
            prnn_button_dropdown: true,

            copy_button_action: 1,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
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
    pub google_translate_api_key: String,

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

    #[serde(default)]
    pub translate_hotkey: String,
    #[serde(default)]
    pub dict_hotkey: String,
    #[serde(default)]
    pub ocr_hotkey: String,
    pub single_word_to_dict: bool,

    pub ext_service_unload_timeout: u64,
    pub http_throttling: f64,
    pub http_request_timeout: u64,
    pub openai_api_request_timeout: u64,
    pub openai_api_stream_request_timeout: u64,
    pub proxy: ProxyOption,
    
    #[serde(default = "default_as_false")]
    pub use_proxy_global: bool,

    pub source_text_max_length: usize, //TODO: chunking

    #[serde(default)]
    pub re_str_find: String,
    #[serde(default)]
    pub re_str_replace: String,
    
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

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, EnumIter)]
#[serde(rename_all = "lowercase")]
pub enum ServiceType {
    #[default]
    Native,
    Sidecar,
    OpenAI,
    QTranslate,
    DSLDict,
}
impl ServiceType {
    pub fn from_str(s: &str) -> ServiceType {
        match s {
            "native" => ServiceType::Native,
            "sidecar" => ServiceType::Sidecar,
            "openai" => ServiceType::OpenAI,
            "qtranslate" => ServiceType::QTranslate,
            "dsldict" => ServiceType::DSLDict,
            _ => ServiceType::Native
        }
    }
}
impl std::fmt::Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_str = match self {
            ServiceType::Native => "native",
            ServiceType::Sidecar => "sidecar",
            ServiceType::OpenAI => "openai",
            ServiceType::QTranslate => "qtranslate",
            ServiceType::DSLDict => "dsldict",
        };
        write!(f, "{}", type_str)
    }
}

pub trait Named {
    fn name(&self) -> &str;
    fn uid(&self) -> &str;
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TranslatorOption {
    #[serde(default)]
    pub service_type: ServiceType,
    pub uid: String,
    pub name: String,
    #[serde(default = "default_as_true")]
    pub enabled: bool,

    #[serde(default = "default_as_false")]
    pub use_proxy: bool,

    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub reload_if_lang_changed: bool,
    #[serde(default)]
    pub emulation: String,

    #[serde(default)]
    pub openai_url: String,
    #[serde(default)]
    pub openai_api_key: String,
    #[serde(default)]
    pub openai_model: String,
    #[serde(default = "default_prompt")]
    pub openai_prompt: String,
    #[serde(default = "default_as_true")]
    pub stream: bool,
    #[serde(default)]
    pub openai_extra: serde_json::Map<String, serde_json::Value>,
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

    #[serde(default = "default_as_true")]
    pub supports_language_detection: bool,
}
impl Named for TranslatorOption {
    fn name(&self) -> &str {
        &self.name
    }
    fn uid(&self) -> &str {
        &self.uid
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DictOption {
    #[serde(default)]
    pub service_type: ServiceType,
    pub uid: String,
    pub name: String,
    #[serde(default = "default_as_true")]
    pub enabled: bool,
    
    #[serde(default = "default_as_false")]
    pub use_proxy: bool,

    #[serde(default)]
    pub command: String, //todo?
    #[serde(default)]
    pub path: String, //todo?
    #[serde(default)]
    pub dict_path: String,
    #[serde(default)]
    pub emulation: String,
    #[serde(default = "default_as_false")]
    pub cookies: bool,
}
impl Named for DictOption {
    fn name(&self) -> &str {
        &self.name
    }
    fn uid(&self) -> &str {
        &self.uid
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TTServiceOption {
    #[serde(default)]
    pub service_type: ServiceType,
    pub uid: String,
    pub name: String,
    #[serde(default = "default_as_true")]
    pub enabled: bool,
    
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub voices: Vec<String>,

    #[serde(default = "default_as_false")]
    pub use_proxy: bool,
    #[serde(default)]
    pub emulation: String,

    #[serde(default = "default_as_float_one")]
    pub speed: f32,
    #[serde(default)]
    pub openai_url: String,
    #[serde(default)]
    pub openai_api_key: String,
    #[serde(default)]
    pub openai_model: String,
    #[serde(default)]
    pub openai_response_format: String,
    #[serde(default)]
    pub openai_extra: serde_json::Map<String, serde_json::Value>,
    #[serde(default = "default_as_false")]
    pub api_key_requied: bool,
    #[serde(default)]
    pub api_key_url: String,
}
impl Named for TTServiceOption {
    fn name(&self) -> &str {
        &self.name
    }
    fn uid(&self) -> &str {
        &self.uid
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PRNNSourceOption {
    #[serde(default)]
    pub service_type: ServiceType,
    pub uid: String,
    pub name: String,
    #[serde(default = "default_as_true")]
    pub enabled: bool,

    #[serde(default = "default_as_false")]
    pub use_proxy: bool,
    #[serde(default)]
    pub emulation: String,
}
impl Named for PRNNSourceOption {
    fn name(&self) -> &str {
        &self.name
    }
    fn uid(&self) -> &str {
        &self.uid
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProxyOption {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OCRModelOption {
    pub uid: String,
    pub name: String,
    pub det_model: String,
    pub rec_model: String,
    pub charset: String
}
impl Named for OCRModelOption {
    fn name(&self) -> &str {
        &self.name
    }
    fn uid(&self) -> &str {
        &self.uid
    }
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
                                enabled: true,
                                use_proxy: false,
                                command: "".to_string(),
                                args: vec![],
                                reload_if_lang_changed: false,
                                emulation: "".to_string(),

                                openai_url: "".to_string(),
                                openai_api_key: "".to_string(),
                                openai_model: "".to_string(),
                                openai_prompt: "".to_string(),
                                stream: false,
                                openai_extra: serde_json::Map::new(),
                                api_key_requied: false,
                                api_key_url: "".to_string(),
                                markdown: false,
                                cookies: false,
                                force_split_view: false,

                                supports_language_detection: true,
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
                                enabled: true,
                                use_proxy: false,
                                command: "".to_string(),
                                path: "".to_string(),
                                dict_path: "".to_string(),
                                emulation: "".to_string(),
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

    pub fn make_backup() -> Result<()> {
        let settings_path = std::path::Path::new("./settings.json");
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let (year, month, day) = crate::utils::helpers::date_from_duration(now);
        let ts = now.as_millis() % 1000;
        let first_path = settings_path.with_file_name(format!("settings_{:04}-{:02}-{:02}.backup_{}.json", year, month, day, ts));

        crate::utils::helpers::try_copy_exclusive(settings_path, &first_path)
    }

    pub fn write(s: Settings) -> Result<()> {
        if std::path::Path::new("./settings.json").exists() {
            let settings = serde_json::to_string_pretty(&s).map_err(|_| {
                app_message("Failed to serialize GLOBAL_SETTINGS");
                anyhow!("Failed to serialize GLOBAL_SETTINGS")
            })?;
            std::fs::write("./settings.json", settings).map_err(|_| {
                app_message("Failed to write settings.json");
                anyhow!("Failed to write settings.json")
            })?;
            Ok(())
        } else {
            Err(anyhow!("Error: settings.json is not exist"))
        }
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

    pub fn write(config: UIConfig) {
        if std::path::Path::new("./ui_config.json").exists() {
            let ui_config = serde_json::to_string_pretty(&config).unwrap_or_else(|e| {
                app_message("Failed to serialize UICONFIG");
                panic!("Error: {}", e);
            });
            std::fs::write("./ui_config.json", ui_config).unwrap_or_else(|e| {
                app_message("Failed to write ui_config.json");
                panic!("Error: {}", e);
            });
        }    
    }
}
