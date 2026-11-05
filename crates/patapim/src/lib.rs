//! Patapim M1: a bounded local-document renderer with its own retained DOM.
mod parser;
pub use parser::{parse, ParseError, MAX_DOM_DEPTH, MAX_DOM_NODES, MAX_HTML_BYTES};
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub enum Node {
    Element {
        tag: String,
        style: String,
        children: Vec<Node>,
    },
    Text(String),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color(pub u8, pub u8, pub u8);
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Paint {
    Rect {
        x: i32,
        y: i32,
        w: u32,
        h: u32,
        color: Color,
    },
    Text {
        x: i32,
        y: i32,
        text: String,
        size: u32,
        color: Color,
    },
}
#[derive(Clone, Debug, Default)]
pub struct Metrics {
    pub parse: Duration,
    pub layout: Duration,
    pub nodes: usize,
    pub commands: usize,
}
pub struct Document {
    pub root: Node,
    pub display_list: Vec<Paint>,
    pub metrics: Metrics,
    pub viewport: (u32, u32),
}
#[derive(Clone, Copy)]
struct Style {
    color: Color,
    background: Option<Color>,
    font: u32,
    margin: u32,
    padding: u32,
}
impl Default for Style {
    fn default() -> Self {
        Self {
            color: Color(34, 44, 60),
            background: None,
            font: 16,
            margin: 0,
            padding: 0,
        }
    }
}

impl Document {
    pub fn from_local_html(html: &str, width: u32, height: u32) -> Result<Self, ParseError> {
        let start = Instant::now();
        let root = parse(html)?;
        let parse = start.elapsed();
        let mut doc = Self {
            root,
            display_list: Vec::new(),
            metrics: Metrics {
                parse,
                ..Metrics::default()
            },
            viewport: (width, height),
        };
        doc.rebuild();
        Ok(doc)
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        if self.viewport != (width, height) {
            self.viewport = (width, height);
            self.rebuild();
        }
    }
    fn rebuild(&mut self) {
        let start = Instant::now();
        self.display_list.clear();
        self.display_list.push(Paint::Rect {
            x: 0,
            y: 0,
            w: self.viewport.0,
            h: self.viewport.1,
            color: Color(245, 246, 250),
        });
        let mut y = 16;
        let mut count = 0;
        layout(
            &self.root,
            Style::default(),
            24,
            &mut y,
            self.viewport.0.saturating_sub(48),
            &mut self.display_list,
            &mut count,
        );
        self.metrics.layout = start.elapsed();
        self.metrics.nodes = count;
        self.metrics.commands = self.display_list.len();
    }
}

fn color(s: &str) -> Option<Color> {
    let hex = s.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let n = u32::from_str_radix(hex, 16).ok()?;
    Some(Color((n >> 16) as u8, (n >> 8) as u8, n as u8))
}
fn px(s: &str) -> Option<u32> {
    s.trim().strip_suffix("px").unwrap_or(s.trim()).parse().ok()
}
fn style(parent: Style, tag: &str, attr: &str) -> Style {
    let mut s = parent;
    s.background = None;
    s.margin = match tag {
        "h1" | "h2" | "h3" => 14,
        "p" | "li" => 8,
        _ => 0,
    };
    s.font = match tag {
        "h1" => 32,
        "h2" => 25,
        "h3" => 20,
        _ => parent.font,
    };
    for d in attr.split(';') {
        let Some((k, v)) = d.split_once(':') else {
            continue;
        };
        let v = v.trim();
        match k.trim().to_ascii_lowercase().as_str() {
            "color" => {
                if let Some(c) = color(v) {
                    s.color = c
                }
            }
            "background" | "background-color" => {
                if let Some(c) = color(v) {
                    s.background = Some(c)
                }
            }
            "font-size" => {
                if let Some(n) = px(v) {
                    s.font = n.clamp(8, 72)
                }
            }
            "margin" => {
                if let Some(n) = px(v) {
                    s.margin = n.min(64)
                }
            }
            "padding" => {
                if let Some(n) = px(v) {
                    s.padding = n.min(64)
                }
            }
            _ => {}
        }
    }
    s
}
fn layout(
    node: &Node,
    parent: Style,
    x: i32,
    y: &mut i32,
    width: u32,
    out: &mut Vec<Paint>,
    count: &mut usize,
) {
    *count += 1;
    match node {
        Node::Text(text) => {
            let max = (width / (parent.font * 3 / 5).max(1)).max(1) as usize;
            let mut line = String::new();
            for word in text.split_whitespace() {
                if !line.is_empty() && line.chars().count() + word.chars().count() + 1 > max {
                    out.push(Paint::Text {
                        x,
                        y: *y,
                        text: std::mem::take(&mut line),
                        size: parent.font,
                        color: parent.color,
                    });
                    *y += parent.font as i32 + 8;
                }
                if !line.is_empty() {
                    line.push(' ')
                }
                line.push_str(word);
            }
            if !line.is_empty() {
                out.push(Paint::Text {
                    x,
                    y: *y,
                    text: line,
                    size: parent.font,
                    color: parent.color,
                });
                *y += parent.font as i32 + 8
            }
        }
        Node::Element {
            tag,
            style: attr,
            children,
        } => {
            let s = style(parent, tag, attr);
            let block = matches!(tag.as_str(), "div" | "p" | "h1" | "h2" | "h3" | "ul" | "li");
            if block {
                *y += s.margin as i32
            }
            if let Some(c) = s.background {
                out.push(Paint::Rect {
                    x,
                    y: *y,
                    w: width,
                    h: s.font + s.padding * 2 + 12,
                    color: c,
                })
            }
            *y += s.padding as i32;
            if tag == "br" {
                *y += s.font as i32 + 8
            }
            for child in children {
                layout(
                    child,
                    s,
                    x + s.padding as i32,
                    y,
                    width.saturating_sub(s.padding * 2),
                    out,
                    count,
                )
            }
            if block {
                *y += s.margin as i32
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn renders_local_subset() {
        let d = Document::from_local_html(
            "<h1 style=\"color:#ff0000\">Hello</h1><p>A &amp; B</p>",
            640,
            480,
        )
        .unwrap();
        assert!(d.metrics.nodes >= 5);
        assert!(d
            .display_list
            .iter()
            .any(|p| matches!(p,Paint::Text{text,color:Color(255,0,0),..} if text=="Hello")));
    }
    #[test]
    fn resize_rebuilds() {
        let mut d = Document::from_local_html("<p>Hello</p>", 320, 200).unwrap();
        d.resize(640, 200);
        assert!(matches!(d.display_list[0], Paint::Rect { w: 640, .. }));
    }
    #[test]
    fn script_content_is_inert() {
        let d =
            Document::from_local_html("<script>alert(1)</script><p>Safe</p>", 320, 200).unwrap();
        assert!(!d
            .display_list
            .iter()
            .any(|p| matches!(p,Paint::Text{text,..} if text.contains("alert"))));
    }
}
