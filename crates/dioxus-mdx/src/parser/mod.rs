//! MDX documentation parser for extracting frontmatter and components.
//!
//! This module provides functionality to parse MDX (Markdown with JSX) content,
//! extracting YAML frontmatter and converting custom components like Cards, Tabs,
//! Steps, and Callouts into an intermediate representation for rendering.

// ── Value types (always compiled; no dependencies beyond serde) ───────────
mod openapi_types;
mod types;
pub mod yaml_lite;

// ── Parsing (the `parse` feature; build-time only in a docs-kit app) ──────
#[cfg(feature = "parse")]
mod accordion;
#[cfg(feature = "parse")]
mod callout;
#[cfg(feature = "parse")]
mod card;
#[cfg(feature = "parse")]
mod code_group;
#[cfg(feature = "parse")]
mod content;
#[cfg(feature = "parse")]
mod fields;
#[cfg(feature = "parse")]
mod frontmatter;
#[cfg(feature = "parse")]
mod heading;
#[cfg(feature = "openapi-parse")]
mod openapi_parser;
#[cfg(feature = "openapi-parse")]
mod openapi_tag;
#[cfg(feature = "parse")]
mod render;
#[cfg(feature = "parse")]
mod steps;
#[cfg(feature = "parse")]
mod tabs;
#[cfg(feature = "parse")]
mod update;
#[cfg(feature = "parse")]
mod utils;

pub use openapi_types::*;
pub use types::*;
pub use yaml_lite::{YamlLiteError, YamlMap, YamlValue, parse_yaml_lite};

#[cfg(feature = "parse")]
pub use content::parse_mdx;
#[cfg(feature = "parse")]
pub use frontmatter::extract_frontmatter;
#[cfg(feature = "parse")]
pub use heading::strip_leading_h1;
#[cfg(feature = "openapi-parse")]
pub use openapi_parser::{OpenApiError, parse_openapi};
#[cfg(feature = "parse")]
pub use render::{to_html, to_html_with_heading_ids};

/// Parse a complete MDX document, extracting frontmatter and content.
///
/// This is the main entry point for parsing MDX content. It extracts
/// YAML frontmatter from the beginning of the document and parses the
/// remaining content into a tree of `DocNode` elements whose prose is already
/// rendered to HTML.
#[cfg(feature = "parse")]
pub fn parse_document(content: &str) -> ParsedDoc {
    let (frontmatter, remaining) = extract_frontmatter(content);
    // Consumer layouts render the frontmatter title in their own <h1>; drop a
    // duplicate body H1 so the page emits exactly one.
    let body = strip_leading_h1(remaining);
    // Frontmatter is already gone; parse_mdx would strip a second time and eat
    // the body up to the next `---`.
    let (nodes, raw_markdown) = parse_body(body);

    ParsedDoc {
        frontmatter,
        content: nodes,
        raw_markdown,
    }
}

/// Parse an MDX body whose frontmatter has already been removed.
///
/// Returns the rendered node tree plus the reconstructed Markdown source (the
/// input minus imports and MDX component syntax), which feeds `llms.txt`, the
/// search index and the table of contents.
///
/// Use this instead of [`parse_mdx`] when you extracted the frontmatter
/// yourself, or a body starting with a thematic break gets mistaken for a
/// second frontmatter block and everything up to the next `---` is discarded.
#[cfg(feature = "parse")]
pub fn parse_body(body: &str) -> (Vec<DocNode>, String) {
    let mut nodes = content::parse_body_nodes(body);
    // `get_raw_markdown` reads the Markdown still sitting in the prose fields,
    // so it has to run before they are rendered to HTML.
    let raw_markdown = content::get_raw_markdown(&nodes);
    render::render_nodes(&mut nodes);
    (nodes, raw_markdown)
}

#[cfg(all(test, feature = "parse"))]
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
