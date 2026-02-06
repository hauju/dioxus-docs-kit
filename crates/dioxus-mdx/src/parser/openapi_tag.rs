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
