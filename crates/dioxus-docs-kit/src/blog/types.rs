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
/// Returns the parsed frontmatter and the remaining content after the frontmatter block.
pub fn extract_blog_frontmatter(content: &str) -> Option<(BlogFrontmatter, &str)> {
    let content = content.trim();

    if !content.starts_with("---") {
        return None;
    }

    let after_first_delim = &content[3..];
    let end_idx = after_first_delim.find("\n---")?;
    let yaml_content = after_first_delim[..end_idx].trim();
    let remaining = after_first_delim[end_idx + 4..].trim_start();

    let fm: BlogFrontmatter = serde_yaml::from_str(yaml_content).ok()?;
    Some((fm, remaining))
}

/// Calculate reading time from raw text (words / 200 WPM, minimum 1 minute).
pub fn calculate_reading_time(text: &str) -> u32 {
    let word_count = text.split_whitespace().count();
    ((word_count as f64 / 200.0).ceil() as u32).max(1)
}
