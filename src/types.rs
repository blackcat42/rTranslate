

use global_hotkey::{
    GlobalHotKeyEvent, 
};
use tray_icon::{
    menu::{
        MenuEvent,
    },
    TrayIconEvent,
};
//use std::str::FromStr;
use strum_macros::AsRefStr;
use strum_macros::EnumString;
use anyhow::Result;

use strum_macros::EnumIter;
//use strum::IntoEnumIterator;


#[derive(Debug)]
#[allow(dead_code)]
pub enum AppEvent {
    HotKey(GlobalHotKeyEvent),
    TrayIcon(TrayIconEvent),
    TrayMenuEvent(MenuEvent),

    //TODO:
    SaveTranslation((i64, String, String, Lang, Lang, String)),
    SaveDictEntry((i64, String, String, String)),
    SavePRNN((i64, String, String)),
    ToggleFav(Option<String>, bool),

    Translate(bool),
    RequestDictEntry(bool),
    SendToDict(),
    TTString(),
    PRNNString(),
    TTSPlay(String),

    SetWaiting(),
    SetReady(),
    SetStatus(Box<str>, bool, bool),
    
    //TODO:
    UpdateUi(UIState),
    //UpdateUiSrc(String, bool),
    UpdateUiDict(UIStateDict),
    //SetUiFavState(bool),


    UpdateHistoryBrowserView(Vec<TranslSource>),
    UpdateFavBrowserView(Vec<TranslSource>),
    UpdateTTSBrowser(String, Vec<TTSource>, Vec<PRNNSource>),

    SetSrcLang(Lang),
    SetTargetLang(Lang),
    SetTranslator(String),
    SetDict(String),
    SetTTSEngine(String, String),
    SetPRNNEngine(String),
    //SetTTSVoice(String),
    UpdateTTState(i32),

}

//#[derive(AsRefStr, Clone)]
// pub enum TranslateServices {
//     Bergamot,
//     Google,
// }

// #[derive(AsRefStr, Clone)]
// pub enum TTService {
//     KokoroAfHeart,
//     KokoroAfNicole,
// }

// #[derive(AsRefStr, Clone)]
// pub enum TTSVoice {
//     AfHeart,
//     AfNicole,
// }


//from ISO 639_3 or ISO 639_1 str:    Lang::from_str
//Lang --> ISO 639_1 str:             .as_ref()
//https://en.wikipedia.org/wiki/List_of_ISO_639_language_codes
/*#[allow(unused)]
#[derive(Debug, AsRefStr, EnumString, Clone, PartialEq, Eq, EnumIter)]
pub enum Lang {
    #[strum(serialize = "eng", to_string="en")]
    En,
    #[strum(serialize = "rus", to_string="ru")]
    Ru,
    #[strum(serialize = "spa", to_string="es")]
    Es,
    #[strum(serialize = "fra", to_string="fr")]
    Fr,
    #[strum(serialize = "jpn", to_string="ja")]
    Ja,

}

#[allow(unreachable_patterns)]
#[derive(AsRefStr, EnumString, Clone, PartialEq, Eq)]
pub enum LangNames {
    #[strum(serialize = "en", to_string="English")]
    En,
    #[strum(serialize = "ru", to_string="Russian")]
    Ru,
    #[strum(serialize = "es", to_string="Spanish")]
    Es,
    #[strum(serialize = "fr", to_string="French")]
    Fr,
    #[strum(serialize = "ja", to_string="Japanese")]
    Ja,

}*/

//TODO: llm-generated, need to verify
#[allow(unused)]
#[derive(Debug, AsRefStr, EnumString, Clone, PartialEq, Eq, EnumIter)]
pub enum Lang {
    #[strum(serialize = "epo", to_string = "eo")] Epo,
    #[strum(serialize = "eng", to_string = "en")] En,
    #[strum(serialize = "rus", to_string = "ru")] Ru,
    #[strum(serialize = "cmn", to_string = "zh")] Zh,
    #[strum(serialize = "spa", to_string = "es")] Es,
    #[strum(serialize = "por", to_string = "pt")] Pt,
    #[strum(serialize = "ita", to_string = "it")] It,
    #[strum(serialize = "ben", to_string = "bn")] Bn,
    #[strum(serialize = "fra", to_string = "fr")] Fr,
    #[strum(serialize = "deu", to_string = "de")] De,
    #[strum(serialize = "ukr", to_string = "uk")] Uk,
    #[strum(serialize = "kat", to_string = "ka")] Ka,
    #[strum(serialize = "ara", to_string = "ar")] Ar,
    #[strum(serialize = "hin", to_string = "hi")] Hi,
    #[strum(serialize = "jpn", to_string = "ja")] Ja,
    #[strum(serialize = "heb", to_string = "he")] He,
    #[strum(serialize = "yid", to_string = "yi")] Yi,
    #[strum(serialize = "pol", to_string = "pl")] Pl,
    #[strum(serialize = "amh", to_string = "am")] Am,
    #[strum(serialize = "jav", to_string = "jv")] Jv,
    #[strum(serialize = "kor", to_string = "ko")] Ko,
    #[strum(serialize = "nob", to_string = "nb")] Nb,
    #[strum(serialize = "dan", to_string = "da")] Da,
    #[strum(serialize = "swe", to_string = "sv")] Sv,
    #[strum(serialize = "fin", to_string = "fi")] Fi,
    #[strum(serialize = "tur", to_string = "tr")] Tr,
    #[strum(serialize = "nld", to_string = "nl")] Nl,
    #[strum(serialize = "hun", to_string = "hu")] Hu,
    #[strum(serialize = "ces", to_string = "cs")] Cs,
    #[strum(serialize = "ell", to_string = "el")] El,
    #[strum(serialize = "bul", to_string = "bg")] Bg,
    #[strum(serialize = "bel", to_string = "be")] Be,
    #[strum(serialize = "mar", to_string = "mr")] Mr,
    #[strum(serialize = "kan", to_string = "kn")] Kn,
    #[strum(serialize = "ron", to_string = "ro")] Ro,
    #[strum(serialize = "slv", to_string = "sl")] Sl,
    #[strum(serialize = "hrv", to_string = "hr")] Hr,
    #[strum(serialize = "srp", to_string = "sr")] Sr,
    #[strum(serialize = "mkd", to_string = "mk")] Mk,
    #[strum(serialize = "lit", to_string = "lt")] Lt,
    #[strum(serialize = "lav", to_string = "lv")] Lv,
    #[strum(serialize = "est", to_string = "et")] Et,
    #[strum(serialize = "tam", to_string = "ta")] Ta,
    #[strum(serialize = "vie", to_string = "vi")] Vi,
    #[strum(serialize = "urd", to_string = "ur")] Ur,
    #[strum(serialize = "tha", to_string = "th")] Th,
    #[strum(serialize = "guj", to_string = "gu")] Gu,
    #[strum(serialize = "uzb", to_string = "uz")] Uz,
    #[strum(serialize = "pan", to_string = "pa")] Pa,
    #[strum(serialize = "aze", to_string = "az")] Az,
    #[strum(serialize = "ind", to_string = "id")] Id,
    #[strum(serialize = "tel", to_string = "te")] Te,
    #[strum(serialize = "pes", to_string = "fa")] Fa,
    #[strum(serialize = "mal", to_string = "ml")] Ml,
    #[strum(serialize = "ori", to_string = "or")] Or,
    #[strum(serialize = "mya", to_string = "my")] My,
    #[strum(serialize = "nep", to_string = "ne")] Ne,
    #[strum(serialize = "sin", to_string = "si")] Si,
    #[strum(serialize = "khm", to_string = "km")] Km,
    #[strum(serialize = "tuk", to_string = "tk")] Tk,
    #[strum(serialize = "aka", to_string = "ak")] Ak,
    #[strum(serialize = "zul", to_string = "zu")] Zu,
    #[strum(serialize = "sna", to_string = "sn")] Sn,
    #[strum(serialize = "afr", to_string = "af")] Af,
    #[strum(serialize = "lat", to_string = "la")] La,
    #[strum(serialize = "slk", to_string = "sk")] Sk,
    #[strum(serialize = "cat", to_string = "ca")] Ca,
    #[strum(serialize = "tgl", to_string = "tl")] Tl,
    #[strum(serialize = "hye", to_string = "hy")] Hy,
}


#[allow(unreachable_patterns)]
#[derive(AsRefStr, EnumString, Clone, PartialEq, Eq)]
pub enum LangNames {
    #[strum(serialize = "eo", to_string = "Esperanto")] Epo,
    #[strum(serialize = "en", to_string = "English")] En,
    #[strum(serialize = "ru", to_string = "Russian")] Ru,
    #[strum(serialize = "zh", to_string = "Chinese")] Zh,
    #[strum(serialize = "es", to_string = "Spanish")] Es,
    #[strum(serialize = "pt", to_string = "Portuguese")] Pt,
    #[strum(serialize = "it", to_string = "Italian")] It,
    #[strum(serialize = "bn", to_string = "Bengali")] Bn,
    #[strum(serialize = "fr", to_string = "French")] Fr,
    #[strum(serialize = "de", to_string = "German")] De,
    #[strum(serialize = "uk", to_string = "Ukrainian")] Uk,
    #[strum(serialize = "ka", to_string = "Georgian")] Ka,
    #[strum(serialize = "ar", to_string = "Arabic")] Ar,
    #[strum(serialize = "hi", to_string = "Hindi")] Hi,
    #[strum(serialize = "ja", to_string = "Japanese")] Ja,
    #[strum(serialize = "he", to_string = "Hebrew")] He,
    #[strum(serialize = "yi", to_string = "Yiddish")] Yi,
    #[strum(serialize = "pl", to_string = "Polish")] Pl,
    #[strum(serialize = "am", to_string = "Amharic")] Am,
    #[strum(serialize = "jv", to_string = "Javanese")] Jv,
    #[strum(serialize = "ko", to_string = "Korean")] Ko,
    #[strum(serialize = "nb", to_string = "Norwegian Bokmål")] Nb,
    #[strum(serialize = "da", to_string = "Danish")] Da,
    #[strum(serialize = "sv", to_string = "Swedish")] Sv,
    #[strum(serialize = "fi", to_string = "Finnish")] Fi,
    #[strum(serialize = "tr", to_string = "Turkish")] Tr,
    #[strum(serialize = "nl", to_string = "Dutch")] Nl,
    #[strum(serialize = "hu", to_string = "Hungarian")] Hu,
    #[strum(serialize = "cs", to_string = "Czech")] Cs,
    #[strum(serialize = "el", to_string = "Greek")] El,
    #[strum(serialize = "bg", to_string = "Bulgarian")] Bg,
    #[strum(serialize = "be", to_string = "Belarusian")] Be,
    #[strum(serialize = "mr", to_string = "Marathi")] Mr,
    #[strum(serialize = "kn", to_string = "Kannada")] Kn,
    #[strum(serialize = "ro", to_string = "Romanian")] Ro,
    #[strum(serialize = "sl", to_string = "Slovenian")] Sl,
    #[strum(serialize = "hr", to_string = "Croatian")] Hr,
    #[strum(serialize = "sr", to_string = "Serbian")] Sr,
    #[strum(serialize = "mk", to_string = "Macedonian")] Mk,
    #[strum(serialize = "lt", to_string = "Lithuanian")] Lt,
    #[strum(serialize = "lv", to_string = "Latvian")] Lv,
    #[strum(serialize = "et", to_string = "Estonian")] Et,
    #[strum(serialize = "ta", to_string = "Tamil")] Ta,
    #[strum(serialize = "vi", to_string = "Vietnamese")] Vi,
    #[strum(serialize = "ur", to_string = "Urdu")] Ur,
    #[strum(serialize = "th", to_string = "Thai")] Th,
    #[strum(serialize = "gu", to_string = "Gujarati")] Gu,
    #[strum(serialize = "uz", to_string = "Uzbek")] Uz,
    #[strum(serialize = "pa", to_string = "Punjabi")] Pa,
    #[strum(serialize = "az", to_string = "Azerbaijani")] Az,
    #[strum(serialize = "id", to_string = "Indonesian")] Id,
    #[strum(serialize = "te", to_string = "Telugu")] Te,
    #[strum(serialize = "fa", to_string = "Persian")] Fa,
    #[strum(serialize = "ml", to_string = "Malayalam")] Ml,
    #[strum(serialize = "or", to_string = "Odia")] Or,
    #[strum(serialize = "my", to_string = "Burmese")] My,
    #[strum(serialize = "ne", to_string = "Nepali")] Ne,
    #[strum(serialize = "si", to_string = "Sinhala")] Si,
    #[strum(serialize = "km", to_string = "Khmer")] Km,
    #[strum(serialize = "tk", to_string = "Turkmen")] Tk,
    #[strum(serialize = "ak", to_string = "Akan")] Ak,
    #[strum(serialize = "zu", to_string = "Zulu")] Zu,
    #[strum(serialize = "sn", to_string = "Shona")] Sn,
    #[strum(serialize = "af", to_string = "Afrikaans")] Af,
    #[strum(serialize = "la", to_string = "Latin")] La,
    #[strum(serialize = "sk", to_string = "Slovak")] Sk,
    #[strum(serialize = "ca", to_string = "Catalan")] Ca,
    #[strum(serialize = "tl", to_string = "Tagalog")] Tl,
    #[strum(serialize = "hy", to_string = "Armenian")] Hy,
}


/*//#[derive(Clone)]
#[derive(Clone, PartialEq, Eq)]
pub enum LangPair {
    SrcLang(Lang),
    TargetLang(Lang)
}
*/

#[allow(dead_code)]
pub trait Translator {
    fn translate(
        &mut self, 
        src_id: i64, 
        selected_text: String, //TODO get this from db if exist
        src_lang: Lang,
        target_lang: Lang,
        is_fav: bool
    ) -> ();
    //fn translate_sync(&mut self, text: String) -> String;
    fn terminate(&mut self) -> ();
    fn get_uid(&self) -> String;
    fn get_name(&self) -> String;
}

#[allow(dead_code)]
pub trait Dictionary {
    fn translate(
        &mut self, 
        src_id: i64, 
        selected_text: String, //TODO get this from db if exist
        src_lang: Lang,
        target_lang: Lang,
        is_fav: bool
    ) -> ();
    //fn translate_sync(&mut self, text: String) -> String;
    fn terminate(&mut self) -> ();
    fn get_uid(&self) -> String;
    fn get_name(&self) -> String;
}
pub trait TTSEngine {
    fn generate(&self, text: String, src_id: i64, speaker_uid: String) -> Result<String>;
}

pub trait PRNNService {
    fn generate(&self, text: String, src_id: i64) -> Result<()>;
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct TranslationRequest {
    text: String, 
    src_lang: Lang, 
    target_lang: Lang
}

#[derive(Debug)]
pub struct UIState {
    pub src_text: Option<String>,
    pub tr_uid: Option<String>,
    pub translator: Option<String>, 
    pub src: Option<Lang>, 
    pub target: Option<Lang>, 
    pub translation_text: Option<String>,
    pub is_fav: Option<bool>
}

#[derive(Debug)]
pub struct UIStateDict {
    pub src_id: Option<i64>,
    pub src_text_dict: Option<String>,
    pub dict_uid: Option<String>,
    pub dict_name: Option<String>, 
    //pub src: Lang, 
    //pub target: Lang, 
    pub dict_text: Option<String>,
    pub is_fav: Option<bool>
}

//browser row structs
#[derive(Debug)]
#[allow(dead_code)]
pub struct TranslSource {
    pub id: i32,
    pub text: String,
}
#[derive(Debug)]
#[allow(dead_code)]
pub struct TTSource {
    pub path: String,
    pub engine: String,
    pub voice: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PRNNSource {
    pub path: String,
    pub service: String,
}

