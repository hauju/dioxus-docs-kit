//! CodeGroup, RequestExample, and ResponseExample parsers.

use super::utils::{find_closing_tag, find_fenced_blocks};
use crate::parser::types::*;

/// Try to parse a CodeGroup container.
pub(super) fn try_parse_code_group(content: &str) -> Option<(DocNode, &str)> {
    if !content.starts_with("<CodeGroup>") {
        return None;
    }

    let after_open = &content["<CodeGroup>".len()..];
    let close_idx = find_closing_tag(after_open, "CodeGroup")?;
    let inner = &after_open[..close_idx];
    let rest = &after_open[close_idx + "</CodeGroup>".len()..];

    let blocks = parse_code_blocks(inner);

    Some((DocNode::CodeGroup(CodeGroupNode { blocks }), rest))
}

/// Try to parse a RequestExample container.
pub(super) fn try_parse_request_example(content: &str) -> Option<(DocNode, &str)> {
    if !content.starts_with("<RequestExample>") {
        return None;
    }

    let after_open = &content["<RequestExample>".len()..];
    let close_idx = find_closing_tag(after_open, "RequestExample")?;
    let inner = &after_open[..close_idx];
    let rest = &after_open[close_idx + "</RequestExample>".len()..];

    // Parse code blocks within
    let blocks = parse_code_blocks(inner);

    Some((DocNode::RequestExample(RequestExampleNode { blocks }), rest))
}

/// Try to parse a ResponseExample container.
pub(super) fn try_parse_response_example(content: &str) -> Option<(DocNode, &str)> {
    if !content.starts_with("<ResponseExample>") {
        return None;
    }

    let after_open = &content["<ResponseExample>".len()..];
    let close_idx = find_closing_tag(after_open, "ResponseExample")?;
    let inner = &after_open[..close_idx];
    let rest = &after_open[close_idx + "</ResponseExample>".len()..];

    let blocks = parse_code_blocks(inner);

    Some((
        DocNode::ResponseExample(ResponseExampleNode { blocks }),
        rest,
    ))
}

/// Parse fenced code blocks from content.
fn parse_code_blocks(content: &str) -> Vec<CodeBlockNode> {
    find_fenced_blocks(content)
        .into_iter()
        .map(|block| CodeBlockNode {
            language: block.language.map(str::to_string),
            filename: block.filename.map(str::to_string),
            code: block.code.trim().to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::try_parse_code_group;
    use crate::parser::content::parse_mdx;
    use crate::parser::types::*;

    #[test]
    fn test_parse_code_group() {
        let content = r#"<CodeGroup>
```bash cURL
curl https://example.com
```

```python Python
import requests
```
</CodeGroup>"#;
        let nodes = parse_mdx(content);
        assert_eq!(nodes.len(), 1);
        if let DocNode::CodeGroup(cg) = &nodes[0] {
            assert_eq!(cg.blocks.len(), 2);
            assert_eq!(cg.blocks[0].language, Some("bash".to_string()));
            assert_eq!(cg.blocks[0].filename, Some("cURL".to_string()));
            assert!(cg.blocks[0].code.contains("curl"));
            assert_eq!(cg.blocks[1].language, Some("python".to_string()));
        } else {
            panic!("Expected CodeGroup node");
        }
    }

    #[test]
    fn test_parse_request_example() {
        let content = r#"<RequestExample>
```bash
curl -X POST https://api.example.com
```
</RequestExample>"#;
        let nodes = parse_mdx(content);
        assert_eq!(nodes.len(), 1);
        if let DocNode::RequestExample(ex) = &nodes[0] {
            assert_eq!(ex.blocks.len(), 1);
            assert_eq!(ex.blocks[0].language, Some("bash".to_string()));
        } else {
            panic!("Expected RequestExample node");
        }
    }

    #[test]
    fn test_parse_response_example() {
        let content = r#"<ResponseExample>
```json
{"id": "123", "name": "Test"}
```
</ResponseExample>"#;
        let nodes = parse_mdx(content);
        assert_eq!(nodes.len(), 1);
        if let DocNode::ResponseExample(ex) = &nodes[0] {
            assert_eq!(ex.blocks.len(), 1);
            assert_eq!(ex.blocks[0].language, Some("json".to_string()));
        } else {
            panic!("Expected ResponseExample node");
        }
    }

    #[test]
    fn language_less_fence_keeps_its_body() {
        // `\s+` for the filename separator used to match across the newline,
        // turning the first code line into the tab label.
        let content = "<CodeGroup>\n```\ncurl https://example.com\n```\n</CodeGroup>";
        let (node, _) = try_parse_code_group(content).expect("CodeGroup");
        match node {
            DocNode::CodeGroup(group) => {
                assert_eq!(group.blocks.len(), 1);
                assert_eq!(group.blocks[0].filename, None);
                assert_eq!(group.blocks[0].code, "curl https://example.com");
            }
            other => panic!("expected CodeGroup, got {other:?}"),
        }
    }

    #[test]
    fn tilde_fence_inside_code_group_is_extracted() {
        let content = "<CodeGroup>\n~~~bash cURL\ncurl https://example.com\n~~~\n</CodeGroup>";
        let (node, _) = try_parse_code_group(content).expect("CodeGroup");
        match node {
            DocNode::CodeGroup(group) => {
                assert_eq!(group.blocks.len(), 1);
                assert_eq!(group.blocks[0].language.as_deref(), Some("bash"));
                assert_eq!(group.blocks[0].filename.as_deref(), Some("cURL"));
                assert_eq!(group.blocks[0].code, "curl https://example.com");
            }
            other => panic!("expected CodeGroup, got {other:?}"),
        }
    }

    #[test]
    fn crlf_fence_does_not_leak_carriage_return_into_filename() {
        let content = "<CodeGroup>\r\n```bash cURL\r\necho hi\r\n```\r\n</CodeGroup>";
        let (node, _) = try_parse_code_group(content).expect("CodeGroup");
        match node {
            DocNode::CodeGroup(group) => {
                assert_eq!(group.blocks[0].filename.as_deref(), Some("cURL"));
                assert_eq!(group.blocks[0].language.as_deref(), Some("bash"));
            }
            other => panic!("expected CodeGroup, got {other:?}"),
        }
    }
}
