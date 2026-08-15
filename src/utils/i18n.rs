use std::sync::{LazyLock};
//use anyhow::{anyhow, Result};
use crate::utils::helpers::{
    app_message, 
};
use serde::{Deserialize, Serialize};
use sys_locale::get_locale;
use super::GLOBAL_SETTINGS;

#[derive(Debug, Deserialize, Serialize)]
pub struct Locale {
    //TRAY MENU
    pub screen_ocr: String,
    pub show_popup: String,
    pub show_popup_dict: String,
    pub settings: String,
    pub exit: String,

    //MAIN WINDOW
    pub from: String,
    pub to: String,

    pub translate_refresh: String,
    pub send_to_dict: String,
    pub play: String,
    pub download: String,
    pub add_to_fav: String,
    pub remove_from_fav: String,

    pub recent_history: String,
    pub favorites: String,

    pub source_text: String,
    pub translation: String,
    pub dictionary_entry: String,
    pub prnn_cached: String,
    pub tts_cached: String,

    pub translate_with: String,
    pub dictionary: String,
    pub tts_engine_voice: String,
    pub pronunciation: String,

    //POPUP WINDOW
    pub close: String,
    //"add_to_fav", 
    //"remove_from_fav",
    pub refresh: String,
    pub lang: String,
    pub tts: String,
    //"send_to_dict",
    pub open_main_win: String,
}
impl Default for Locale {
    fn default() -> Self {
        Self {
            screen_ocr: "Screen OCR".to_string(),
            show_popup: "Show popup window".to_string(),
            show_popup_dict: "Show dict. popup window".to_string(),
            settings: "Settings".to_string(),
            exit: "Exit".to_string(),
            from: "From".to_string(),
            to: "To".to_string(),
            translate_refresh: "Translate / Refresh".to_string(),
            send_to_dict: "Send to dictionary".to_string(),
            play: "Play".to_string(),
            download: "Download".to_string(),
            add_to_fav: "Add to favorites".to_string(),
            remove_from_fav: "Remove from fav.".to_string(),
            recent_history: "Recent history".to_string(),
            favorites: "Favorites".to_string(),
            source_text: "Source (editable)".to_string(),
            translation: "Translation".to_string(),
            dictionary_entry: "Dictionary entry".to_string(),
            prnn_cached: "Pronunciations (cached)".to_string(),
            tts_cached: "TTS (cached)".to_string(),
            translate_with: "Translate with".to_string(),
            dictionary: "Dictionary".to_string(),
            tts_engine_voice: "TTS (engine-voice)".to_string(),
            pronunciation: "Pronunciation".to_string(),
            close: "Close".to_string(),
            refresh: "Refresh".to_string(),
            lang: "Languages".to_string(),
            tts: "TTS".to_string(),
            open_main_win: "Open main window".to_string(),
        }
    }
}


pub static LOCALIZATION: LazyLock<Locale> = LazyLock::new(|| {

    let locale = &GLOBAL_SETTINGS.ui_lang;
    //let locale = get_locale().unwrap_or_else(|| "en-US".to_string());
    //let locale: String = locale.chars().take(2).collect();

    let file = format!("./i18n/{}.json", locale);
    if !std::path::Path::new(&file).exists() {
        return Locale::default();
    }

    let l_json = std::fs::read_to_string(&file);

    if let Ok(l_json) = l_json {
        let l10n: Locale = json5::from_str(&l_json).unwrap_or_else(|e| {
            app_message(&format!("Failed to parse {}: {}", &file, e));
            Locale::default()
        });
        l10n
    } else {
        app_message(&format!("Failed to open {}", &file));
        Locale::default()
    }
});

#[macro_export]
macro_rules! t {
    ($key:ident) => {
        &crate::utils::i18n::LOCALIZATION.$key
    };
}
