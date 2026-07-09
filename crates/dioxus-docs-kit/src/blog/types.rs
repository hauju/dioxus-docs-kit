use dioxus_mdx::DocNode;
use serde::Deserialize;
use std::collections::HashMap;

/// Blog manifest parsed from `_blog.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct BlogManifest {
    #[serde(default)]
    pub authors: HashMap<String, Author>,
    pub posts: Vec<String>,
}

/// Author definition from the blog manifest.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Author {
    pub name: String,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub bio: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

/// Blog post frontmatter extracted from MDX files.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BlogFrontmatter {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    /// ISO 8601 date string, e.g. "2026-03-15"
    pub date: String,
    /// Author ID referencing `_blog.json` authors map
    pub author: String,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Cover image path (relative to assets/)
    #[serde(default, rename = "coverImage")]
    pub cover_image: Option<String>,
    /// Set to true to hide from listing
    #[serde(default)]
    pub draft: bool,
    /// Set to true to pin this post to the featured section
    #[serde(default)]
    pub featured: bool,
}

/// A fully parsed blog post.
#[derive(Debug, Clone, PartialEq)]
pub struct BlogPost {
    /// URL slug (from filename)
    pub slug: String,
    pub frontmatter: BlogFrontmatter,
    /// Parsed MDX content nodes
    pub content: Vec<DocNode>,
    /// Raw markdown for search indexing and reading time calculation
    pub raw_markdown: String,
    /// Estimated reading time in minutes
    pub reading_time_minutes: u32,
}

/// A searchable entry in the blog.
#[derive(PartialEq)]
pub struct BlogSearchEntry {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub content_preview: String,
    pub date: String,
    pub tags: Vec<String>,
}

/// Extract blog frontmatter from MDX content.
///
/// Returns the parsed frontmatter and the remaining content after the frontmatter block,
/// or a description of why the frontmatter is invalid.
pub fn extract_blog_frontmatter(content: &str) -> Result<(BlogFrontmatter, &str), String> {
    let content = content.trim();

    if !content.starts_with("---") {
        return Err("missing frontmatter block (expected leading ---)".to_string());
    }

    let after_first_delim = &content[3..];
    let end_idx = after_first_delim
        .find("\n---")
        .ok_or_else(|| "unclosed frontmatter block (missing closing ---)".to_string())?;
    let yaml_content = after_first_delim[..end_idx].trim();
    let remaining = after_first_delim[end_idx + 4..].trim_start();

    let fm: BlogFrontmatter =
        serde_yaml::from_str(yaml_content).map_err(|e| format!("invalid frontmatter: {e}"))?;
    Ok((fm, remaining))
}

/// Calculate reading time from raw text (words / 200 WPM, minimum 1 minute).
pub fn calculate_reading_time(text: &str) -> u32 {
    let word_count = text.split_whitespace().count();
    ((word_count as f64 / 200.0).ceil() as u32).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_valid_frontmatter() {
        let content =
            "---\ntitle: Hello\ndate: \"2026-03-15\"\nauthor: jane\ntags: [rust]\n---\n\nBody text";
        let (fm, body) = extract_blog_frontmatter(content).unwrap();
        assert_eq!(fm.title, "Hello");
        assert_eq!(fm.date, "2026-03-15");
        assert_eq!(fm.author, "jane");
        assert_eq!(fm.tags, vec!["rust".to_string()]);
        assert!(!fm.draft);
        assert!(body.starts_with("Body text"));
    }

    #[test]
    fn missing_frontmatter_block_errors() {
        let err = extract_blog_frontmatter("Just body text").unwrap_err();
        assert!(err.contains("missing frontmatter"), "got: {err}");
    }

    #[test]
    fn unclosed_frontmatter_errors() {
        let err = extract_blog_frontmatter("---\ntitle: Hello\nno closing").unwrap_err();
        assert!(err.contains("unclosed"), "got: {err}");
    }

    #[test]
    fn missing_required_field_errors_with_detail() {
        // No `date` field.
        let err =
            extract_blog_frontmatter("---\ntitle: Hello\nauthor: jane\n---\nBody").unwrap_err();
        assert!(err.contains("invalid frontmatter"), "got: {err}");
        assert!(
            err.contains("date"),
            "expected serde detail naming the missing field, got: {err}"
        );
    }

    #[test]
    fn reading_time_rounds_up_with_minimum() {
        assert_eq!(calculate_reading_time("a few words"), 1);
        let long = "word ".repeat(400);
        assert_eq!(calculate_reading_time(&long), 2);
    }
}
