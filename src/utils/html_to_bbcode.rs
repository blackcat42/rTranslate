use debug_print::{debug_println as dprintln};
use tl::{Node, Parser, ParserOptions};
use anyhow::{anyhow, Result};
use regex::Regex;

fn normalize_text(text: &str) -> String {
    let multiple_newlines = Regex::new(r"\r?\n(?:[ \t]*\r?\n)+").unwrap();
    let text = multiple_newlines.replace_all(text, "\n").into_owned();
    let text = text.replace("$RT_NEWLINE$", "\n");
    text
}

/*fn trim_to_one(text: &str) -> String {
    if text.trim().is_empty() {
        return if text.is_empty() { String::new() } else { " ".to_string() };
    }
    let leading = text.chars().next().filter(|c| c.is_whitespace()).map(|c| c.to_string()).unwrap_or("".to_string());
    let trailing = text.chars().last().filter(|c| c.is_whitespace()).map(|c| c.to_string()).unwrap_or("".to_string());
    let trimmed = text.trim();
    
    format!("{leading}{trimmed}{trailing}")
}*/

pub fn node_to_bb(
    node: &Node, 
    parser: &Parser, 
    tag_handler: fn(tag: &tl::HTMLTag, inner_text: String) -> String
) -> String {

    match node {
        Node::Raw(raw) => {
            raw.as_utf8_str().into_owned()
        },
        Node::Comment(raw) => {
            "".to_string()
        },
        Node::Tag(tag) => {
            let mut inner_text = String::new();
            for child_handle in tag.children().top().iter() {
                if let Some(child_node) = child_handle.get(parser) {
                    let new_text = node_to_bb(child_node, parser, tag_handler);                
                    inner_text.push_str(&new_text);
                }
            }
            tag_handler(tag, inner_text)
        }
    }
}

#[derive(Debug)]
pub enum HTMLSelectorType {
    Id(String),
    QuerySelector(String),
    None
}

pub fn html_to_bbcode(
    html: &str,
    container: HTMLSelectorType,
    tag_h: fn(tag: &tl::HTMLTag, inner_text: String) -> String
) -> Result<String> {

    let html = html
        .replace("&#91;", "(")
        .replace("&#93;", ")")
        .replace('[', "(")
        .replace(']', ")");
    let dom = tl::parse(&html, ParserOptions::default())?;

    let parser = dom.parser();
    let container_tag: &tl::HTMLTag;

    match container {
        HTMLSelectorType::Id(id) => {
            let container_handle = dom.get_element_by_id(id.as_str()).ok_or(
                anyhow!("dom.get_element_by_id error")
            )?;
            let container_node = container_handle.get(parser).ok_or(
                anyhow!("container_handle.get error")
            )?;
            container_tag = container_node.as_tag().ok_or(
                anyhow!("container_node.as_tag() error")
            )?;
        },

        HTMLSelectorType::QuerySelector(selector) => {
            let container_handle = dom.query_selector(selector.as_str())
            .and_then(|mut iter| iter.next()).ok_or(
                anyhow!("query_selector error")
            )?;
            let container_node = container_handle.get(parser).ok_or(
                anyhow!("container_handle.get error")
            )?;
            container_tag = container_node.as_tag().ok_or(
                anyhow!("container_node.as_tag() error")
            )?;
        },

        HTMLSelectorType::None => {
            let container_handle = dom.query_selector("body")
            .and_then(|mut iter| iter.next()).ok_or(
                anyhow!("query_selector error")
            )?;
            let container_node = container_handle.get(parser).ok_or(
                anyhow!("container_handle.get error")
            )?;
            container_tag = container_node.as_tag().ok_or(
                anyhow!("container_node.as_tag() error")
            )?;
        }
    };

    let mut result = String::new();

    for child_handle in container_tag.children().top().iter() {
        if let Some(child_node) = child_handle.get(parser) {
            result.push_str(&node_to_bb(child_node, parser, tag_h));
        }
    }

    let result = result.trim().to_owned();
    
    let result = html_escape::decode_html_entities(&result).to_string();
    let result = result.replace("([c blue]edit[/c])", "");
    let result = result.replace("  • [quote]", "[quote]  ");

    Ok(normalize_text(&result))
}
