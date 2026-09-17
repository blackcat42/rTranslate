use super::GLOBAL_SETTINGS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Plain,
    Bold,
    Italic,
    Header,
}

impl Mark {
    fn as_char(self) -> char {
        match self {
            Mark::Plain => 'A',
            Mark::Bold => 'B',
            Mark::Italic => 'C',
            Mark::Header => 'D',
        }
    }
    fn to_string(self) -> String {
        self.as_char().to_string()
    }
}


fn parse_line(s: &str) -> (String, String) {
    let mut out1 = String::new();
    let mut out2 = String::new();
    let mut i = 0;

    while i < s.len() {
        let rest = &s[i..];

        /*if i == 0 && rest.starts_with("* ") {
            out1.push_str("• ");
            out2.push_str(&(Mark::Plain).to_string().repeat("• ".len()));
            i += 2;
            continue;
        }*/
        if i == 0 && rest.starts_with("### ") {
            let content = &rest[4..];
            //out1.push_str(content.to_string().to_uppercase().as_str());
            out1.push_str(content);
            out2.push_str(&(Mark::Header).to_string().repeat(content.len()));
            break;
        }

        if rest.starts_with("**") {
            if let Some(pos) = rest[2..].find("**") {
                let content = &rest[2..2 + pos];
                out1.push_str(content);
                out2.push_str(&(Mark::Bold).to_string().repeat(content.len()));
                i += 2 + pos + 2;
                continue;
            } else {
                out1.push_str("**");
                out2.push((Mark::Plain).as_char());
                out2.push((Mark::Plain).as_char());
                i += 2;
                continue;
            }
        }

        if rest.starts_with('*') {
            if let Some(pos) = rest[1..].find('*') {
                let content = &rest[1..1 + pos];
                out1.push_str(content);
                out2.push_str(&(Mark::Italic).to_string().repeat(content.len()));
                i += 1 + pos + 1;
                continue;
            } else {
                out1.push('*');
                out2.push((Mark::Plain).as_char());
                i += 1;
                continue;
            }
        }

        if rest.starts_with('_') {
            if let Some(pos) = rest[1..].find('_') {
                let content = &rest[1..1 + pos];
                out1.push_str(content);
                out2.push_str(&(Mark::Italic).to_string().repeat(content.len()));
                i += 1 + pos + 1;
                continue;
            } else {
                out1.push('_');
                out2.push((Mark::Plain).as_char());
                i += 1;
                continue;
            }
        }

        let ch = rest.chars().next().unwrap();
        out1.push(ch);
        out2.push_str(&(Mark::Plain).to_string().repeat(ch.len_utf8()));
        i += ch.len_utf8();
    }

    (out1, out2)
}

pub fn from_md(input: &str) -> (String, String, Vec<fltk::text::StyleTableEntryExt>) {
    let style_a = fltk::text::StyleTableEntryExt {
        color: fltk::enums::Color::Black,
        font: fltk::enums::Font::Helvetica,
        size: GLOBAL_SETTINGS.text_font_size,
        attr: fltk::text::TextAttr::None,
        bgcolor: fltk::enums::Color::Yellow,
    };
    let style_b = fltk::text::StyleTableEntryExt {
        color: fltk::enums::Color::Black,
        font: fltk::enums::Font::HelveticaBold,
        size: GLOBAL_SETTINGS.text_font_size,
        attr: fltk::text::TextAttr::None,
        bgcolor: fltk::enums::Color::Yellow,
    };
    let style_c = fltk::text::StyleTableEntryExt {
        color: fltk::enums::Color::Black,
        font: fltk::enums::Font::HelveticaItalic,
        size: GLOBAL_SETTINGS.text_font_size,
        attr: fltk::text::TextAttr::None,
        bgcolor: fltk::enums::Color::Yellow,
    };
    let style_d = fltk::text::StyleTableEntryExt {
        color: fltk::enums::Color::from_hex(0x1E3A8A),
        font: fltk::enums::Font::HelveticaBold,
        size: GLOBAL_SETTINGS.text_font_size + 1,
        attr: fltk::text::TextAttr::None,
        bgcolor: fltk::enums::Color::Yellow,
    };

    let mut result_text = "".to_string();
    let mut result_style = "".to_string();

    let input = input
        .replace("\n* ", "\n• ")
        .replace("$\\rightarrow$", "→")
        .replace("$\\Rightarrow;", "⇒")
        .replace("$\\supset;", "⊃")
        .replace("$\\Leftrightarrow;", "⇔");

    for line in input.split_inclusive('\n') {
        let (line_text, line_style) = parse_line(&line);

        result_text.push_str(&line_text);
        result_style.push_str(&line_style);
    }

    let styles = vec![style_a, style_b, style_c, style_d];
    (result_text, result_style, styles)
}

pub fn from_bbcode(dict_text: &str) -> (String, String, Vec<fltk::text::StyleTableEntryExt>) {
    let text_chuncs = crate::utils::bbcode::dsl_parse(dict_text);

    //teal, red, green, blue, indigo
    let style_a = fltk::text::StyleTableEntryExt {
        color: fltk::enums::Color::Black,
        font: fltk::enums::Font::Helvetica,
        size: GLOBAL_SETTINGS.text_font_size,
        attr: fltk::text::TextAttr::None,
        bgcolor: fltk::enums::Color::Yellow,
    };
    let style_b = fltk::text::StyleTableEntryExt {
        color: fltk::enums::Color::Black,
        font: fltk::enums::Font::HelveticaBold,
        size: GLOBAL_SETTINGS.text_font_size,
        attr: fltk::text::TextAttr::None,
        bgcolor: fltk::enums::Color::Yellow,
    };
    let style_c = fltk::text::StyleTableEntryExt {
        color: fltk::enums::Color::Red,
        font: fltk::enums::Font::Helvetica,
        size: GLOBAL_SETTINGS.text_font_size,
        attr: fltk::text::TextAttr::None,
        bgcolor: fltk::enums::Color::Yellow,
    };
    let style_d = fltk::text::StyleTableEntryExt {
        color: fltk::enums::Color::DarkGreen,
        font: fltk::enums::Font::Helvetica,
        size: GLOBAL_SETTINGS.text_font_size,
        attr: fltk::text::TextAttr::None,
        bgcolor: fltk::enums::Color::Yellow,
    };
    let style_e = fltk::text::StyleTableEntryExt {
        color: fltk::enums::Color::DarkBlue,
        font: fltk::enums::Font::Helvetica,
        size: GLOBAL_SETTINGS.text_font_size,
        attr: fltk::text::TextAttr::None,
        bgcolor: fltk::enums::Color::Yellow,
    };
    let style_f = fltk::text::StyleTableEntryExt {
        color: fltk::enums::Color::from_hex(0x008080), //teal
        font: fltk::enums::Font::Helvetica,
        size: GLOBAL_SETTINGS.text_font_size,
        attr: fltk::text::TextAttr::None,
        bgcolor: fltk::enums::Color::Yellow,
    };
    let style_g = fltk::text::StyleTableEntryExt {
        color: fltk::enums::Color::from_hex(0x4B0082), //indigo
        font: fltk::enums::Font::Helvetica,
        size: GLOBAL_SETTINGS.text_font_size,
        attr: fltk::text::TextAttr::None,
        bgcolor: fltk::enums::Color::Yellow,
    };

    let style_h = fltk::text::StyleTableEntryExt {
        color: fltk::enums::Color::Black,
        font: fltk::enums::Font::HelveticaItalic,
        size: GLOBAL_SETTINGS.text_font_size,// - 1,
        attr: fltk::text::TextAttr::None,
        bgcolor: fltk::enums::Color::Yellow,
    };
    let style_i = fltk::text::StyleTableEntryExt {
        color: fltk::enums::Color::Black,
        font: fltk::enums::Font::Helvetica,
        size: GLOBAL_SETTINGS.text_font_size,// - 1,
        attr: fltk::text::TextAttr::BgColorExt,
        bgcolor: fltk::enums::Color::from_hex(0xFFF2CC),
    };
    let style_j = fltk::text::StyleTableEntryExt {
        color: fltk::enums::Color::Black,
        font: fltk::enums::Font::HelveticaBold,
        size: GLOBAL_SETTINGS.text_font_size,// - 1,
        attr: fltk::text::TextAttr::BgColorExt,
        bgcolor: fltk::enums::Color::from_hex(0xFFF2CC),
    };


    //sbuf.set_text("");
    let mut result_text = "".to_string();
    let mut str_f = "".to_string();
    //dbg!(&text_chuncs);
    for chunc in text_chuncs.iter() {
        
        result_text.push_str(&chunc.text);
        if chunc.is_highlighted && chunc.is_bold {
            str_f.push_str(&"J".repeat(chunc.text.len()));
        } else if chunc.is_highlighted {
            str_f.push_str(&"I".repeat(chunc.text.len()));
        } else if &chunc.color == "red" {
            str_f.push_str(&"C".repeat(chunc.text.len()));
        } else if &chunc.color == "green" {
            str_f.push_str(&"D".repeat(chunc.text.len()));
        } else if &chunc.color == "blue" || &chunc.color == "darkblue" {
            str_f.push_str(&"E".repeat(chunc.text.len()));
        } else if &chunc.color == "teal" {
            str_f.push_str(&"F".repeat(chunc.text.len()));
        } else if &chunc.color == "indigo" {
            str_f.push_str(&"G".repeat(chunc.text.len()));
        } else if chunc.is_bold {
            str_f.push_str(&"B".repeat(chunc.text.len()));
        } else if chunc.is_i {
            str_f.push_str(&"H".repeat(chunc.text.len()));
        } else {
            str_f.push_str(&"A".repeat(chunc.text.len()));
        }
    }

    let styles = vec![style_a, style_b, style_c, style_d, style_e, style_f, style_g, style_h, style_i, style_j];
    (result_text, str_f, styles)
}
