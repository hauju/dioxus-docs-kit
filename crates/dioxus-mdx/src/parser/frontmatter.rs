//! YAML frontmatter extraction from MDX files.

use crate::parser::types::DocFrontmatter;

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
        match serde_yaml::from_str(yaml_content) {
            Ok(fm) => (fm, remaining),
            Err(e) => {
                eprintln!("Failed to parse frontmatter: {}", e);
                (DocFrontmatter::default(), content)
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
