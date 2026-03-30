//! Blog content registry.

use crate::blog::config::BlogConfig;
use crate::blog::types::{
    Author, BlogManifest, BlogPost, BlogSearchEntry, calculate_reading_time,
    extract_blog_frontmatter,
};
use crate::config::ThemeConfig;
use dioxus_mdx::{get_raw_markdown, parse_mdx};
use std::collections::HashMap;

/// Central blog registry holding all parsed content.
///
/// Created via [`BlogConfig`] builder and typically stored in a `LazyLock<BlogRegistry>` static.
pub struct BlogRegistry {
    /// All parsed blog posts, sorted by date (newest first).
    posts: Vec<BlogPost>,
    /// Author definitions from `_blog.json`.
    authors: HashMap<String, Author>,
    /// All unique tags across all posts, sorted alphabetically.
    all_tags: Vec<String>,
    /// Prebuilt search index.
    search_index: Vec<BlogSearchEntry>,
    /// Posts per page for pagination.
    pub posts_per_page: usize,
    /// Date display format string.
    pub date_format: String,
    /// Optional theme configuration.
    pub theme: Option<ThemeConfig>,
}

impl BlogRegistry {
    pub(crate) fn from_config(config: BlogConfig) -> Self {
        let manifest: BlogManifest =
            serde_json::from_str(config.manifest_json()).expect("Failed to parse _blog.json");

        let mut posts: Vec<BlogPost> = config
            .content_map()
            .iter()
            .filter(|(key, _)| **key != "__manifest__")
            .filter_map(|(&slug, &content)| {
                let (frontmatter, remaining) = extract_blog_frontmatter(content)?;

                if frontmatter.draft {
                    return None;
                }

                let nodes = parse_mdx(remaining);
                let raw_markdown = get_raw_markdown(&nodes);
                let reading_time_minutes = calculate_reading_time(&raw_markdown);

                Some(BlogPost {
                    slug: slug.to_string(),
                    frontmatter,
                    content: nodes,
                    raw_markdown,
                    reading_time_minutes,
                })
            })
            .collect();

        posts.sort_by(|a, b| b.frontmatter.date.cmp(&a.frontmatter.date));

        let mut tag_set: Vec<String> = posts
            .iter()
            .flat_map(|p| p.frontmatter.tags.iter().cloned())
            .collect();
        tag_set.sort();
        tag_set.dedup();

        let search_index = Self::build_search_index(&posts);

        let posts_per_page = config.posts_per_page();
        let date_format = config.date_format().to_string();
        let theme = config.theme_config().cloned();

        Self {
            posts,
            authors: manifest.authors,
            all_tags: tag_set,
            search_index,
            posts_per_page,
            date_format,
            theme,
        }
    }

    // ── Post access ──────────────────────────────────────────────────────

    pub fn get_post(&self, slug: &str) -> Option<&BlogPost> {
        self.posts.iter().find(|p| p.slug == slug)
    }

    pub fn all_posts(&self) -> &[BlogPost] {
        &self.posts
    }

    pub fn posts_by_tag(&self, tag: &str) -> Vec<&BlogPost> {
        self.posts
            .iter()
            .filter(|p| p.frontmatter.tags.iter().any(|t| t == tag))
            .collect()
    }

    pub fn posts_page(&self, page: usize) -> &[BlogPost] {
        let start = page * self.posts_per_page;
        let end = (start + self.posts_per_page).min(self.posts.len());
        if start >= self.posts.len() {
            return &[];
        }
        &self.posts[start..end]
    }

    pub fn posts_page_by_tag(&self, tag: &str, page: usize) -> Vec<&BlogPost> {
        let filtered = self.posts_by_tag(tag);
        let start = page * self.posts_per_page;
        let end = (start + self.posts_per_page).min(filtered.len());
        if start >= filtered.len() {
            return Vec::new();
        }
        filtered[start..end].to_vec()
    }

    pub fn total_pages(&self) -> usize {
        if self.posts.is_empty() {
            return 1;
        }
        self.posts.len().div_ceil(self.posts_per_page)
    }

    pub fn total_pages_for_tag(&self, tag: &str) -> usize {
        let count = self.posts_by_tag(tag).len();
        if count == 0 {
            return 1;
        }
        count.div_ceil(self.posts_per_page)
    }

    // ── Navigation ───────────────────────────────────────────────────────

    /// Get the previous post (older) relative to the given slug.
    pub fn prev_post(&self, slug: &str) -> Option<&BlogPost> {
        let idx = self.posts.iter().position(|p| p.slug == slug)?;
        if idx + 1 < self.posts.len() {
            Some(&self.posts[idx + 1])
        } else {
            None
        }
    }

    /// Get the next post (newer) relative to the given slug.
    pub fn next_post(&self, slug: &str) -> Option<&BlogPost> {
        let idx = self.posts.iter().position(|p| p.slug == slug)?;
        if idx > 0 {
            Some(&self.posts[idx - 1])
        } else {
            None
        }
    }

    // ── Metadata ─────────────────────────────────────────────────────────

    pub fn all_tags(&self) -> &[String] {
        &self.all_tags
    }

    pub fn tag_count(&self, tag: &str) -> usize {
        self.posts
            .iter()
            .filter(|p| p.frontmatter.tags.iter().any(|t| t == tag))
            .count()
    }

    pub fn get_author(&self, id: &str) -> Option<&Author> {
        self.authors.get(id)
    }

    // ── Search ───────────────────────────────────────────────────────────

    pub fn search_posts(&self, query: &str) -> Vec<&BlogSearchEntry> {
        let query = query.trim();
        if query.is_empty() {
            return Vec::new();
        }
        let q = query.to_lowercase();

        let mut title_matches: Vec<&BlogSearchEntry> = Vec::new();
        let mut desc_matches: Vec<&BlogSearchEntry> = Vec::new();
        let mut content_matches: Vec<&BlogSearchEntry> = Vec::new();

        for entry in &self.search_index {
            if entry.title.to_lowercase().contains(&q) {
                title_matches.push(entry);
            } else if entry.description.to_lowercase().contains(&q) {
                desc_matches.push(entry);
            } else if entry.content_preview.to_lowercase().contains(&q) {
                content_matches.push(entry);
            }
        }

        title_matches.extend(desc_matches);
        title_matches.extend(content_matches);
        title_matches
    }

    fn build_search_index(posts: &[BlogPost]) -> Vec<BlogSearchEntry> {
        posts
            .iter()
            .map(|post| {
                let preview: String = post.raw_markdown.chars().take(200).collect();
                BlogSearchEntry {
                    slug: post.slug.clone(),
                    title: post.frontmatter.title.clone(),
                    description: post.frontmatter.description.clone().unwrap_or_default(),
                    content_preview: preview,
                    date: post.frontmatter.date.clone(),
                    tags: post.frontmatter.tags.clone(),
                }
            })
            .collect()
    }

    // ── RSS ──────────────────────────────────────────────────────────────

    pub fn generate_rss(&self, site_title: &str, site_url: &str, blog_path: &str) -> String {
        let mut rss = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
<channel>
<title>{site_title}</title>
<link>{site_url}{blog_path}</link>
<description>{site_title} RSS Feed</description>
<atom:link href="{site_url}{blog_path}/rss.xml" rel="self" type="application/rss+xml"/>
"#
        );

        for post in &self.posts {
            let title = &post.frontmatter.title;
            let desc = post
                .frontmatter
                .description
                .as_deref()
                .unwrap_or_default();
            let link = format!("{site_url}{blog_path}/{}", post.slug);
            rss.push_str(&format!(
                "<item>\n<title>{title}</title>\n<link>{link}</link>\n<description>{desc}</description>\n<pubDate>{}</pubDate>\n<guid>{link}</guid>\n</item>\n",
                post.frontmatter.date
            ));
        }

        rss.push_str("</channel>\n</rss>\n");
        rss
    }

    pub fn generate_llms_txt(
        &self,
        site_title: &str,
        site_description: &str,
        base_url: &str,
        blog_path: &str,
    ) -> String {
        let mut out = format!("# {site_title}\n\n> {site_description}\n\n");

        for post in &self.posts {
            let title = &post.frontmatter.title;
            let desc = post
                .frontmatter
                .description
                .as_deref()
                .unwrap_or_default();
            let url = format!("{base_url}{blog_path}/{}", post.slug);
            if desc.is_empty() {
                out.push_str(&format!("- [{title}]({url})\n"));
            } else {
                out.push_str(&format!("- [{title}]({url}): {desc}\n"));
            }
        }

        out
    }

    // ── Date formatting ──────────────────────────────────────────────────

    pub fn format_date(&self, date: &str) -> String {
        format_date_with(date, &self.date_format)
    }
}

/// Format an ISO 8601 date string (YYYY-MM-DD) with a simple format pattern.
pub fn format_date_with(date: &str, fmt: &str) -> String {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return date.to_string();
    }

    let year = parts[0];
    let month = parts[1];
    let day = parts[2];

    let month_name = match month {
        "01" => "January",
        "02" => "February",
        "03" => "March",
        "04" => "April",
        "05" => "May",
        "06" => "June",
        "07" => "July",
        "08" => "August",
        "09" => "September",
        "10" => "October",
        "11" => "November",
        "12" => "December",
        _ => month,
    };

    fmt.replace("%Y", year)
        .replace("%m", month)
        .replace("%d", day)
        .replace("%B", month_name)
}
