//! Tabs component parser.

use regex::Regex;

use super::content::parse_content;
use super::utils::find_closing_tag;
use crate::parser::types::*;

/// Try to parse a Tabs component.
pub(super) fn try_parse_tabs(content: &str) -> Option<(DocNode, &str)> {
    if !content.starts_with("<Tabs>") {
        return None;
    }

    let after_open = &content["<Tabs>".len()..];
    let close_idx = find_closing_tag(after_open, "Tabs")?;
    let inner = &after_open[..close_idx];
    let rest = &after_open[close_idx + "</Tabs>".len()..];

    // Parse inner tabs
    let tabs = parse_tabs(inner);

    Some((DocNode::Tabs(TabsNode { tabs }), rest))
}

/// Parse Tab elements from content.
fn parse_tabs(content: &str) -> Vec<TabNode> {
    let mut tabs = Vec::new();
    let mut remaining = content.trim();

    let tab_open_re = Regex::new(r#"^<Tab\s+title="([^"]*)">"#).unwrap();

    while !remaining.is_empty() {
        remaining = remaining.trim();

        if let Some(caps) = tab_open_re.captures(remaining) {
            let full_match = caps.get(0).expect("regex group 0");
            let title = caps
                .get(1)
                .map(|m| m.as_str())
                .unwrap_or_default()
                .to_string();

            let after_open = &remaining[full_match.end()..];
            if let Some(close_idx) = find_closing_tag(after_open, "Tab") {
                let inner = after_open[..close_idx].trim();
                // Parse inner content recursively
                let parsed_content = parse_content(inner);
                tabs.push(TabNode {
                    title,
                    content: parsed_content,
                });
                remaining = &after_open[close_idx + "</Tab>".len()..];
                continue;
            }
        }

        // Skip to next Tab
        if let Some(idx) = remaining[1..].find("<Tab") {
            remaining = &remaining[idx + 1..];
        } else {
            break;
        }
    }

    tabs
}

#[cfg(test)]
mod tests {
    use crate::parser::content::parse_mdx;
    use crate::parser::types::*;

    #[test]
    fn test_parse_tabs() {
        let content = r#"<Tabs>
  <Tab title="macOS">Mac instructions</Tab>
  <Tab title="Windows">Windows instructions</Tab>
</Tabs>"#;

        let nodes = parse_mdx(content);
        assert_eq!(nodes.len(), 1);
        if let DocNode::Tabs(t) = &nodes[0] {
            assert_eq!(t.tabs.len(), 2);
            assert_eq!(t.tabs[0].title, "macOS");
            assert_eq!(t.tabs[1].title, "Windows");
        } else {
            panic!("Expected Tabs node");
        }
    }

    #[test]
    fn test_code_block_in_tab() {
        let content = r#"<Tabs>
  <Tab title="JavaScript">
    ```js
    console.log("hello");
    ```
  </Tab>
</Tabs>"#;
        let nodes = parse_mdx(content);
        assert_eq!(nodes.len(), 1);
        if let DocNode::Tabs(t) = &nodes[0] {
            assert_eq!(t.tabs.len(), 1);
            // Tab content should contain the code block
            let has_code_block = t.tabs[0]
                .content
                .iter()
                .any(|n| matches!(n, DocNode::CodeBlock(_)));
            assert!(has_code_block, "Expected CodeBlock in Tab content");
        } else {
            panic!("Expected Tabs node");
        }
    }
}
