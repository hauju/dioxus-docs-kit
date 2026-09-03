//! YAML frontmatter extraction from MDX files.

use crate::parser::types::DocFrontmatter;
use crate::parser::yaml_lite::{YamlLiteError, parse_yaml_lite};

/// Build a [`DocFrontmatter`] from a frontmatter block.
///
/// Every field is optional, so only an unsupported YAML shape (see
/// [`parse_yaml_lite`]) or a wrongly typed field is an error.
fn parse_doc_frontmatter(yaml: &str) -> Result<DocFrontmatter, YamlLiteError> {
    let map = parse_yaml_lite(yaml)?;
    Ok(DocFrontmatter {
        title: map.optional_str("title")?.unwrap_or_default(),
        description: map.optional_str("description")?,
        sidebar_title: map.optional_str("sidebarTitle")?,
        icon: map.optional_str("icon")?,
    })
}

/// Extract YAML frontmatter from MDX content.
///
/// Returns the parsed frontmatter and the remaining content after the frontmatter block.
pub fn extract_frontmatter(content: &str) -> (DocFrontmatter, &str) {
    let content = content.trim();

    // Check if content starts with frontmatter delimiter
    if !content.starts_with("---") {
        return (DocFrontmatter::default(), content);
    }

    // Find the closing delimiter
    let after_first_delim = &content[3..];
    if let Some(end_idx) = after_first_delim.find("\n---") {
        let yaml_content = &after_first_delim[..end_idx].trim();
        let remaining = &after_first_delim[end_idx + 4..].trim_start();

        // Parse YAML frontmatter
        match parse_doc_frontmatter(yaml_content) {
            Ok(fm) => (fm, remaining),
            Err(e) => {
                tracing::warn!("Failed to parse frontmatter: {e}");
                // Still strip the frontmatter block so the raw YAML doesn't
                // render as page text.
                (DocFrontmatter::default(), remaining)
            }
        }
    } else {
        // No closing delimiter found
        (DocFrontmatter::default(), content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_frontmatter() {
        let content = r#"---
title: Test Page
description: A test description
icon: "brain-circuit"
---

## Content here
"#;

        let (fm, remaining) = extract_frontmatter(content);
        assert_eq!(fm.title, "Test Page");
        assert_eq!(fm.description, Some("A test description".to_string()));
        assert_eq!(fm.icon, Some("brain-circuit".to_string()));
        assert!(remaining.starts_with("## Content here"));
    }

    #[test]
    fn test_no_frontmatter() {
        let content = "## Just content";
        let (fm, remaining) = extract_frontmatter(content);
        assert_eq!(fm.title, "");
        assert_eq!(remaining, "## Just content");
    }

    #[test]
    fn test_frontmatter_with_sidebar_title() {
        let content = r#"---
title: "Long Page Title"
sidebarTitle: "Short"
---

Content"#;

        let (fm, _) = extract_frontmatter(content);
        assert_eq!(fm.title, "Long Page Title");
        assert_eq!(fm.sidebar_title, Some("Short".to_string()));
    }

    #[test]
    fn test_empty_frontmatter() {
        let content = r#"---
---

Content"#;

        let (fm, remaining) = extract_frontmatter(content);
        assert_eq!(fm.title, "");
        assert!(remaining.starts_with("Content"));
    }

    #[test]
    fn test_malformed_frontmatter_is_stripped() {
        let content = r#"---
title: [unclosed
---

Content"#;

        let (fm, remaining) = extract_frontmatter(content);
        assert_eq!(fm.title, "");
        assert!(remaining.starts_with("Content"));
    }

    #[test]
    fn test_quoted_values_with_escapes_and_comments() {
        let content = "---\n# page metadata\ntitle: \"A \\\"quoted\\\" title\"\ndescription: 'it''s fine'\nicon: book-open # lucide name\n---\n\nContent";

        let (fm, remaining) = extract_frontmatter(content);
        assert_eq!(fm.title, "A \"quoted\" title");
        assert_eq!(fm.description, Some("it's fine".to_string()));
        assert_eq!(fm.icon, Some("book-open".to_string()));
        assert!(remaining.starts_with("Content"));
    }

    #[test]
    fn test_unsupported_shape_falls_back_to_default_and_strips_block() {
        // A nested mapping is outside the supported subset.
        let content = "---\ntitle: Test\nauthor:\n  name: Jane\n---\n\nContent";

        let (fm, remaining) = extract_frontmatter(content);
        assert_eq!(fm.title, "");
        assert!(remaining.starts_with("Content"));
    }

    #[test]
    fn test_unclosed_frontmatter() {
        let content = r#"---
title: Test
No closing delimiter
"#;

        let (fm, remaining) = extract_frontmatter(content);
        assert_eq!(fm.title, "");
        assert!(remaining.starts_with("---"));
    }
}
