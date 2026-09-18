//! Build-time Markdown → HTML rendering.
//!
//! The parser first builds the node tree with the prose fields still holding
//! Markdown (that is what [`get_raw_markdown`](super::get_raw_markdown)
//! reconstructs the page source from), then [`render_nodes`] turns every prose
//! field into HTML in place. Because this runs in `dioxus-docs-kit-build`, the
//! `markdown` crate never reaches the wasm bundle.

use std::sync::LazyLock;

use crate::re::{Captures, Regex};
use crate::text::slugify;

use super::types::{DocNode, ResponseFieldNode};

static HEADING_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<(h[2-4])>(.*?)</h[2-4]>").unwrap());
static HTML_TAG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").unwrap());

/// Render GitHub-flavoured Markdown to HTML.
///
/// Falls back to the input unchanged if markdown-rs errors (it does not for
/// GFM, but the API is fallible).
pub fn to_html(md: &str) -> String {
    markdown::to_html_with_options(md, &markdown::Options::gfm()).unwrap_or_else(|_| md.to_string())
}

/// Render Markdown to HTML and inject `id` attributes into h2–h4 headings so
/// table-of-contents anchor links resolve.
pub fn to_html_with_heading_ids(md: &str) -> String {
    inject_heading_ids(&to_html(md))
}

/// Inject `id` attributes into heading tags so TOC anchor links work.
fn inject_heading_ids(html: &str) -> String {
    HEADING_RE
        .replace_all(html, |caps: &Captures| {
            let tag = &caps[1];
            let inner = &caps[2];
            // Strip any inner HTML tags to get plain text for the slug
            let plain = HTML_TAG_RE.replace_all(inner, "");
            let id = slugify(&plain);
            format!("<{tag} id=\"{id}\">{inner}</{tag}>")
        })
        .into_owned()
}

/// Replace every Markdown-carrying field in the tree with its rendered HTML.
///
/// Call this *after* `get_raw_markdown`, which needs the Markdown source.
pub(super) fn render_nodes(nodes: &mut [DocNode]) {
    for node in nodes {
        match node {
            DocNode::Html(text) => *text = to_html_with_heading_ids(text),
            DocNode::Callout(callout) => callout.content_html = to_html(&callout.content_html),
            DocNode::Card(card) => card.content_html = to_html(&card.content_html),
            DocNode::CardGroup(group) => {
                for card in &mut group.cards {
                    card.content_html = to_html(&card.content_html);
                }
            }
            DocNode::Tabs(tabs) => {
                for tab in &mut tabs.tabs {
                    render_nodes(&mut tab.content);
                }
            }
            DocNode::Steps(steps) => {
                for step in &mut steps.steps {
                    render_nodes(&mut step.content);
                }
            }
            DocNode::AccordionGroup(group) => {
                for item in &mut group.items {
                    render_nodes(&mut item.content);
                }
            }
            DocNode::ParamField(field) => render_nodes(&mut field.content),
            DocNode::ResponseField(field) => render_response_field(field),
            DocNode::Expandable(expandable) => {
                for field in &mut expandable.fields {
                    render_response_field(field);
                }
            }
            DocNode::Update(update) => render_nodes(&mut update.content),
            DocNode::CodeBlock(_)
            | DocNode::CodeGroup(_)
            | DocNode::RequestExample(_)
            | DocNode::ResponseExample(_)
            | DocNode::OpenApi(_) => {}
        }
    }
}

fn render_response_field(field: &mut ResponseFieldNode) {
    field.content_html = to_html(&field.content_html);
    if let Some(expandable) = &mut field.expandable {
        for nested in &mut expandable.fields {
            render_response_field(nested);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headings_get_slug_ids() {
        let html = to_html_with_heading_ids("## Getting Started!\n\ntext\n");
        assert!(
            html.contains("<h2 id=\"getting-started\">Getting Started!</h2>"),
            "got: {html}"
        );
    }

    #[test]
    fn heading_id_ignores_inner_markup() {
        let html = to_html_with_heading_ids("## Use `cargo build`\n");
        assert!(html.contains("id=\"use-cargo-build\""), "got: {html}");
        assert!(html.contains("<code>cargo build</code>"), "got: {html}");
    }

    #[test]
    fn gfm_tables_and_autolinks_render() {
        let html = to_html("| a | b |\n| - | - |\n| 1 | 2 |\n");
        assert!(html.contains("<table>"), "got: {html}");
    }
}
