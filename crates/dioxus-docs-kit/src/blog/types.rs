use dioxus_mdx::DocNode;
use serde::Deserialize;

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

/// Blog post frontmatter, extracted from the post's MDX at build time.
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
    /// Cover image path (relative to assets/), from the `coverImage` key
    #[serde(default)]
    pub cover_image: Option<String>,
    /// Set to true to hide from listing. Drafts are dropped from the bundle at
    /// build time, so this is always `false` on a post the registry holds.
    #[serde(default)]
    pub draft: bool,
    /// Set to true to pin this post to the featured section
    #[serde(default)]
    pub featured: bool,
}

/// A fully parsed blog post.
#[derive(Debug, Clone, PartialEq, Deserialize)]
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
/// Built by `dioxus-docs-kit-build`, `*_lower` fields included, so search never
/// re-lowercases per keystroke.
#[derive(PartialEq, Deserialize)]
pub struct BlogSearchEntry {
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    /// Cleaned post body used for matching and snippet extraction.
    #[serde(default)]
    pub body: String,
    pub date: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub(crate) title_lower: String,
    #[serde(default)]
    pub(crate) description_lower: String,
    #[serde(default)]
    pub(crate) body_lower: String,
}
