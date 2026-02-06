//! ParamField, ResponseField, and Expandable parsers.

use super::content::parse_content;
use super::utils::{extract_attr, find_closing_tag};
use crate::parser::types::*;

/// Try to parse a ParamField component.
/// Handles: `<ParamField header="X-API-Key" type="string" required>Description</ParamField>`
/// Or: `<ParamField path="project_id" type="string" required>Description</ParamField>`
pub(super) fn try_parse_param_field(content: &str) -> Option<(DocNode, &str)> {
    if !content.starts_with("<ParamField") {
        return None;
    }

    let tag_end = content.find('>')?;
    let tag_content = &content[11..tag_end]; // Skip "<ParamField"

    // Determine location from attribute name
    let (name, location) = if let Some(name) = extract_attr(tag_content, "header") {
        (name, ParamLocation::Header)
    } else if let Some(name) = extract_attr(tag_content, "path") {
        (name, ParamLocation::Path)
    } else if let Some(name) = extract_attr(tag_content, "query") {
        (name, ParamLocation::Query)
    } else if let Some(name) = extract_attr(tag_content, "body") {
        (name, ParamLocation::Body)
    } else {
        return None;
    };

    let param_type = extract_attr(tag_content, "type").unwrap_or_else(|| "string".to_string());
    let required = tag_content.contains("required");
    let default = extract_attr(tag_content, "default");

    // Handle self-closing vs block
    if tag_content.trim().ends_with('/') {
        return Some((
            DocNode::ParamField(ParamFieldNode {
                name,
                location,
                param_type,
                required,
                default,
                content: Vec::new(),
            }),
            &content[tag_end + 1..],
        ));
    }

    // Find closing tag
    let after_open = &content[tag_end + 1..];
    let close_idx = find_closing_tag(after_open, "ParamField")?;
    let inner = after_open[..close_idx].trim();
    let rest = &after_open[close_idx + "</ParamField>".len()..];

    // Parse inner content recursively to handle nested components
    let parsed_content = parse_content(inner);

    Some((
        DocNode::ParamField(ParamFieldNode {
            name,
            location,
            param_type,
            required,
            default,
            content: parsed_content,
        }),
        rest,
    ))
}

/// Try to parse a ResponseField component with potential nested Expandable.
pub(super) fn try_parse_response_field(content: &str) -> Option<(DocNode, &str)> {
    if !content.starts_with("<ResponseField") {
        return None;
    }

    let tag_end = content.find('>')?;
    let tag_content = &content[14..tag_end]; // Skip "<ResponseField"

    // Handle self-closing
    if tag_content.trim().ends_with('/') {
        let name = extract_attr(tag_content, "name")?;
        let field_type = extract_attr(tag_content, "type").unwrap_or_else(|| "any".to_string());
        let required = tag_content.contains("required");
        return Some((
            DocNode::ResponseField(ResponseFieldNode {
                name,
                field_type,
                required,
                content: String::new(),
                expandable: None,
            }),
            &content[tag_end + 1..],
        ));
    }

    let name = extract_attr(tag_content, "name")?;
    let field_type = extract_attr(tag_content, "type").unwrap_or_else(|| "any".to_string());
    let required = tag_content.contains("required");

    let after_open = &content[tag_end + 1..];
    let close_idx = find_closing_tag(after_open, "ResponseField")?;
    let inner = &after_open[..close_idx];
    let rest = &after_open[close_idx + "</ResponseField>".len()..];

    // Check for nested Expandable
    let expandable = parse_nested_expandable(inner);
    let content_text = if expandable.is_some() {
        // Extract text before <Expandable>
        if let Some(exp_start) = inner.find("<Expandable") {
            inner[..exp_start].trim().to_string()
        } else {
            inner.trim().to_string()
        }
    } else {
        inner.trim().to_string()
    };

    Some((
        DocNode::ResponseField(ResponseFieldNode {
            name,
            field_type,
            required,
            content: content_text,
            expandable,
        }),
        rest,
    ))
}

/// Try to parse an Expandable component (standalone, not nested in ResponseField).
pub(super) fn try_parse_expandable(content: &str) -> Option<(DocNode, &str)> {
    if !content.starts_with("<Expandable") {
        return None;
    }

    let tag_end = content.find('>')?;
    let tag_content = &content[11..tag_end]; // Skip "<Expandable"

    let title = extract_attr(tag_content, "title").unwrap_or_else(|| "Details".to_string());

    let after_open = &content[tag_end + 1..];
    let close_idx = find_closing_tag(after_open, "Expandable")?;
    let inner = &after_open[..close_idx];
    let rest = &after_open[close_idx + "</Expandable>".len()..];

    // Parse nested ResponseFields
    let fields = parse_response_fields(inner);

    Some((DocNode::Expandable(ExpandableNode { title, fields }), rest))
}

/// Parse a nested Expandable section within ResponseField.
fn parse_nested_expandable(content: &str) -> Option<ExpandableNode> {
    let start = content.find("<Expandable")?;
    let tag_end = content[start..].find('>')? + start;
    let tag_content = &content[start + 11..tag_end]; // Skip "<Expandable"

    let title = extract_attr(tag_content, "title").unwrap_or_else(|| "Properties".to_string());

    let after_open = &content[tag_end + 1..];
    let close_idx = find_closing_tag(after_open, "Expandable")?;
    let inner = &after_open[..close_idx];

    // Parse nested ResponseFields
    let fields = parse_response_fields(inner);

    Some(ExpandableNode { title, fields })
}

/// Parse multiple ResponseField elements from content.
fn parse_response_fields(content: &str) -> Vec<ResponseFieldNode> {
    let mut fields = Vec::new();
    let mut remaining = content.trim();

    while !remaining.is_empty() {
        remaining = remaining.trim();

        if remaining.starts_with("<ResponseField")
            && let Some((DocNode::ResponseField(field), rest)) = try_parse_response_field(remaining)
        {
            fields.push(field);
            remaining = rest;
            continue;
        }

        // Skip to next ResponseField
        if let Some(idx) = remaining[1..].find("<ResponseField") {
            remaining = &remaining[idx + 1..];
        } else {
            break;
        }
    }

    fields
}

#[cfg(test)]
mod tests {
    use crate::parser::content::parse_mdx;
    use crate::parser::types::*;

    #[test]
    fn test_parse_param_field_path() {
        let content = r#"<ParamField path="project_id" type="string" required>
  The project identifier.
</ParamField>"#;
        let nodes = parse_mdx(content);
        assert_eq!(nodes.len(), 1);
        if let DocNode::ParamField(f) = &nodes[0] {
            assert_eq!(f.name, "project_id");
            assert_eq!(f.location, ParamLocation::Path);
            assert_eq!(f.param_type, "string");
            assert!(f.required);
            // Content is now a Vec<DocNode>
            assert_eq!(f.content.len(), 1);
            if let DocNode::Markdown(md) = &f.content[0] {
                assert!(md.contains("project identifier"));
            } else {
                panic!("Expected Markdown node in content");
            }
        } else {
            panic!("Expected ParamField node");
        }
    }

    #[test]
    fn test_parse_param_field_header() {
        let content = r#"<ParamField header="X-API-Key" type="string" required>
  Your API key.
</ParamField>"#;
        let nodes = parse_mdx(content);
        assert_eq!(nodes.len(), 1);
        if let DocNode::ParamField(f) = &nodes[0] {
            assert_eq!(f.name, "X-API-Key");
            assert_eq!(f.location, ParamLocation::Header);
            assert!(f.required);
            assert!(!f.content.is_empty());
        } else {
            panic!("Expected ParamField node");
        }
    }

    #[test]
    fn test_parse_param_field_with_default() {
        let content = r#"<ParamField query="limit" type="integer" default="10">
  Items per page.
</ParamField>"#;
        let nodes = parse_mdx(content);
        assert_eq!(nodes.len(), 1);
        if let DocNode::ParamField(f) = &nodes[0] {
            assert_eq!(f.name, "limit");
            assert_eq!(f.location, ParamLocation::Query);
            assert_eq!(f.default, Some("10".to_string()));
            assert!(!f.required);
        } else {
            panic!("Expected ParamField node");
        }
    }

    #[test]
    fn test_parse_response_field_simple() {
        let content = r#"<ResponseField name="id" type="string" required>
  The unique identifier.
</ResponseField>"#;
        let nodes = parse_mdx(content);
        assert_eq!(nodes.len(), 1);
        if let DocNode::ResponseField(f) = &nodes[0] {
            assert_eq!(f.name, "id");
            assert_eq!(f.field_type, "string");
            assert!(f.required);
            assert!(f.expandable.is_none());
        } else {
            panic!("Expected ResponseField node");
        }
    }

    #[test]
    fn test_parse_response_field_with_expandable() {
        let content = r#"<ResponseField name="data" type="object">
  The response data.
  <Expandable title="Data properties">
    <ResponseField name="id" type="string">The ID</ResponseField>
    <ResponseField name="name" type="string">The name</ResponseField>
  </Expandable>
</ResponseField>"#;
        let nodes = parse_mdx(content);
        assert_eq!(nodes.len(), 1);
        if let DocNode::ResponseField(f) = &nodes[0] {
            assert_eq!(f.name, "data");
            assert_eq!(f.field_type, "object");
            assert!(f.expandable.is_some());
            let exp = f.expandable.as_ref().unwrap();
            assert_eq!(exp.title, "Data properties");
            assert_eq!(exp.fields.len(), 2);
            assert_eq!(exp.fields[0].name, "id");
            assert_eq!(exp.fields[1].name, "name");
        } else {
            panic!("Expected ResponseField node");
        }
    }

    #[test]
    fn test_multiple_param_fields() {
        let content = r#"<ParamField header="Authorization" type="string" required>
  Bearer token.
</ParamField>

<ParamField path="project_id" type="string" required>
  Project ID.
</ParamField>

<ParamField query="page" type="integer" default="1">
  Page number.
</ParamField>"#;
        let nodes = parse_mdx(content);
        assert_eq!(nodes.len(), 3);
        assert!(matches!(&nodes[0], DocNode::ParamField(_)));
        assert!(matches!(&nodes[1], DocNode::ParamField(_)));
        assert!(matches!(&nodes[2], DocNode::ParamField(_)));
    }

    #[test]
    fn test_nested_note_in_param_field() {
        let content = r#"<ParamField path="data-button-position" type="string" default="bottom-right">
  Control where the feedback button appears.

  <Note>
    The `icon-only` position is perfect for minimalist designs.
  </Note>
</ParamField>"#;
        let nodes = parse_mdx(content);
        assert_eq!(nodes.len(), 1);
        if let DocNode::ParamField(f) = &nodes[0] {
            assert_eq!(f.name, "data-button-position");
            // Should have markdown and a nested Note
            assert!(f.content.len() >= 2);
            // Find the Note in the content
            let has_note = f.content.iter().any(|n| matches!(n, DocNode::Callout(c) if c.callout_type == CalloutType::Note));
            assert!(has_note, "Expected nested Note callout in ParamField content");
        } else {
            panic!("Expected ParamField node");
        }
    }

    #[test]
    fn test_nested_card_in_param_field() {
        let content = r#"<ParamField path="data-language" type="string" default="auto-detect">
  Set the interface language.

  <Card title="Multi-language support" icon="globe" href="/widget/i18n">
    Learn more about internationalization
  </Card>
</ParamField>"#;
        let nodes = parse_mdx(content);
        assert_eq!(nodes.len(), 1);
        if let DocNode::ParamField(f) = &nodes[0] {
            assert_eq!(f.name, "data-language");
            // Find the Card in the content
            let has_card = f.content.iter().any(|n| matches!(n, DocNode::CardGroup(_)));
            assert!(has_card, "Expected nested Card in ParamField content");
        } else {
            panic!("Expected ParamField node");
        }
    }
}
