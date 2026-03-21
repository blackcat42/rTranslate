#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BBRichText {
    pub text: String,
    pub is_bold: bool,
    pub color: String,
}

pub fn dsl_parse(s: &str) -> Vec<BBRichText> {
    //https://anatoly.dev/dsl-manual/
    let parser = BBCode::default();

    let first_line = s.lines().next();
    let first_line = first_line.unwrap_or("~").trim();

    let s = s
        .replace(r"\[", "(")
        .replace(r"\]", ")");

    let s = s
        .replace("[trn]", "")
        .replace("[/trn]", "")
        .replace("[com]", "")
        .replace("[/com]", "")
        .replace("[p]", "")
        .replace("[/p]", "");


    let s = s
        .replace("[m1]", "[m]")
        .replace("[m2]", "[m]")
        .replace("[m3]", "[m]") //todo regex
        .replace("[m]", "")
        .replace("[/m]", "");

    let s = s
        .replace("[']", "[color=red]")
        .replace("[/']", "[/color]")
        .replace("~", first_line);

    
    let s = s
        .replace("[c ", "[color=")
        .replace("[/c]", "[/color]");

    let s = s
        .replace("\t", "    ");

    let tree = parser.parse(&s);

    //dbg!(&tree);
    let mut sorted_entries: Vec<(&i32, &BBNode)> = tree.nodes.iter().collect();
    sorted_entries.sort_by(|a, b| a.0.cmp(b.0));

    let mut final_arr: Vec<BBRichText> = vec![];
    for (_key, value) in sorted_entries {
        let mut tags: Vec<(BBTag, String)> = Vec::new();

        if let Some(v) = &value.value {
            tags.push((value.tag.clone(), v.to_string()));
        } else {
            tags.push((value.tag.clone(), "".to_string()));
        }

        #[allow(clippy::collapsible_if)]
        if let Some(id) = &value.parent {
            if let Some(parent) = tree.nodes.get(id) {
                if let Some(v) = &value.value {
                    tags.push((parent.tag.clone(), v.to_string()));
                } else {
                    tags.push((parent.tag.clone(), "".to_string()));
                }
            };
        };
        let mut is_bold = false;
        let mut color = "".to_string();
        for tag in tags {
            match tag.0 {
                BBTag::FontColor => {
                    color = tag.1.clone();
                },
                BBTag::Bold => {
                    is_bold = true;
                },
                _ => {

                }
            }
        }
        #[allow(clippy::len_zero)]
        if value.text.len() > 0 {
            if let Some(el) = final_arr.last_mut() {
                if el.is_bold == is_bold && el.color == color {
                    el.text.push_str(&value.text);
                } else {
                    final_arr.push(
                        BBRichText {
                            text: value.text.clone(),
                            is_bold,
                            color: color.to_string()
                        }
                    );
                }
            } else {
                final_arr.push(
                    BBRichText {
                        text: value.text.clone(),
                        is_bold,
                        color: color.to_string()
                    }
                );
            }
        }
        
    }

    final_arr
}




//A tree parser and tagger for BBCode formatted text.
//original source:
//https://github.com/arviceblot/bbcode-tagger
//author: arviceblot <github@relay.arviceblot.com>
//license: "MIT"

//use core::fmt;
use regex::Regex;
use std::collections::HashMap;
use std::fmt::Display;

static RE_OPEN_TAG: &str = r#"^\[(?P<tag>[^/\]]+?\S*?)((?:[ \t]+\S+?)?="?(?P<val>[^\]\n]*?))?"?\]"#;
static RE_CLOSE_TAG: &str = r#"^\[/(?P<tag>[^/\]]+?\S*?)\]"#;
static RE_NEWLINE: &str = r#"^\r?\n"#;

/// BBCode tag type enum
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BBTag {
    /// No tag
    None,
    Bold,
    Italic,
    Underline,
    Strikethrough,
    FontSize,
    FontColor,
    Center,
    Left,
    Right,
    Superscript,
    Subscript,
    Blur,
    Quote,
    Spoiler,
    Link,
    Email,
    Image,
    ListOrdered,
    ListUnordered,
    ListItem,
    Code,
    Preformatted,
    Table,
    TableHeading,
    TableRow,
    TableCell,
    YouTube,
    /// Some other unhandled tag
    Unknown,
    // TODO: Handle some extra codes
    // - indent
}
impl From<&str> for BBTag {
    fn from(value: &str) -> BBTag {
        let binding = value.trim().to_lowercase();
        let trim_tag = binding.as_str();
        match trim_tag {
            "b" => BBTag::Bold,
            "i" => BBTag::Italic,
            "u" => BBTag::Underline,
            "s" => BBTag::Strikethrough,
            "size" => BBTag::FontSize,
            "color" => BBTag::FontColor,
            "center" => BBTag::Center,
            "left" => BBTag::Left,
            "right" => BBTag::Right,
            "sup" => BBTag::Superscript,
            "sub" => BBTag::Subscript,
            "blur" => BBTag::Blur,
            "email" => BBTag::Email,
            "quote" => BBTag::Quote,
            "spoiler" => BBTag::Spoiler,
            "url" => BBTag::Link,
            "img" => BBTag::Image,
            "ul" | "list" => BBTag::ListUnordered,
            "ol" => BBTag::ListOrdered,
            "li" | "*" => BBTag::ListItem,
            "code" | "highlight" => BBTag::Code,
            "pre" => BBTag::Preformatted,
            "table" => BBTag::Table,
            "tr" => BBTag::TableRow,
            "th" => BBTag::TableHeading,
            "td" => BBTag::TableCell,
            "youtube" => BBTag::YouTube,
            "" => BBTag::None,
            &_ => BBTag::Unknown,
        }
    }
}

/// Node in the BBTag Tree with associated data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BBNode {
    /// Unformatted string text
    pub text: String,
    /// Associated tag
    pub tag: BBTag,
    /// Possible value related to tag (i.e. "4" in [SIZE=4])
    pub value: Option<String>,
    /// Parent node. Only root (id = 0) node should not have parent
    pub parent: Option<i32>,
    /// Child nodes
    pub children: Vec<i32>,
}
impl Default for BBNode {
    fn default() -> Self {
        Self {
            text: "".to_string(),
            tag: BBTag::None,
            value: None,
            parent: None,
            children: vec![],
        }
    }
}

#[allow(unused)]
impl BBNode {
    /// Create a new BBNode with Text and Tag
    pub fn new(text: &str, tag: BBTag) -> BBNode {
        BBNode {
            text: String::from(text),
            tag,
            value: None,
            parent: None,
            children: vec![],
        }
    }
}

/// Main data scructure for parsed BBCode, usually a root node and child nodes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BBTree {
    /// Nodes stored in the tree
    pub nodes: HashMap<i32, BBNode>,
    id: i32,
}
impl Default for BBTree {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            id: -1,
        }
    }
}
impl Display for BBTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Nodes: {}", self.id)?;
        self.fmt_node(f, 0)
    }
}
#[allow(clippy::all)]
impl BBTree {
    /// Get a node by ID
    pub fn get_node(&self, i: i32) -> &BBNode {
        self.nodes.get(&i).unwrap()
    }
    /// Get a node as mutable by ID
    pub fn get_node_mut(&mut self, i: i32) -> &mut BBNode {
        self.nodes.get_mut(&i).unwrap()
    }
    /// Add a new node and return the new node ID
    pub fn add_node(&mut self, node: BBNode) -> i32 {
        self.id += 1;
        self.nodes.insert(self.id, node);
        self.id
    }
    /// Recursive (I know...) function to get the depth of a given node ID in the tree
    pub fn get_depth(&self, i: i32) -> usize {
        if self.get_node(i).parent.is_none() {
            return 0;
        }
        return 1 + self.get_depth(self.get_node(i).parent.unwrap());
    }
    fn fmt_node(&self, f: &mut std::fmt::Formatter<'_>, i: i32) -> std::fmt::Result {
        let indent = self.get_depth(i) * 2;
        let node = self.get_node(i);
        writeln!(f, "{:indent$}ID    : {}", "", i, indent = indent)?;
        writeln!(f, "{:indent$}Text  : {}", "", node.text, indent = indent)?;
        writeln!(f, "{:indent$}Tag   : {:?}", "", node.tag, indent = indent)?;
        writeln!(f, "{:indent$}Value : {:?}", "", node.value, indent = indent)?;
        writeln!(
            f,
            "{:indent$}Parent: {:?}",
            "",
            node.parent,
            indent = indent
        )?;
        writeln!(f)?;
        for child in node.children.iter() {
            self.fmt_node(f, *child)?;
        }
        Ok(())
    }
}

/// BBCode parser
#[derive(Debug)]
pub struct BBCode {
    open_matcher: Regex,
    close_matcher: Regex,
    newline_matcher: Regex,
}
impl Default for BBCode {
    fn default() -> Self {
        Self {
            open_matcher: Regex::new(RE_OPEN_TAG).unwrap(),
            close_matcher: Regex::new(RE_CLOSE_TAG).unwrap(),
            newline_matcher: Regex::new(RE_NEWLINE).unwrap(),
        }
    }
}

impl BBCode {
    /// Parse the given input into tagged BBCode tree
    pub fn parse(&self, input: &str) -> BBTree {
        // Slice through string until open or close tag match
        let mut slice = &input[0..];

        // set up initial tree with empty node
        // let curr_node = BBNode::new("", BBTag::None);
        let mut tree = BBTree::default();
        let mut curr_node = tree.add_node(BBNode::default());
        let mut closed_tag = false;

        while !slice.is_empty() {
            // special handling for [*] short code
            // check for newline while ListItem is open
            if let Some(captures) = self.newline_matcher.captures(slice) {
                if tree.get_node(curr_node).tag == BBTag::ListItem {
                    // we are in a ListItem, close list item
                    curr_node = tree.get_node(curr_node).parent.unwrap();

                    // move past newline
                    slice = &slice[captures.get(0).unwrap().as_str().len()..];
                    closed_tag = true;
                    continue;
                }
                if tree.get_node(curr_node).parent.is_some()
                    && tree.get_node(tree.get_node(curr_node).parent.unwrap()).tag
                        == BBTag::ListItem
                {
                    // parent is a list item
                    // close current and parent
                    curr_node = tree
                        .get_node(tree.get_node(curr_node).parent.unwrap())
                        .parent
                        .unwrap();
                    // move past newline
                    slice = &slice[captures.get(0).unwrap().as_str().len()..];
                    closed_tag = true;
                    continue;
                }
            }
            // check open
            if let Some(captures) = self.open_matcher.captures(slice) {
                // we have open tag, create child and go deeper
                // if current node has no tag, use it's parent as the parent,
                // instead of creating child of just text
                let tag = captures.name("tag").unwrap().as_str();
                let curr_node_obj = tree.get_node(curr_node);
                // do not attempt to get parent of root node
                if curr_node_obj.tag == BBTag::None && curr_node != 0 {
                    curr_node = curr_node_obj.parent.unwrap();
                }
                let mut node = BBNode {
                    tag: BBTag::from(tag),
                    parent: Some(curr_node),
                    ..Default::default()
                };
                if let Some(val) = captures.name("val") {
                    node.value = Some(val.as_str().to_string());
                }
                let new_id = tree.add_node(node);
                tree.get_node_mut(curr_node).children.push(new_id);
                curr_node = new_id;

                // increment slice past open tag
                slice = &slice[captures.get(0).unwrap().as_str().len()..];
                closed_tag = false;
                continue;
            } else if let Some(captures) = self.close_matcher.captures(slice) {
                // if close tag, check current. If same, end child node and go back up. Otherwise toss the tag and keep going.
                let tag = captures.name("tag").unwrap().as_str();
                let bbtag = BBTag::from(tag);
                let curr_node_obj = tree.get_node(curr_node);
                if curr_node_obj.tag == BBTag::None && !curr_node_obj.text.is_empty() {
                    // current tag is only text, check the parent and close current if has text and matching,
                    // then close parent
                    let parent = tree.get_node(curr_node_obj.parent.unwrap());
                    if parent.tag == bbtag {
                        curr_node = parent.parent.unwrap();
                        slice = &slice[captures.get(0).unwrap().as_str().len()..];
                        closed_tag = true;
                        continue;
                    }
                }
                if bbtag == tree.get_node(curr_node).tag {
                    // matching open and close tags
                    // we're done with this node
                    curr_node = tree.get_node(curr_node).parent.unwrap();
                    // increment slice past close tag
                    slice = &slice[captures.get(0).unwrap().as_str().len()..];
                    closed_tag = true;
                    continue;
                } else {
                    // not a matching close tag, toss the tag and keep going
                    slice = &slice[captures.get(0).unwrap().as_str().len()..];
                    closed_tag = false;
                    continue;
                }
            }

            // no tags, grab text and continue
            if let Some(ch) = slice.chars().next() {
                if closed_tag {
                    // we just closed a tag but have more text to get, create a new node
                    let node = BBNode {
                        parent: Some(curr_node),
                        ..Default::default()
                    };
                    let new_id = tree.add_node(node);
                    tree.get_node_mut(curr_node).children.push(new_id);
                    curr_node = new_id;
                }

                tree.get_node_mut(curr_node).text.push(ch);
                slice = &slice[ch.len_utf8()..];
                closed_tag = false;
            } else {
                // end of the line
                break;
            }
        }

        tree
    }
}

