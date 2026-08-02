#![allow(unreachable_patterns)]

use global_hotkey::{
    GlobalHotKeyEvent, 
};

//use std::str::FromStr;
use strum_macros::AsRefStr;
use strum_macros::EnumString;
use anyhow::Result;

use strum_macros::EnumIter;
//use strum::IntoEnumIterator;
use crate::screen_ocr::CropSegment;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum TrayEvent {
    ShowMainWin,
    ShowPopupWin,
    ShowPopupDictWin,
    Settings,
    Exit
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppEvent {
    HotKey(GlobalHotKeyEvent),
    TrayMenuEvent(TrayEvent),
    OCRInit,
    OCRDrop,
    OCRSuccess(String),
    OCRCropUpdate(CropSegment),
    OCRun,

    //TODO:
    SaveTranslation(TranslResult),
    SaveDictEntry(DictResult),
    PRNNSave((i64, String, String)),
    ToggleFav(bool),

    TranslateText(String, bool),
    Translate(bool, bool, bool),
    RequestDictEntry(bool, bool, bool),
    SendToDict(),
    TTString(),
    PRNNString(bool),
    TTSave(i64, String, String, String),
    TTSPlay(String),

    SetWaiting(Option<String>, bool),
    SetReady(Option<String>, bool),
    SetStatus(Box<str>, bool, bool),
    Message(Box<str>),
    
    //TODO:
    UpdateUi(UIState, bool),
    UpdateUiDict(UIStateDict, bool),
    ClearUi(bool),


    UpdateHistoryBrowserView(Vec<TranslSource>),
    UpdateFavBrowserView(Vec<TranslSource>),
    UpdateTTSBrowser(String, Vec<TTSource>, Vec<PRNNSource>),

    SetSrcLang(Lang),
    SetTargetLang(Lang),
    SetTranslator(String),
    SetDict(String),
    SetTTSEngine(String, String),
    SetPRNNEngine(String),
    UpdateTTState(i32),

}


//from ISO 639_3 or ISO 639_1 str:    Lang::from_str
//Lang --> ISO 639_1 str:             .as_ref()
//https://en.wikipedia.org/wiki/List_of_ISO_639_language_codes

//TODO: llm-generated, need to verify
#[allow(unused)]
#[derive(Debug, AsRefStr, EnumString, Clone, PartialEq, Eq, EnumIter)]
pub enum Lang {
    #[strum(serialize = "auto", to_string = "auto")] Auto,
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

    #[strum(serialize = "sqi", to_string = "sq")] Sq,
    #[strum(serialize = "eus", to_string = "eu")] Eu,
    #[strum(serialize = "glg", to_string = "gl")] Gl,
    #[strum(serialize = "hat", to_string = "ht")] Ht,
    #[strum(serialize = "isl", to_string = "is")] Is,
    #[strum(serialize = "gle", to_string = "ga")] Ga,
    #[strum(serialize = "msa", to_string = "ms")] Ms,
    #[strum(serialize = "mlt", to_string = "mt")] Mt,
    #[strum(serialize = "nor", to_string = "no")] No,
    #[strum(serialize = "swa", to_string = "sw")] Sw,
    #[strum(serialize = "cym", to_string = "cy")] Cy,
    #[strum(serialize = "hmn", to_string = "hmn")] Hmn,
    #[strum(serialize = "lao", to_string = "lo")] Lo,
    #[strum(serialize = "kaz", to_string = "kk")] Kk,
    #[strum(serialize = "tgk", to_string = "tg")] Tg,
    #[strum(serialize = "mon", to_string = "mn")] Mn,
    #[strum(serialize = "tat", to_string = "mo")] Tt,
}

impl Lang {
    pub fn code_3(&self) -> &str {
        match self {
            Lang::Auto => "auto",
            Lang::Epo => "epo",
            Lang::En => "eng",
            Lang::Ru => "rus",
            Lang::Zh => "cmn",
            Lang::Es => "spa",
            Lang::Pt => "por",
            Lang::It => "ita",
            Lang::Bn => "ben",
            Lang::Fr => "fra",
            Lang::De => "deu",
            Lang::Uk => "ukr",
            Lang::Ka => "kat",
            Lang::Ar => "ara",
            Lang::Hi => "hin",
            Lang::Ja => "jpn",
            Lang::He => "heb",
            Lang::Yi => "yid",
            Lang::Pl => "pol",
            Lang::Am => "amh",
            Lang::Jv => "jav",
            Lang::Ko => "kor",
            Lang::Nb => "nob",
            Lang::Da => "dan",
            Lang::Sv => "swe",
            Lang::Fi => "fin",
            Lang::Tr => "tur",
            Lang::Nl => "nld",
            Lang::Hu => "hun",
            Lang::Cs => "ces",
            Lang::El => "ell",
            Lang::Bg => "bul",
            Lang::Be => "bel",
            Lang::Mr => "mar",
            Lang::Kn => "kan",
            Lang::Ro => "ron",
            Lang::Sl => "slv",
            Lang::Hr => "hrv",
            Lang::Sr => "srp",
            Lang::Mk => "mkd",
            Lang::Lt => "lit",
            Lang::Lv => "lav",
            Lang::Et => "est",
            Lang::Ta => "tam",
            Lang::Vi => "vie",
            Lang::Ur => "urd",
            Lang::Th => "tha",
            Lang::Gu => "guj",
            Lang::Uz => "uzb",
            Lang::Pa => "pan",
            Lang::Az => "aze",
            Lang::Id => "ind",
            Lang::Te => "tel",
            Lang::Fa => "pes",
            Lang::Ml => "mal",
            Lang::Or => "ori",
            Lang::My => "mya",
            Lang::Ne => "nep",
            Lang::Si => "sin",
            Lang::Km => "khm",
            Lang::Tk => "tuk",
            Lang::Ak => "aka",
            Lang::Zu => "zul",
            Lang::Sn => "sna",
            Lang::Af => "afr",
            Lang::La => "lat",
            Lang::Sk => "slk",
            Lang::Ca => "cat",
            Lang::Tl => "tgl",
            Lang::Hy => "hye",

            Lang::Sq => "sqi",
            Lang::Eu => "eus",
            Lang::Gl => "glg",
            Lang::Ht => "hat",
            Lang::Is => "isl",
            Lang::Ga => "gle",
            Lang::Ms => "msa",
            Lang::Mt => "mlt",
            Lang::No => "nor",
            Lang::Sw => "swa",
            Lang::Cy => "cym",
            Lang::Hmn => "hmn",
            Lang::Lo => "lao",
            Lang::Kk => "kaz",
            Lang::Tg => "tgk",
            Lang::Mn => "mon",
            Lang::Tt => "tat",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Lang::Auto => "Auto",
            Lang::Epo => "Esperanto",
            Lang::En => "English",
            Lang::Ru => "Russian",
            Lang::Zh => "Chinese",
            Lang::Es => "Spanish",
            Lang::Pt => "Portuguese",
            Lang::It => "Italian",
            Lang::Bn => "Bengali",
            Lang::Fr => "French",
            Lang::De => "German",
            Lang::Uk => "Ukrainian",
            Lang::Ka => "Georgian",
            Lang::Ar => "Arabic",
            Lang::Hi => "Hindi",
            Lang::Ja => "Japanese",
            Lang::He => "Hebrew",
            Lang::Yi => "Yiddish",
            Lang::Pl => "Polish",
            Lang::Am => "Amharic",
            Lang::Jv => "Javanese",
            Lang::Ko => "Korean",
            Lang::Nb => "Norwegian Bokmål",
            Lang::Da => "Danish",
            Lang::Sv => "Swedish",
            Lang::Fi => "Finnish",
            Lang::Tr => "Turkish",
            Lang::Nl => "Dutch",
            Lang::Hu => "Hungarian",
            Lang::Cs => "Czech",
            Lang::El => "Greek",
            Lang::Bg => "Bulgarian",
            Lang::Be => "Belarusian",
            Lang::Mr => "Marathi",
            Lang::Kn => "Kannada",
            Lang::Ro => "Romanian",
            Lang::Sl => "Slovenian",
            Lang::Hr => "Croatian",
            Lang::Sr => "Serbian",
            Lang::Mk => "Macedonian",
            Lang::Lt => "Lithuanian",
            Lang::Lv => "Latvian",
            Lang::Et => "Estonian",
            Lang::Ta => "Tamil",
            Lang::Vi => "Vietnamese",
            Lang::Ur => "Urdu",
            Lang::Th => "Thai",
            Lang::Gu => "Gujarati",
            Lang::Uz => "Uzbek",
            Lang::Pa => "Punjabi",
            Lang::Az => "Azerbaijani",
            Lang::Id => "Indonesian",
            Lang::Te => "Telugu",
            Lang::Fa => "Persian",
            Lang::Ml => "Malayalam",
            Lang::Or => "Oriya",
            Lang::My => "Burmese",
            Lang::Ne => "Nepali",
            Lang::Si => "Sinhala",
            Lang::Km => "Khmer",
            Lang::Tk => "Turkmen",
            Lang::Ak => "Akan",
            Lang::Zu => "Zulu",
            Lang::Sn => "Shona",
            Lang::Af => "Afrikaans",
            Lang::La => "Latin",
            Lang::Sk => "Slovak",
            Lang::Ca => "Catalan",
            Lang::Tl => "Tagalog",
            Lang::Hy => "Armenian",

            Lang::Sq => "Albanian",
            Lang::Eu => "Basque",
            Lang::Gl => "Galician",
            Lang::Ht => "Haitian",
            Lang::Is => "Icelandic",
            Lang::Ga => "Irish",
            Lang::Ms => "Malay",
            Lang::Mt => "Maltese",
            Lang::No => "Norwegian",
            Lang::Sw => "Swahili",
            Lang::Cy => "Welsh",
            Lang::Hmn => "Hmong",
            Lang::Lo => "Lao",
            Lang::Kk => "Kazakh",
            Lang::Tg => "Tajik",
            Lang::Mn => "Mongolian",
            Lang::Tt => "Tatar",
        }
    }
}

//todo: whatlang::Lang::from_code("eng").unwrap().eng_name()


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
        is_lang_detected: bool
    ) -> ();
    //fn translate_sync(&mut self, text: String) -> String;
    fn terminate(&mut self) -> ();
    fn get_uid(&self) -> &str;
    fn get_name(&self) -> &str;
}

#[allow(dead_code)]
pub trait Dictionary {
    fn translate(
        &mut self, 
        src_id: i64, 
        selected_text: String, //TODO get this from db if exist
        src_lang: Lang,
        target_lang: Lang,
    ) -> ();
    //fn translate_sync(&mut self, text: String) -> String;
    fn terminate(&mut self) -> ();
    fn get_uid(&self) -> &str;
    fn get_name(&self) -> &str;
}
pub trait TTService {
    fn generate(&self, text: String, src_id: i64, speaker_uid: String) -> ();
    fn get_name(&self) -> &str;
}

pub trait PRNNService {
    fn generate(&self, text: String, src_lang: Lang, src_id: i64) -> Result<()>;
    fn get_name(&self) -> &str;
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
    pub src_text: String,
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
    pub src_text_dict: String,
    pub dict_uid: Option<String>,
    pub dict_name: Option<String>, 
    pub src: Option<Lang>,
    pub target: Option<Lang>,
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
    pub service: String,
    pub voice: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PRNNSource {
    pub path: String,
    pub service: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct TranslResult {
    pub src_id: i64,
    pub text: String,
    pub tr_uid: String, 
    pub src: Lang, 
    pub target: Lang, 
    pub translation_text: String
}

#[derive(Debug)]
pub struct DictResult {
    pub src_id: i64,
    pub dict_uid: String,
    pub text: String, 
    pub src: Option<Lang>, 
    pub target: Option<Lang>
}

#[derive(Default)]
pub struct BLWCoords {
    pub x: i32,
    pub y: i32,
    pub x_start: i32,
    pub y_start: i32,
    pub initial_window_height: i32,
    pub initial_window_width: i32,
    pub init_on_border_left: bool,
    pub init_on_border_right: bool,
    pub init_on_border_top: bool,
    pub init_on_border_bottom: bool,
}



#[derive(Debug, Deserialize, Serialize)]
pub struct OCRModelOption {
    pub name: String,
    pub det_model: String,
    pub rec_model: String,
    pub charset: String
}
