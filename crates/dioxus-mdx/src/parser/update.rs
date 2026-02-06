//! Update (changelog entry) parser.

use super::content::parse_content;
use super::utils::{extract_attr, find_closing_tag};
use crate::parser::types::*;

/// Try to parse an Update (changelog) entry.
pub(super) fn try_parse_update(content: &str) -> Option<(DocNode, &str)> {
    if !content.starts_with("<Update") {
        return None;
    }

    let tag_end = content.find('>')?;
    let tag_content = &content[7..tag_end]; // Skip "<Update"

    let label = extract_attr(tag_content, "label")?;
    let description = extract_attr(tag_content, "description").unwrap_or_default();

    let after_open = &content[tag_end + 1..];
    let close_idx = find_closing_tag(after_open, "Update")?;
    let inner = after_open[..close_idx].trim();
    let rest = &after_open[close_idx + "</Update>".len()..];

    // Parse inner content recursively
    let parsed_content = parse_content(inner);

    Some((
        DocNode::Update(UpdateNode {
            label,
            description,
            content: parsed_content,
        }),
        rest,
    ))
}

#[cfg(test)]
mod tests {
    use crate::parser::content::{get_raw_markdown, parse_mdx};
    use crate::parser::types::*;

    #[test]
    fn test_parse_update() {
        let content = r#"<Update label="v0.9.0" description="December 2025">
- New feature A
- Bug fix B
</Update>"#;
        let nodes = parse_mdx(content);
        assert_eq!(nodes.len(), 1);
        if let DocNode::Update(u) = &nodes[0] {
            assert_eq!(u.label, "v0.9.0");
            assert_eq!(u.description, "December 2025");
            // Content is now a Vec<DocNode>
            assert!(!u.content.is_empty());
            let raw = get_raw_markdown(&u.content);
            assert!(raw.contains("New feature A"));
            assert!(raw.contains("Bug fix B"));
        } else {
            panic!("Expected Update node");
        }
    }
}
