//! MDX documentation parser for extracting frontmatter and components.
//!
//! This module provides functionality to parse MDX (Markdown with JSX) content,
//! extracting YAML frontmatter and converting custom components like Cards, Tabs,
//! Steps, and Callouts into an intermediate representation for rendering.

mod accordion;
mod callout;
mod card;
mod code_group;
mod content;
mod fields;
mod frontmatter;
mod openapi_parser;
mod openapi_tag;
mod openapi_types;
mod steps;
mod syntax;
mod tabs;
mod types;
mod update;
mod utils;

pub use content::{get_raw_markdown, parse_mdx};
pub use frontmatter::extract_frontmatter;
pub use openapi_parser::{parse_openapi, OpenApiError};
pub use openapi_types::*;
pub use syntax::highlight_code;
pub use types::*;

/// Parse a complete MDX document, extracting frontmatter and content.
///
/// This is the main entry point for parsing MDX content. It extracts
/// YAML frontmatter from the beginning of the document and parses the
/// remaining content into a tree of `DocNode` elements.
pub fn parse_document(content: &str) -> ParsedDoc {
    let (frontmatter, remaining) = extract_frontmatter(content);
    let nodes = parse_mdx(remaining);
    let raw_markdown = get_raw_markdown(&nodes);

    ParsedDoc {
        frontmatter,
        content: nodes,
        raw_markdown,
    }
}
