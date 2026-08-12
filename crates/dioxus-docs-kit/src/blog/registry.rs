//! Blog content registry.

use crate::blog::config::BlogConfig;
use crate::blog::types::{
    Author, BlogManifest, BlogPost, BlogSearchEntry, calculate_reading_time,
    extract_blog_frontmatter,
};
use crate::components::seo::xml_escape;
use crate::config::ThemeConfig;
use crate::error::DocsKitError;
use dioxus_mdx::{get_raw_markdown, parse_mdx, strip_leading_h1};
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
    /// Indices into `posts` for featured posts, preserving date order.
    featured_indices: Vec<usize>,
    /// Posts per page for pagination.
    pub posts_per_page: usize,
    /// Date display format string.
    pub date_format: String,
    /// Optional theme configuration.
    pub theme: Option<ThemeConfig>,
}

impl BlogRegistry {
    pub(crate) fn try_from_config(config: BlogConfig) -> Result<Self, DocsKitError> {
        let manifest: BlogManifest = serde_json::from_str(config.manifest_json())
            .map_err(DocsKitError::BlogManifestParse)?;

        let mut posts: Vec<BlogPost> = config
            .content_map()
            .iter()
            .filter(|(key, _)| **key != "__manifest__")
            .filter_map(|(&slug, &content)| {
                let (frontmatter, remaining) = match extract_blog_frontmatter(content) {
                    Ok(parsed) => parsed,
                    Err(e) => {
                        tracing::warn!("dioxus-docs-kit: skipping blog post \"{slug}\": {e}");
                        return None;
                    }
                };

                if frontmatter.draft {
                    return None;
                }

                // Blog post views render the frontmatter title in their own
                // <h1>; strip a duplicate body H1 so each page emits exactly one.
                let body = strip_leading_h1(remaining);
                let nodes = parse_mdx(body);
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

        let featured_indices: Vec<usize> = posts
            .iter()
            .enumerate()
            .filter(|(_, p)| p.frontmatter.featured)
            .map(|(i, _)| i)
            .collect();

        let search_index = Self::build_search_index(&posts);

        let posts_per_page = config.posts_per_page();
        let date_format = config.date_format().to_string();
        let theme = config.theme_config().cloned();

        Ok(Self {
            posts,
            authors: manifest.authors,
            all_tags: tag_set,
            featured_indices,
            search_index,
            posts_per_page,
            date_format,
            theme,
        })
    }

    // ── Post access ──────────────────────────────────────────────────────

    pub fn get_post(&self, slug: &str) -> Option<&BlogPost> {
        self.posts.iter().find(|p| p.slug == slug)
    }

    pub fn all_posts(&self) -> &[BlogPost] {
        &self.posts
    }

    /// Get all featured/pinned posts, sorted by date (newest first).
    pub fn featured_posts(&self) -> Vec<&BlogPost> {
        self.featured_indices
            .iter()
            .map(|&i| &self.posts[i])
            .collect()
    }

    /// Check if there are any featured posts.
    pub fn has_featured(&self) -> bool {
        !self.featured_indices.is_empty()
    }

    pub fn posts_by_tag(&self, tag: &str) -> Vec<&BlogPost> {
        self.posts
            .iter()
            .filter(|p| p.frontmatter.tags.iter().any(|t| t == tag))
            .collect()
    }

    /// Get a page of non-featured posts for the main blog index.
    pub fn non_featured_posts_page(&self, page: usize) -> Vec<&BlogPost> {
        let filtered: Vec<&BlogPost> = self
            .posts
            .iter()
            .filter(|p| !p.frontmatter.featured)
            .collect();
        let start = page * self.posts_per_page;
        let end = (start + self.posts_per_page).min(filtered.len());
        if start >= filtered.len() {
            return Vec::new();
        }
        filtered[start..end].to_vec()
    }

    /// Total number of pages for the main blog index, excluding featured posts.
    pub fn non_featured_total_pages(&self) -> usize {
        let count = self
            .posts
            .iter()
            .filter(|p| !p.frontmatter.featured)
            .count();
        if count == 0 {
            return 1;
        }
        count.div_ceil(self.posts_per_page)
    }

    /// Find posts related to the given slug by tag overlap.
    ///
    /// Returns up to `max` posts sorted by number of overlapping tags (descending),
    /// then by date (newest first). Excludes the current post.
    pub fn related_posts(&self, slug: &str, max: usize) -> Vec<&BlogPost> {
        let current = match self.get_post(slug) {
            Some(p) => p,
            None => return Vec::new(),
        };
        let current_tags: std::collections::HashSet<&str> = current
            .frontmatter
            .tags
            .iter()
            .map(|t| t.as_str())
            .collect();

        if current_tags.is_empty() {
            return Vec::new();
        }

        let mut scored: Vec<(usize, &BlogPost)> = self
            .posts
            .iter()
            .filter(|p| p.slug != slug)
            .filter_map(|p| {
                let overlap = p
                    .frontmatter
                    .tags
                    .iter()
                    .filter(|t| current_tags.contains(t.as_str()))
                    .count();
                if overlap > 0 {
                    Some((overlap, p))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| {
            b.0.cmp(&a.0)
                .then_with(|| b.1.frontmatter.date.cmp(&a.1.frontmatter.date))
        });
        scored.into_iter().take(max).map(|(_, p)| p).collect()
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

    /// Search posts by query string.
    ///
    /// Same multi-term AND / tier scoring as docs search (title > description >
    /// body); posts are indexed whole (no sections).
    pub fn search_posts(&self, query: &str) -> Vec<&BlogSearchEntry> {
        crate::search::rank(&self.search_index, query, |e, buf| {
            buf.push(crate::search::Field::title(&e.title_lower));
            if !e.description_lower.is_empty() {
                buf.push(crate::search::Field::description(&e.description_lower));
            }
            if !e.body_lower.is_empty() {
                buf.push(crate::search::Field::body(&e.body_lower));
            }
        })
    }

    fn build_search_index(posts: &[BlogPost]) -> Vec<BlogSearchEntry> {
        posts
            .iter()
            .map(|post| {
                let title = post.frontmatter.title.clone();
                let description = post.frontmatter.description.clone().unwrap_or_default();
                let body = crate::search::clean_markdown(&post.raw_markdown);
                BlogSearchEntry {
                    slug: post.slug.clone(),
                    title_lower: crate::search::search_lower(&title),
                    description_lower: crate::search::search_lower(&description),
                    body_lower: crate::search::search_lower(&body),
                    title,
                    description,
                    body,
                    date: post.frontmatter.date.clone(),
                    tags: post.frontmatter.tags.clone(),
                }
            })
            .collect()
    }

    // ── RSS ──────────────────────────────────────────────────────────────

    pub fn generate_rss(&self, site_title: &str, site_url: &str, blog_path: &str) -> String {
        let channel_title = xml_escape(site_title);
        let self_link = xml_escape(&format!("{site_url}{blog_path}"));
        let mut rss = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
<channel>
<title>{channel_title}</title>
<link>{self_link}</link>
<description>{channel_title} RSS Feed</description>
<atom:link href="{self_link}/rss.xml" rel="self" type="application/rss+xml"/>
"#
        );

        for post in &self.posts {
            let title = xml_escape(&post.frontmatter.title);
            let desc = xml_escape(post.frontmatter.description.as_deref().unwrap_or_default());
            let link = xml_escape(&format!("{site_url}{blog_path}/{}", post.slug));
            rss.push_str(&format!(
                "<item>\n<title>{title}</title>\n<link>{link}</link>\n<description>{desc}</description>\n<pubDate>{}</pubDate>\n<guid>{link}</guid>\n</item>\n",
                xml_escape(&post.frontmatter.date)
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
            let desc = post.frontmatter.description.as_deref().unwrap_or_default();
            let url = format!("{base_url}{blog_path}/{}", post.slug);
            if desc.is_empty() {
                out.push_str(&format!("- [{title}]({url})\n"));
            } else {
                out.push_str(&format!("- [{title}]({url}): {desc}\n"));
            }
        }

        out
    }

    // ── Sitemap ──────────────────────────────────────────────────────────

    /// Generate a sitemap.xml for all blog posts.
    pub fn generate_sitemap(&self, site_url: &str, blog_path: &str) -> String {
        let mut xml = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
        );

        // Blog index page
        let index_loc = xml_escape(&format!("{site_url}{blog_path}"));
        xml.push_str(&format!(
            "<url>\n<loc>{index_loc}</loc>\n<changefreq>weekly</changefreq>\n<priority>0.8</priority>\n</url>\n"
        ));

        // Individual posts
        for post in &self.posts {
            let loc = xml_escape(&format!("{site_url}{blog_path}/{}", post.slug));
            let lastmod = xml_escape(&post.frontmatter.date);
            xml.push_str(&format!(
                "<url>\n<loc>{loc}</loc>\n<lastmod>{lastmod}</lastmod>\n<changefreq>monthly</changefreq>\n<priority>0.6</priority>\n</url>\n"
            ));
        }

        xml.push_str("</urlset>\n");
        xml
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blog::config::BlogConfig;
    use std::collections::HashMap;

    fn build_registry(posts_per_page: usize) -> BlogRegistry {
        let manifest = r#"{
            "authors": {
                "author": { "name": "Author" }
            },
            "posts": ["featured", "regular-1", "regular-2", "regular-3", "rust-new", "rust-old", "misc"]
        }"#;

        let mut content_map = HashMap::new();
        content_map.insert(
            "featured",
            r#"---
title: "Featured"
date: "2026-03-21"
author: "author"
tags: ["announcement"]
featured: true
---
Featured post
"#,
        );
        content_map.insert(
            "regular-1",
            r#"---
title: "Regular 1"
date: "2026-03-20"
author: "author"
tags: ["announcement"]
---
Regular one
"#,
        );
        content_map.insert(
            "regular-2",
            r#"---
title: "Regular 2"
date: "2026-03-19"
author: "author"
tags: ["announcement"]
---
Regular two
"#,
        );
        content_map.insert(
            "regular-3",
            r#"---
title: "Regular 3"
date: "2026-03-18"
author: "author"
tags: ["announcement"]
---
Regular three
"#,
        );
        content_map.insert(
            "rust-new",
            r#"---
title: "Rust New"
date: "2026-03-17"
author: "author"
tags: ["rust", "web", "async"]
---
Rust new
"#,
        );
        content_map.insert(
            "rust-old",
            r#"---
title: "Rust Old"
date: "2026-03-16"
author: "author"
tags: ["rust", "web"]
---
Rust old
"#,
        );
        content_map.insert(
            "misc",
            r#"---
title: "Misc"
date: "2026-03-15"
author: "author"
tags: ["rust"]
---
Misc
"#,
        );

        BlogConfig::new(manifest, content_map)
            .with_posts_per_page(posts_per_page)
            .build()
    }

    #[test]
    fn unfiltered_pagination_excludes_featured_posts() {
        let registry = build_registry(2);

        let page_1: Vec<_> = registry
            .non_featured_posts_page(0)
            .into_iter()
            .map(|post| post.slug.as_str())
            .collect();
        let page_2: Vec<_> = registry
            .non_featured_posts_page(1)
            .into_iter()
            .map(|post| post.slug.as_str())
            .collect();
        let page_3: Vec<_> = registry
            .non_featured_posts_page(2)
            .into_iter()
            .map(|post| post.slug.as_str())
            .collect();
        let page_4 = registry.non_featured_posts_page(3);

        assert_eq!(page_1, vec!["regular-1", "regular-2"]);
        assert_eq!(page_2, vec!["regular-3", "rust-new"]);
        assert_eq!(page_3, vec!["rust-old", "misc"]);
        assert!(page_4.is_empty());
        assert_eq!(registry.non_featured_total_pages(), 3);
    }

    #[test]
    fn tag_pagination_still_includes_featured_posts() {
        let registry = build_registry(2);

        let page: Vec<_> = registry
            .posts_page_by_tag("announcement", 0)
            .into_iter()
            .map(|post| post.slug.as_str())
            .collect();

        assert_eq!(page, vec!["featured", "regular-1"]);
        assert_eq!(registry.total_pages_for_tag("announcement"), 2);
    }

    #[test]
    fn blog_search_matches_title_and_requires_all_terms() {
        let registry = build_registry(10);

        // Single term: both Rust posts match on title, newest first.
        let single: Vec<&str> = registry
            .search_posts("rust")
            .iter()
            .map(|e| e.slug.as_str())
            .collect();
        assert_eq!(single, vec!["rust-new", "rust-old"]);

        // Multi-term AND: only "Rust New" contains both words.
        let multi: Vec<&str> = registry
            .search_posts("rust new")
            .iter()
            .map(|e| e.slug.as_str())
            .collect();
        assert_eq!(multi, vec!["rust-new"]);

        assert!(registry.search_posts("   ").is_empty());
    }

    #[test]
    fn related_posts_tie_break_on_date() {
        let registry = build_registry(10);

        let related: Vec<_> = registry
            .related_posts("misc", 3)
            .into_iter()
            .map(|post| post.slug.as_str())
            .collect();

        assert_eq!(related, vec!["rust-new", "rust-old"]);
    }

    #[test]
    fn rss_escapes_xml_metacharacters() {
        let manifest = r#"{
            "authors": { "author": { "name": "Author" } },
            "posts": ["ampersand"]
        }"#;
        let mut content_map = HashMap::new();
        content_map.insert(
            "ampersand",
            "---\ntitle: \"Rust & WASM: <T> generics\"\ndate: \"2026-03-21\"\nauthor: \"author\"\ndescription: \"a \\\"quoted\\\" & thing\"\n---\nBody\n",
        );
        let registry = BlogConfig::new(manifest, content_map).build();

        let rss = registry.generate_rss("Site & Co", "https://example.com", "/blog");

        assert!(
            rss.contains("Rust &amp; WASM: &lt;T&gt; generics"),
            "got: {rss}"
        );
        assert!(rss.contains("Site &amp; Co"), "got: {rss}");
        // No bare `&` survives: every one must start an entity.
        for (idx, _) in rss.match_indices('&') {
            let tail = &rss[idx..];
            assert!(
                tail.starts_with("&amp;")
                    || tail.starts_with("&lt;")
                    || tail.starts_with("&gt;")
                    || tail.starts_with("&quot;")
                    || tail.starts_with("&apos;"),
                "unescaped `&` at {idx} makes the whole feed unparseable: {:?}",
                &tail[..tail.len().min(40)]
            );
        }
    }
}
