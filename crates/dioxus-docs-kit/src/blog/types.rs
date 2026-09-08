use dioxus_mdx::{DocNode, YamlLiteError, parse_yaml_lite};
use serde::Deserialize;
use std::collections::HashMap;

/// Blog manifest parsed from `_blog.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct BlogManifest {
    #[serde(default)]
    pub authors: HashMap<String, Author>,
    /// Optional category metadata keyed by the exact post tag.
    #[serde(default)]
    pub categories: HashMap<String, BlogCategoryMetadata>,
    pub posts: Vec<String>,
}

/// Optional presentation and URL overrides for a tag's category page.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct BlogCategoryMetadata {
    pub slug: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
}

/// A topic with at least one published post. Membership comes from post tags.
#[derive(Debug, Clone, PartialEq)]
pub struct BlogCategory {
    pub tag: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub image: Option<String>,
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
#[derive(Debug, Clone, PartialEq)]
pub struct BlogFrontmatter {
    pub title: String,
    pub description: Option<String>,
    /// ISO 8601 date string, e.g. "2026-03-15"
    pub date: String,
    /// Author ID referencing `_blog.json` authors map
    pub author: String,
    pub tags: Vec<String>,
    /// Cover image path (relative to assets/), from the `coverImage` key
    pub cover_image: Option<String>,
    /// Set to true to hide from listing
    pub draft: bool,
    /// Set to true to pin this post to the featured section
    pub featured: bool,
}

impl BlogFrontmatter {
    fn from_yaml(yaml: &str) -> Result<Self, YamlLiteError> {
        let map = parse_yaml_lite(yaml)?;
        Ok(Self {
            title: map.require_str("title")?,
            description: map.optional_str("description")?,
            date: map.require_str("date")?,
            author: map.require_str("author")?,
            tags: map.optional_str_seq("tags")?,
            cover_image: map.optional_str("coverImage")?,
            draft: map.optional_bool("draft")?,
            featured: map.optional_bool("featured")?,
        })
    }
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

/// A searchable entry in the blog (one per post — blog search has no sections).
///
/// The `*_lower` fields are lowercased once at build time so search never
/// re-lowercases per keystroke.
#[derive(PartialEq)]
pub struct BlogSearchEntry {
    pub slug: String,
    pub title: String,
    pub description: String,
    /// Cleaned post body used for matching and snippet extraction.
    pub body: String,
    pub date: String,
    pub tags: Vec<String>,
    pub(crate) title_lower: String,
    pub(crate) description_lower: String,
    pub(crate) body_lower: String,
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

    let fm = BlogFrontmatter::from_yaml(yaml_content)
        .map_err(|e| format!("invalid frontmatter: {e}"))?;
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
    fn extracts_block_sequence_tags_booleans_and_comments() {
        let content = "---\n# post metadata\ntitle: 'It''s fine'\ndate: \"2026-03-15\"\nauthor: jane\ntags:\n  - rust\n  - \"dioxus\"\ndraft: true\nfeatured: false\ncoverImage: /img/cover.png # relative to assets/\n---\n\nBody";
        let (fm, body) = extract_blog_frontmatter(content).unwrap();
        assert_eq!(fm.title, "It's fine");
        assert_eq!(fm.tags, vec!["rust".to_string(), "dioxus".to_string()]);
        assert!(fm.draft);
        assert!(!fm.featured);
        assert_eq!(fm.cover_image, Some("/img/cover.png".to_string()));
        assert!(body.starts_with("Body"));
    }

    #[test]
    fn unsupported_yaml_shape_errors() {
        // A nested mapping is outside the supported frontmatter subset.
        let err = extract_blog_frontmatter(
            "---\ntitle: Hi\ndate: \"2026-01-01\"\nauthor:\n  name: jane\n---\nBody",
        )
        .unwrap_err();
        assert!(err.contains("invalid frontmatter"), "got: {err}");
    }

    #[test]
    fn wrongly_typed_field_errors() {
        // `tags` must be a sequence, not a scalar.
        let err = extract_blog_frontmatter(
            "---\ntitle: Hi\ndate: \"2026-01-01\"\nauthor: jane\ntags: rust\n---\nBody",
        )
        .unwrap_err();
        assert!(err.contains("tags"), "got: {err}");
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
