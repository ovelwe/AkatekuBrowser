//! HTML5 tree construction is delegated to html5ever; the retained DOM is Patapim's.
use crate::Node;
use html5ever::{parse_document, tendril::TendrilSink, ParseOpts};
use markup5ever_rcdom::{Handle, NodeData, RcDom};

pub const MAX_HTML_BYTES: usize = 1024 * 1024;
pub const MAX_DOM_NODES: usize = 20_000;
pub const MAX_DOM_DEPTH: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseError {
    InputTooLarge,
    TreeTooLarge,
    TooDeep,
}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InputTooLarge => "HTML exceeds the 1 MiB input limit",
            Self::TreeTooLarge => "HTML exceeds the 20,000 node limit",
            Self::TooDeep => "HTML exceeds the 64-level nesting limit",
        })
    }
}
impl std::error::Error for ParseError {}

pub fn parse(input: &str) -> Result<Node, ParseError> {
    if input.len() > MAX_HTML_BYTES {
        return Err(ParseError::InputTooLarge);
    }
    let dom = parse_document(RcDom::default(), ParseOpts::default()).one(input.to_owned());
    // Validate the entire parsed tree, including subtrees that will not be painted.
    // This is a post-parse gate, not a memory/time sandbox for html5ever itself.
    let mut pending = vec![(dom.document.clone(), 0usize)];
    let mut seen = 0;
    while let Some((node, depth)) = pending.pop() {
        seen += 1;
        if seen > MAX_DOM_NODES {
            return Err(ParseError::TreeTooLarge);
        }
        if depth > MAX_DOM_DEPTH {
            return Err(ParseError::TooDeep);
        }
        for child in node.children.borrow().iter() {
            pending.push((child.clone(), depth + 1));
        }
        if let NodeData::Element {
            template_contents, ..
        } = &node.data
        {
            if let Some(contents) = template_contents.borrow().as_ref() {
                pending.push((contents.clone(), depth + 1));
            }
        }
    }
    let mut remaining = MAX_DOM_NODES;
    convert(&dom.document, 0, &mut remaining)?.ok_or(ParseError::TreeTooLarge)
}

fn convert(
    handle: &Handle,
    depth: usize,
    remaining: &mut usize,
) -> Result<Option<Node>, ParseError> {
    if depth > MAX_DOM_DEPTH {
        return Err(ParseError::TooDeep);
    }
    if *remaining == 0 {
        return Err(ParseError::TreeTooLarge);
    }
    *remaining -= 1;
    match &handle.data {
        NodeData::Document => Ok(Some(Node::Element {
            tag: "document".into(),
            style: String::new(),
            children: convert_children(handle, depth, remaining)?,
        })),
        NodeData::Text { contents } => {
            let text = contents.borrow().to_string();
            Ok((!text.trim().is_empty()).then_some(Node::Text(text)))
        }
        NodeData::Element { name, attrs, .. } => {
            if name.ns.as_ref() != "http://www.w3.org/1999/xhtml" {
                return Ok(None);
            }
            let tag = name.local.as_ref();
            if matches!(
                tag,
                "head"
                    | "script"
                    | "style"
                    | "iframe"
                    | "object"
                    | "embed"
                    | "template"
                    | "svg"
                    | "math"
                    | "noscript"
            ) {
                return Ok(None);
            }
            let style = if supported(tag) {
                attrs
                    .borrow()
                    .iter()
                    .find(|a| a.name.local.as_ref() == "style")
                    .map_or_else(String::new, |a| a.value.to_string())
            } else {
                String::new()
            };
            // Unknown passive elements are transparent and cannot add browser behavior.
            let tag = if supported(tag) { tag } else { "span" };
            Ok(Some(Node::Element {
                tag: tag.into(),
                style,
                children: convert_children(handle, depth, remaining)?,
            }))
        }
        _ => Ok(None),
    }
}

fn convert_children(
    handle: &Handle,
    depth: usize,
    remaining: &mut usize,
) -> Result<Vec<Node>, ParseError> {
    let mut out = Vec::new();
    for child in handle.children.borrow().iter() {
        if let Some(node) = convert(child, depth + 1, remaining)? {
            out.push(node);
        }
    }
    Ok(out)
}

fn supported(tag: &str) -> bool {
    matches!(
        tag,
        "html"
            | "body"
            | "div"
            | "p"
            | "h1"
            | "h2"
            | "h3"
            | "span"
            | "strong"
            | "em"
            | "ul"
            | "li"
            | "br"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn text(node: &Node) -> String {
        match node {
            Node::Text(t) => t.clone(),
            Node::Element { children, .. } => children.iter().map(text).collect(),
        }
    }
    fn elements<'a>(node: &'a Node, tag: &str, out: &mut Vec<&'a Node>) {
        if let Node::Element {
            tag: name,
            children,
            ..
        } = node
        {
            if name == tag {
                out.push(node);
            }
            for child in children {
                elements(child, tag, out);
            }
        }
    }
    #[test]
    fn decodes_named_numeric_and_unicode_entities() {
        let dom = parse("<p>&#65; &#x1F680; &copy; &amp; Привет</p>").unwrap();
        assert_eq!(text(&dom), "A 🚀 © & Привет");
    }
    #[test]
    fn recovers_omitted_paragraph_end_tags() {
        let dom = parse("<p>one<p>two").unwrap();
        let mut paragraphs = vec![];
        elements(&dom, "p", &mut paragraphs);
        assert_eq!(paragraphs.len(), 2);
        assert_eq!(text(paragraphs[0]), "one");
        assert_eq!(text(paragraphs[1]), "two");
    }
    #[test]
    fn quoted_greater_than_does_not_end_tag() {
        let dom = parse("<p title='a > b' STYLE = 'color:#123456'>ok</p>").unwrap();
        let mut nodes = vec![];
        elements(&dom, "p", &mut nodes);
        assert!(matches!(nodes[0], Node::Element { style, .. } if style == "color:#123456"));
        assert_eq!(text(&dom), "ok");
    }
    #[test]
    fn attribute_names_are_matched_exactly() {
        let dom = parse("<p data-style='color:#ff0000'>ok</p>").unwrap();
        let mut nodes = vec![];
        elements(&dom, "p", &mut nodes);
        assert!(matches!(nodes[0], Node::Element { style, .. } if style.is_empty()));
    }
    #[test]
    fn active_and_metadata_subtrees_never_reach_render_dom() {
        let dom = parse("<title>hidden</title><script>hidden</script ><style>hidden</style><template><p>hidden</p></template><svg><text>hidden</text></svg><iframe>hidden</iframe><p>visible</p>").unwrap();
        assert_eq!(text(&dom), "visible");
    }
    #[test]
    fn rejects_input_without_silent_truncation() {
        assert!(matches!(
            parse(&"x".repeat(MAX_HTML_BYTES + 1)),
            Err(ParseError::InputTooLarge)
        ));
    }
    #[test]
    fn rejects_deep_and_large_trees() {
        assert!(matches!(
            parse(&"<div>".repeat(80)),
            Err(ParseError::TooDeep)
        ));
        assert!(matches!(
            parse(&"<br>".repeat(MAX_DOM_NODES)),
            Err(ParseError::TreeTooLarge)
        ));
    }
    #[test]
    fn malformed_input_mutation_smoke() {
        let fragments = [
            "<",
            ">",
            "</p>",
            "<!--",
            "-->",
            "&amp;",
            "&#x0;",
            "\0",
            "Привет",
            "<script>",
            "</script>",
            "<p a='>",
            "<table>",
            "<b>",
            "</i>",
        ];
        let mut seed = 7u64;
        for _ in 0..256 {
            let mut input = String::new();
            for _ in 0..32 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                input.push_str(fragments[(seed as usize) % fragments.len()]);
            }
            let result = parse(&input);
            assert!(
                result.is_ok()
                    || matches!(result, Err(ParseError::TooDeep | ParseError::TreeTooLarge))
            );
        }
    }
}
