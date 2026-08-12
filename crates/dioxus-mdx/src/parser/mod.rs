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
mod heading;
mod openapi_parser;
mod openapi_tag;
mod openapi_types;
mod steps;
mod tabs;
mod types;
mod update;
mod utils;

pub use content::{get_raw_markdown, parse_mdx};
pub use frontmatter::extract_frontmatter;
pub use heading::strip_leading_h1;
pub use openapi_parser::{OpenApiError, parse_openapi};
pub use openapi_types::*;
pub use types::*;

/// Parse a complete MDX document, extracting frontmatter and content.
///
/// This is the main entry point for parsing MDX content. It extracts
/// YAML frontmatter from the beginning of the document and parses the
/// remaining content into a tree of `DocNode` elements.
pub fn parse_document(content: &str) -> ParsedDoc {
    let (frontmatter, remaining) = extract_frontmatter(content);
    // Consumer layouts render the frontmatter title in their own <h1>; drop a
    // duplicate body H1 so the page emits exactly one.
    let body = strip_leading_h1(remaining);
    // Frontmatter is already gone; parse_mdx would strip a second time and eat
    // the body up to the next `---`.
    let nodes = content::parse_body(body);
    let raw_markdown = get_raw_markdown(&nodes);

    ParsedDoc {
        frontmatter,
        content: nodes,
        raw_markdown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_document_strips_duplicate_atx_h1() {
        let content = "---\ntitle: Hello\n---\n\n# Hello\n\nbody text\n";
        let doc = parse_document(content);
        assert_eq!(doc.frontmatter.title, "Hello");
        assert!(
            !doc.raw_markdown.contains("# Hello"),
            "expected leading H1 to be stripped, got: {:?}",
            doc.raw_markdown
        );
        assert!(doc.raw_markdown.contains("body text"));
    }

    #[test]
    fn parse_document_leaves_body_without_leading_h1_untouched() {
        let content = "---\ntitle: Hello\n---\n\nintro paragraph\n\n## Subheading\n";
        let doc = parse_document(content);
        assert!(doc.raw_markdown.contains("intro paragraph"));
        assert!(doc.raw_markdown.contains("## Subheading"));
    }

    #[test]
    fn parse_document_preserves_h1_appearing_mid_document() {
        let content = "---\ntitle: Hello\n---\n\nintro\n\n# Later heading\n\nmore body\n";
        let doc = parse_document(content);
        assert!(doc.raw_markdown.contains("intro"));
        assert!(
            doc.raw_markdown.contains("# Later heading"),
            "mid-document H1 should survive, got: {:?}",
            doc.raw_markdown
        );
    }

    #[test]
    fn parse_document_keeps_body_starting_with_thematic_break() {
        // parse_mdx would strip frontmatter a second time here and swallow
        // everything up to the next `---`.
        let content = "---\ntitle: T\n---\n\n---\n\nfirst para\n\n---\n\nsecond para\n";
        let doc = parse_document(content);
        assert_eq!(doc.frontmatter.title, "T");
        assert!(
            doc.raw_markdown.contains("first para"),
            "body before the second rule was eaten: {:?}",
            doc.raw_markdown
        );
        assert!(doc.raw_markdown.contains("second para"));
    }
}
