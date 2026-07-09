//! OpenAPI specification tag parser.

use super::utils::{extract_attr, find_closing_tag};
use crate::parser::openapi_parser::parse_openapi;
use crate::parser::types::*;

/// Try to parse an OpenAPI specification component.
/// Handles: `<OpenAPI src="/api/spec.yaml" />` or `<OpenAPI>yaml content</OpenAPI>`
pub(super) fn try_parse_openapi(content: &str) -> Option<(DocNode, &str)> {
    if !content.starts_with("<OpenAPI") {
        return None;
    }

    let tag_end = content.find('>')?;
    let tag_content = &content[8..tag_end]; // Skip "<OpenAPI"

    // Extract attributes
    let tags_attr = extract_attr(tag_content, "tags");
    let hide_schemas = tag_content.contains("hideSchemas");
    let show_schemas = !hide_schemas;

    // Parse tags filter
    let tags = tags_attr.map(|t| t.split(',').map(|s| s.trim().to_string()).collect());

    // Check if self-closing with src attribute
    if tag_content.trim().ends_with('/') {
        // Self-closing tag with src attribute - spec content should be embedded
        // For now, return an error node since we can't fetch files at parse time
        // The src attribute would need to be handled at a higher level
        return None;
    }

    // Block tag - spec content is inline
    let after_open = &content[tag_end + 1..];
    let close_idx = find_closing_tag(after_open, "OpenAPI")?;
    let inner = after_open[..close_idx].trim();
    let rest = &after_open[close_idx + "</OpenAPI>".len()..];

    // Parse the OpenAPI spec
    let spec = parse_openapi(inner).ok()?;

    Some((
        DocNode::OpenApi(OpenApiNode {
            spec,
            tags,
            show_schemas,
        }),
        rest,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC: &str = r#"openapi: "3.0.0"
info:
  title: Test API
  version: "1.0.0"
paths:
  /pets:
    get:
      summary: List pets
      responses:
        "200":
          description: OK
"#;

    #[test]
    fn parses_inline_openapi_block() {
        let content = format!("<OpenAPI>\n{SPEC}\n</OpenAPI>after");
        let (node, rest) = try_parse_openapi(&content).unwrap();
        match node {
            DocNode::OpenApi(openapi) => {
                assert_eq!(openapi.spec.info.title, "Test API");
                assert!(openapi.tags.is_none());
                assert!(openapi.show_schemas);
            }
            other => panic!("expected OpenApi node, got {other:?}"),
        }
        assert_eq!(rest, "after");
    }

    #[test]
    fn parses_tags_filter_and_hide_schemas() {
        let content = format!("<OpenAPI tags=\"pets, store\" hideSchemas>\n{SPEC}\n</OpenAPI>");
        let (node, _) = try_parse_openapi(&content).unwrap();
        match node {
            DocNode::OpenApi(openapi) => {
                assert_eq!(
                    openapi.tags,
                    Some(vec!["pets".to_string(), "store".to_string()])
                );
                assert!(!openapi.show_schemas);
            }
            other => panic!("expected OpenApi node, got {other:?}"),
        }
    }

    #[test]
    fn invalid_spec_or_non_openapi_returns_none() {
        assert!(try_parse_openapi("plain text").is_none());
        assert!(try_parse_openapi("<OpenAPI>not a spec</OpenAPI>").is_none());
        // Self-closing src form is not supported at parse time.
        assert!(try_parse_openapi("<OpenAPI src=\"/spec.yaml\" />").is_none());
    }
}
