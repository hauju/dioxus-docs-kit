//! Callout (Tip, Note, Warning, Info) parser.

use regex::Regex;

use crate::parser::types::*;

/// Try to parse a callout (Tip, Note, Warning, Info).
pub(super) fn try_parse_callout(content: &str) -> Option<(DocNode, &str)> {
    // Match opening tag to determine callout type
    let open_re = Regex::new(r"^<(Tip|Note|Warning|Info)>").unwrap();

    let caps = open_re.captures(content)?;
    let tag_name = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
    let callout_type = CalloutType::parse(tag_name)?;

    let open_match = caps.get(0).expect("regex group 0");
    let after_open = &content[open_match.end()..];

    // Find the matching closing tag
    let close_tag = format!("</{}>", tag_name);
    let close_idx = after_open.find(&close_tag)?;

    let inner = after_open[..close_idx].trim().to_string();
    let rest = &after_open[close_idx + close_tag.len()..];

    Some((
        DocNode::Callout(CalloutNode {
            callout_type,
            content: inner,
        }),
        rest,
    ))
}

#[cfg(test)]
mod tests {
    use crate::parser::content::parse_mdx;
    use crate::parser::types::*;

    #[test]
    fn test_parse_callout() {
        let content = "<Warning>\nDon't do this!\n</Warning>";
        let nodes = parse_mdx(content);
        assert_eq!(nodes.len(), 1);
        if let DocNode::Callout(c) = &nodes[0] {
            assert_eq!(c.callout_type, CalloutType::Warning);
            assert_eq!(c.content, "Don't do this!");
        } else {
            panic!("Expected Callout node");
        }
    }
}
