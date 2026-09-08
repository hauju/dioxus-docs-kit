//! Blog content registry.

use crate::blog::config::BlogConfig;
use crate::blog::types::{
    Author, BlogCategory, BlogManifest, BlogPost, BlogSearchEntry, calculate_reading_time,
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
    categories: Vec<BlogCategory>,
    category_base_path: Option<String>,
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
        if config.posts_per_page() == 0 {
            return Err(DocsKitError::BlogConfig(
                "posts_per_page must be greater than zero".into(),
            ));
        }
        if let Some(path) = config.category_base_path() {
            super::categories::validate_base_path(path)?;
        }

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

        posts.sort_by(|a, b| {
            b.frontmatter
                .date
                .cmp(&a.frontmatter.date)
                .then_with(|| a.slug.cmp(&b.slug))
        });

        let mut tag_set: Vec<String> = posts
            .iter()
            .flat_map(|p| p.frontmatter.tags.iter().cloned())
            .collect();
        tag_set.sort();
        tag_set.dedup();

        let categories = if config.category_base_path().is_some() {
            super::categories::build_categories(&tag_set, &manifest.categories)?
        } else {
            Vec::new()
        };
        let category_base_path = config.category_base_path().map(str::to_string);

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
            categories,
            category_base_path,
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

    /// Published categories, in tag order. Empty when category routes are disabled.
    pub fn categories(&self) -> &[BlogCategory] {
        &self.categories
    }

    pub fn category_base_path(&self) -> Option<&str> {
        self.category_base_path.as_deref()
    }

    pub fn get_category(&self, slug: &str) -> Option<&BlogCategory> {
        self.categories
            .iter()
            .find(|category| category.slug == slug)
    }

    pub fn category_for_tag(&self, tag: &str) -> Option<&BlogCategory> {
        self.categories.iter().find(|category| category.tag == tag)
    }

    /// Category pagination uses one-based page numbers and includes featured posts.
    /// Unknown categories and out-of-range pages return `None` (including page zero).
    pub fn category_posts_page(&self, slug: &str, page: usize) -> Option<Vec<&BlogPost>> {
        let category = self.get_category(slug)?;
        if page == 0 || page > self.total_pages_for_tag(&category.tag) {
            return None;
        }
        Some(self.posts_page_by_tag(&category.tag, page - 1))
    }

    /// Canonical root-relative URL; page one has no pagination suffix.
    pub fn category_url(&self, slug: &str, page: usize) -> Option<String> {
        let category = self.get_category(slug)?;
        let base = self.category_base_path()?;
        if page == 0 || page > self.total_pages_for_tag(&category.tag) {
            return None;
        }
        let slug = super::categories::encode_path_segment(&category.slug);
        Some(if page == 1 {
            format!("{base}/{slug}")
        } else {
            format!("{base}/{slug}/page/{page}")
        })
    }

    pub fn category_url_for_tag(&self, tag: &str) -> Option<String> {
        self.category_url(&self.category_for_tag(tag)?.slug, 1)
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

    /// Generate a sitemap.xml for posts and enabled category pages.
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

        for category in &self.categories {
            for page in 1..=self.total_pages_for_tag(&category.tag) {
                if let Some(path) = self.category_url(&category.slug, page) {
                    let loc =
                        xml_escape(&crate::components::seo::join_site_url(site_url, &path, ""));
                    xml.push_str(&format!(
                        "<url>\n<loc>{loc}</loc>\n<changefreq>weekly</changefreq>\n</url>\n"
                    ));
                }
            }
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

    fn category_config() -> BlogConfig {
        BlogConfig::new(r#"{"authors":{}, "posts":[], "categories":{
            "Rust": {"slug":"rust-lang", "title":"Rust programming", "description":"Learn Rust.", "image":"/rust.png"},
            "unused": {"title":"Unused"}
        }}"#, HashMap::from([
            ("a", "---\ntitle: A\ndate: '2026-01-02'\nauthor: a\ntags: [Rust]\nfeatured: true\n---\nA"),
            ("b", "---\ntitle: B\ndate: '2026-01-02'\nauthor: a\ntags: [Rust, Web]\n---\nB"),
            ("c", "---\ntitle: C\ndate: '2026-01-01'\nauthor: a\ntags: [Rust]\n---\nC"),
            ("draft", "---\ntitle: Draft\ndate: '2026-01-03'\nauthor: a\ntags: [Rust, Secret]\ndraft: true\n---\nDraft"),
        ]))
        .with_category_base_path("/topics/")
        .with_posts_per_page(2)
    }

    #[test]
    fn categories_use_published_tags_and_optional_metadata() {
        let registry = category_config().build();
        assert_eq!(registry.categories().len(), 2);
        let rust = registry.get_category("rust-lang").unwrap();
        assert_eq!(rust.title, "Rust programming");
        assert_eq!(rust.description, "Learn Rust.");
        assert_eq!(rust.image.as_deref(), Some("/rust.png"));
        assert_eq!(registry.category_for_tag("Rust"), Some(rust));
        assert!(registry.get_category("secret").is_none());
        assert!(registry.get_category("unused").is_none());
        assert_eq!(
            registry.get_category("web").unwrap().description,
            "Browse articles about Web."
        );
    }

    #[test]
    fn category_urls_percent_encode_non_ascii_slugs() {
        let registry = BlogConfig::new(
            r#"{"authors":{}, "posts":[], "categories":{}}"#,
            HashMap::from([(
                "a",
                "---\ntitle: A\ndate: '2026-01-02'\nauthor: a\ntags: [Café]\n---\nA",
            )]),
        )
        .with_category_base_path("/topics")
        .build();
        let category = registry.category_for_tag("Café").unwrap();
        assert_eq!(category.slug, "café");
        assert_eq!(
            registry.category_url("café", 1).as_deref(),
            Some("/topics/caf%C3%A9")
        );
        assert!(
            registry
                .generate_sitemap("https://example.com", "/blog")
                .contains("<loc>https://example.com/topics/caf%C3%A9</loc>")
        );
    }

    #[test]
    fn category_pages_include_featured_and_have_stable_order() {
        let registry = category_config().build();
        let slugs = |page| {
            registry
                .category_posts_page("rust-lang", page)
                .unwrap()
                .iter()
                .map(|post| post.slug.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(slugs(1), ["a", "b"]);
        assert_eq!(slugs(2), ["c"]);
        for page in [0, 3, usize::MAX] {
            assert!(registry.category_posts_page("rust-lang", page).is_none());
            assert!(registry.category_url("rust-lang", page).is_none());
        }
        assert!(registry.category_posts_page("missing", 1).is_none());
        assert_eq!(
            registry.category_url_for_tag("Rust").as_deref(),
            Some("/topics/rust-lang")
        );
        assert_eq!(
            registry.category_url("rust-lang", 2).as_deref(),
            Some("/topics/rust-lang/page/2")
        );
    }

    #[test]
    fn sitemap_contains_only_valid_category_pages() {
        let registry = category_config().build();
        let xml = registry.generate_sitemap("https://example.com", "/blog");
        for path in [
            "/topics/rust-lang",
            "/topics/rust-lang/page/2",
            "/topics/web",
        ] {
            assert_eq!(
                xml.matches(&format!("<loc>https://example.com{path}</loc>"))
                    .count(),
                1
            );
        }
        assert!(!xml.contains("/page/1"));
        assert!(!xml.contains("/page/3"));
        assert!(!xml.contains("secret"));
        assert!(!xml.contains("unused"));
    }

    #[test]
    fn category_routes_are_opt_in_and_zero_page_size_is_rejected() {
        let registry = build_registry(2);
        assert!(registry.categories().is_empty());
        assert!(registry.category_url_for_tag("rust").is_none());
        assert!(
            !registry
                .generate_sitemap("https://example.com", "/blog")
                .contains("/categories/")
        );
        assert!(matches!(
            category_config().with_posts_per_page(0).try_build(),
            Err(DocsKitError::BlogConfig(_))
        ));
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
