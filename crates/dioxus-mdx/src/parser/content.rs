//! MDX component extraction and parsing.

use regex::Regex;

use super::accordion::{try_parse_accordion_group, try_parse_standalone_accordion};
use super::callout::try_parse_callout;
use super::card::{try_parse_card_group, try_parse_columns, try_parse_standalone_card};
use super::code_group::{
    try_parse_code_group, try_parse_request_example, try_parse_response_example,
};
use super::fields::{try_parse_expandable, try_parse_param_field, try_parse_response_field};
use super::openapi_tag::try_parse_openapi;
use super::steps::try_parse_steps;
use super::tabs::try_parse_tabs;
use super::update::try_parse_update;
use crate::parser::frontmatter::extract_frontmatter;
use crate::parser::types::*;

/// Parse MDX content into a tree of DocNodes.
/// Automatically strips frontmatter and import statements.
pub fn parse_mdx(content: &str) -> Vec<DocNode> {
    // Strip frontmatter if present
    let (_, content) = extract_frontmatter(content);
    let content = strip_imports(content);
    let content = strip_helpful_widget(&content);
    parse_content(&content)
}

/// Strip import statements from MDX content.
fn strip_imports(content: &str) -> String {
    let import_re = Regex::new(r"(?m)^import\s+.*?;\s*\n?").unwrap();
    import_re.replace_all(content, "").to_string()
}

/// Strip the SeggWatIsPageHelpful component (we have our own).
fn strip_helpful_widget(content: &str) -> String {
    let re = Regex::new(r"<SeggWatIsPageHelpful\s*/?>").unwrap();
    re.replace_all(content, "").to_string()
}

/// Parse content into a sequence of DocNodes.
pub(super) fn parse_content(content: &str) -> Vec<DocNode> {
    let mut nodes = Vec::new();
    let mut remaining = content.trim();

    while !remaining.is_empty() {
        // Try to match each component type
        if let Some((node, rest)) = try_parse_callout(remaining) {
            nodes.push(node);
            remaining = rest.trim();
        } else if let Some((node, rest)) = try_parse_card_group(remaining) {
            nodes.push(node);
            remaining = rest.trim();
        } else if let Some((node, rest)) = try_parse_columns(remaining) {
            nodes.push(node);
            remaining = rest.trim();
        } else if let Some((node, rest)) = try_parse_standalone_card(remaining) {
            nodes.push(node);
            remaining = rest.trim();
        } else if let Some((node, rest)) = try_parse_tabs(remaining) {
            nodes.push(node);
            remaining = rest.trim();
        } else if let Some((node, rest)) = try_parse_steps(remaining) {
            nodes.push(node);
            remaining = rest.trim();
        } else if let Some((node, rest)) = try_parse_accordion_group(remaining) {
            nodes.push(node);
            remaining = rest.trim();
        } else if let Some((node, rest)) = try_parse_standalone_accordion(remaining) {
            nodes.push(node);
            remaining = rest.trim();
        } else if let Some((node, rest)) = try_parse_param_field(remaining) {
            nodes.push(node);
            remaining = rest.trim();
        } else if let Some((node, rest)) = try_parse_response_field(remaining) {
            nodes.push(node);
            remaining = rest.trim();
        } else if let Some((node, rest)) = try_parse_expandable(remaining) {
            nodes.push(node);
            remaining = rest.trim();
        } else if let Some((node, rest)) = try_parse_code_group(remaining) {
            nodes.push(node);
            remaining = rest.trim();
        } else if let Some((node, rest)) = try_parse_request_example(remaining) {
            nodes.push(node);
            remaining = rest.trim();
        } else if let Some((node, rest)) = try_parse_response_example(remaining) {
            nodes.push(node);
            remaining = rest.trim();
        } else if let Some((node, rest)) = try_parse_update(remaining) {
            nodes.push(node);
            remaining = rest.trim();
        } else if let Some((node, rest)) = try_parse_openapi(remaining) {
            nodes.push(node);
            remaining = rest.trim();
        } else {
            // Collect markdown until next component or end
            let next_component_idx = find_next_component(remaining);
            let (markdown, rest) = if let Some(idx) = next_component_idx {
                if idx == 0 {
                    // Component tag at position 0 but no parser matched it.
                    // Skip past the '<' to avoid an infinite loop, treating it as markdown.
                    let skip = remaining.find('>').map(|i| i + 1).unwrap_or(1);
                    (&remaining[..skip], &remaining[skip..])
                } else {
                    (&remaining[..idx], &remaining[idx..])
                }
            } else {
                (remaining, "")
            };

            if !markdown.trim().is_empty() {
                // Extract code blocks from markdown and interleave them
                let parsed_nodes = extract_code_blocks_from_markdown(markdown.trim());
                nodes.extend(parsed_nodes);
            }
            remaining = rest.trim();
        }
    }

    nodes
}

/// Extract fenced code blocks from markdown content.
/// Returns a list of nodes interleaving Markdown and CodeBlock.
fn extract_code_blocks_from_markdown(content: &str) -> Vec<DocNode> {
    let mut nodes = Vec::new();
    // Match fenced code blocks with optional language and filename
    // Handle both Unix (\n) and Windows (\r\n) line endings
    // The closing ``` must be on its own line (with optional leading whitespace)
    // IMPORTANT: Use [ \t]+ (not \s+) for filename separator to avoid matching across lines
    let code_re =
        Regex::new(r"(?m)^[ \t]*```(\w+)?(?:[ \t]+([^\r\n]+))?[ \t]*\r?\n([\s\S]*?)\r?\n[ \t]*```[ \t]*(?:\r?\n|$)")
            .unwrap();

    let mut last_end = 0;

    for caps in code_re.captures_iter(content) {
        let full_match = caps.get(0).expect("regex group 0");

        // Add any markdown before this code block
        if full_match.start() > last_end {
            let before = &content[last_end..full_match.start()];
            if !before.trim().is_empty() {
                nodes.push(DocNode::Markdown(before.trim().to_string()));
            }
        }

        // Add the code block
        let language = caps.get(1).map(|m| m.as_str().to_string());
        let filename = caps.get(2).map(|m| m.as_str().to_string());
        let code = caps
            .get(3)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();

        nodes.push(DocNode::CodeBlock(CodeBlockNode {
            language,
            filename,
            code,
        }));

        last_end = full_match.end();
    }

    // Add any remaining markdown after the last code block
    if last_end < content.len() {
        let after = &content[last_end..];
        if !after.trim().is_empty() {
            nodes.push(DocNode::Markdown(after.trim().to_string()));
        }
    }

    // If no code blocks were found, return the original content as markdown
    if nodes.is_empty() && !content.trim().is_empty() {
        nodes.push(DocNode::Markdown(content.trim().to_string()));
    }

    nodes
}

/// Find the index of the next MDX component in the content.
fn find_next_component(content: &str) -> Option<usize> {
    let patterns = [
        "<Tip>",
        "<Note>",
        "<Warning>",
        "<Info>",
        "<Card",
        "<CardGroup",
        "<Columns",
        "<Tabs>",
        "<Steps>",
        "<AccordionGroup>",
        "<Accordion",
        "<ParamField",
        "<ResponseField",
        "<Expandable",
        "<RequestExample>",
        "<ResponseExample>",
        "<CodeGroup>",
        "<Update",
        "<OpenAPI",
    ];

    patterns.iter().filter_map(|p| content.find(p)).min()
}

/// Get raw markdown from parsed content (for fallback rendering).
pub fn get_raw_markdown(nodes: &[DocNode]) -> String {
    let mut output = String::new();

    for node in nodes {
        match node {
            DocNode::Markdown(md) => {
                output.push_str(md);
                output.push_str("\n\n");
            }
            DocNode::Callout(c) => {
                output.push_str(&format!(
                    "> **{}:** {}\n\n",
                    c.callout_type.as_str(),
                    c.content
                ));
            }
            DocNode::Card(c) => {
                output.push_str(&format!("**{}**\n{}\n\n", c.title, c.content));
            }
            DocNode::CardGroup(cg) => {
                for card in &cg.cards {
                    output.push_str(&format!("**{}**\n{}\n\n", card.title, card.content));
                }
            }
            DocNode::Tabs(t) => {
                for tab in &t.tabs {
                    output.push_str(&format!("#### {}\n", tab.title));
                    output.push_str(&get_raw_markdown(&tab.content));
                }
            }
            DocNode::Steps(s) => {
                for (i, step) in s.steps.iter().enumerate() {
                    output.push_str(&format!("{}. **{}**\n", i + 1, step.title));
                    output.push_str(&get_raw_markdown(&step.content));
                }
            }
            DocNode::AccordionGroup(ag) => {
                for item in &ag.items {
                    output.push_str(&format!("### {}\n", item.title));
                    output.push_str(&get_raw_markdown(&item.content));
                }
            }
            DocNode::CodeBlock(cb) => {
                let lang = cb.language.as_deref().unwrap_or("");
                output.push_str(&format!("```{}\n{}\n```\n\n", lang, cb.code));
            }
            DocNode::CodeGroup(cg) => {
                for block in &cg.blocks {
                    let lang = block.language.as_deref().unwrap_or("");
                    output.push_str(&format!("```{}\n{}\n```\n\n", lang, block.code));
                }
            }
            DocNode::ParamField(f) => {
                let required = if f.required { " *(required)*" } else { "" };
                output.push_str(&format!(
                    "**`{}`** _{}_{}: ",
                    f.name, f.param_type, required
                ));
                output.push_str(&get_raw_markdown(&f.content));
            }
            DocNode::ResponseField(f) => {
                let required = if f.required { " *(required)*" } else { "" };
                output.push_str(&format!(
                    "**`{}`** _{}_{}: {}\n\n",
                    f.name, f.field_type, required, f.content
                ));
                if let Some(exp) = &f.expandable {
                    output.push_str(&format!("  **{}**\n", exp.title));
                    for field in &exp.fields {
                        output.push_str(&format!(
                            "  - `{}` _{}_: {}\n",
                            field.name, field.field_type, field.content
                        ));
                    }
                    output.push('\n');
                }
            }
            DocNode::Expandable(e) => {
                output.push_str(&format!("**{}**\n\n", e.title));
                for field in &e.fields {
                    output.push_str(&format!(
                        "- `{}` _{}_: {}\n",
                        field.name, field.field_type, field.content
                    ));
                }
                output.push('\n');
            }
            DocNode::RequestExample(ex) => {
                output.push_str("**Request:**\n\n");
                for block in &ex.blocks {
                    let lang = block.language.as_deref().unwrap_or("");
                    output.push_str(&format!("```{}\n{}\n```\n\n", lang, block.code));
                }
            }
            DocNode::ResponseExample(ex) => {
                output.push_str("**Response:**\n\n");
                for block in &ex.blocks {
                    let lang = block.language.as_deref().unwrap_or("");
                    output.push_str(&format!("```{}\n{}\n```\n\n", lang, block.code));
                }
            }
            DocNode::Update(u) => {
                output.push_str(&format!("### {}\n\n", u.label));
                output.push_str(&get_raw_markdown(&u.content));
            }
            DocNode::OpenApi(api) => {
                output.push_str(&format!("# {}\n\n", api.spec.info.title));
                if let Some(desc) = &api.spec.info.description {
                    output.push_str(desc);
                    output.push_str("\n\n");
                }
                for op in &api.spec.operations {
                    output.push_str(&format!("## {} {}\n\n", op.method.as_str(), op.path));
                    if let Some(summary) = &op.summary {
                        output.push_str(summary);
                        output.push_str("\n\n");
                    }
                }
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_imports() {
        let content = r#"import { Foo } from '/bar';
import Something from 'pkg';

## Content"#;
        let result = strip_imports(content);
        assert!(result.contains("## Content"));
        assert!(!result.contains("import"));
    }

    #[test]
    fn test_mixed_content() {
        let content = r#"## Introduction

Some markdown content.

<Tip>This is a tip!</Tip>

More content after."#;

        let nodes = parse_mdx(content);
        assert_eq!(nodes.len(), 3);
        assert!(matches!(&nodes[0], DocNode::Markdown(_)));
        assert!(matches!(&nodes[1], DocNode::Callout(_)));
        assert!(matches!(&nodes[2], DocNode::Markdown(_)));
    }

    #[test]
    fn test_code_block_extraction_from_markdown() {
        let content = r#"Some intro text.

```html
<script src="test.js"></script>
```

More text after."#;
        let nodes = parse_mdx(content);
        // Should have 3 nodes: Markdown, CodeBlock, Markdown
        assert_eq!(nodes.len(), 3);
        assert!(matches!(&nodes[0], DocNode::Markdown(m) if m.contains("intro text")));
        assert!(
            matches!(&nodes[1], DocNode::CodeBlock(cb) if cb.language == Some("html".to_string()))
        );
        assert!(matches!(&nodes[2], DocNode::Markdown(m) if m.contains("More text")));
    }

    #[test]
    fn test_multiple_code_blocks_in_markdown() {
        let content = r#"First section.

```js
const x = 1;
```

Middle text.

```rust
fn main() {}
```

End section."#;
        let nodes = parse_mdx(content);
        // Should have 5 nodes: Markdown, CodeBlock, Markdown, CodeBlock, Markdown
        assert_eq!(nodes.len(), 5);
        assert!(matches!(&nodes[0], DocNode::Markdown(_)));
        assert!(
            matches!(&nodes[1], DocNode::CodeBlock(cb) if cb.language == Some("js".to_string()))
        );
        assert!(matches!(&nodes[2], DocNode::Markdown(_)));
        assert!(
            matches!(&nodes[3], DocNode::CodeBlock(cb) if cb.language == Some("rust".to_string()))
        );
        assert!(matches!(&nodes[4], DocNode::Markdown(_)));
    }

    #[test]
    fn test_multiline_html_code_block() {
        // Test the specific pattern from customization.mdx that was failing
        let content = r##"Example with React colors:
```html
<script defer
  src="https://seggwat.com/static/widgets/v1/seggwat-feedback.js"
  data-project-key="YOUR_PROJECT_KEY"
  data-button-color="#61dafb">
</script>
```

### Next Section"##;
        let nodes = parse_mdx(content);
        // Should have 3 nodes: Markdown, CodeBlock, Markdown
        assert_eq!(nodes.len(), 3);
        assert!(matches!(&nodes[0], DocNode::Markdown(m) if m.contains("React colors")));
        if let DocNode::CodeBlock(cb) = &nodes[1] {
            assert_eq!(cb.language, Some("html".to_string()));
            // Verify the full script tag is captured
            assert!(cb.code.contains("<script defer"));
            assert!(cb.code.contains("data-button-color"));
            assert!(cb.code.contains("</script>"));
        } else {
            panic!("Expected CodeBlock node, got {:?}", &nodes[1]);
        }
        assert!(matches!(&nodes[2], DocNode::Markdown(m) if m.contains("Next Section")));
    }
}
