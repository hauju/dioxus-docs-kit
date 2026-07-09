//! Server-side (Axum) routes for crawler-facing endpoints.
//!
//! Available behind the `server` feature. These are plain Axum routes, **not**
//! server functions: a server function JSON-encodes its `String` return value
//! (quoted body, `application/json` content type), which robots.txt / sitemap /
//! llms.txt consumers can't parse.
//!
//! # Example
//!
//! ```rust,ignore
//! dioxus::server::serve(|| async {
//!     let seo = SeoRouter::new("https://example.com", "My Docs", "Documentation for My Product")
//!         .with_docs(&DOCS, "/docs")
//!         .with_blog(&BLOG, "/blog")
//!         .into_router();
//!     Ok(dioxus::server::router(App).merge(seo))
//! });
//! ```

use dioxus::server::axum::{Router, http::header, routing::get};

use crate::blog::BlogRegistry;
use crate::registry::DocsRegistry;

const TEXT: &str = "text/plain; charset=utf-8";
const XML: &str = "application/xml; charset=utf-8";
const MD: &str = "text/markdown; charset=utf-8";
const RSS: &str = "application/rss+xml; charset=utf-8";

/// Builder for the crawler-facing routes of a docs/blog site.
///
/// Generated routes:
///
/// | Route | Condition | Content |
/// |---|---|---|
/// | `<docs_path>/<page>.md` | docs | raw Markdown per doc page |
/// | `/llms.txt`, `/llms-full.txt` | docs | LLM-friendly docs index / full corpus |
/// | `/sitemap-docs.xml` | docs | docs sitemap |
/// | `<blog_path>/<slug>.md` | blog | raw Markdown per post |
/// | `<blog_path>/rss.xml` | blog | RSS feed |
/// | `/sitemap-blog.xml` | blog | blog sitemap |
/// | `/sitemap.xml` | always | sitemap index over the above |
/// | `/robots.txt` | always | allow-all incl. explicit AI-crawler entries |
pub struct SeoRouter {
    site_url: String,
    site_title: String,
    site_description: String,
    docs: Option<(&'static DocsRegistry, String)>,
    blog: Option<(&'static BlogRegistry, String)>,
}

/// Normalize a base path to the form `/docs` (leading slash, no trailing slash).
fn normalize_base(base_path: &str) -> String {
    let trimmed = base_path.trim_matches('/');
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("/{trimmed}")
    }
}

impl SeoRouter {
    /// Create a builder.
    ///
    /// - `site_url`: public origin, e.g. `"https://example.com"` (no trailing slash needed).
    /// - `site_title` / `site_description`: used in `llms.txt` headers and the RSS channel.
    pub fn new(site_url: &str, site_title: &str, site_description: &str) -> Self {
        Self {
            site_url: site_url.trim_end_matches('/').to_string(),
            site_title: site_title.to_string(),
            site_description: site_description.to_string(),
            docs: None,
            blog: None,
        }
    }

    /// Serve docs endpoints for `registry`, mounted under `base_path` (e.g. `"/docs"`).
    pub fn with_docs(mut self, registry: &'static DocsRegistry, base_path: &str) -> Self {
        self.docs = Some((registry, normalize_base(base_path)));
        self
    }

    /// Serve blog endpoints for `registry`, mounted under `base_path` (e.g. `"/blog"`).
    pub fn with_blog(mut self, registry: &'static BlogRegistry, base_path: &str) -> Self {
        self.blog = Some((registry, normalize_base(base_path)));
        self
    }

    /// Build the Axum router. Merge it into your app router with
    /// [`Router::merge`].
    pub fn into_router(self) -> Router {
        let mut router = Router::new();
        let site_url = &self.site_url;

        let mut sitemap_index_entries: Vec<String> = Vec::new();

        if let Some((docs, base)) = &self.docs {
            // Raw Markdown for each doc page at `<base>/<page>.md`. Registered as
            // literal routes, one per known doc path, so they take priority over
            // the SSR fallback without shadowing the HTML pages. OpenAPI endpoint
            // pages have no Markdown source.
            for path in docs.get_all_paths() {
                if let Some(markdown) = docs.get_doc_content(path) {
                    router = router.route(
                        &format!("{base}/{path}.md"),
                        get(move || async move { ([(header::CONTENT_TYPE, MD)], markdown) }),
                    );
                }
            }

            let docs_base_url = format!("{site_url}{base}");

            let llms =
                docs.generate_llms_txt(&self.site_title, &self.site_description, &docs_base_url);
            router = router.route(
                "/llms.txt",
                get(move || async move { ([(header::CONTENT_TYPE, TEXT)], llms) }),
            );

            let llms_full = docs.generate_llms_full_txt(
                &self.site_title,
                &self.site_description,
                &docs_base_url,
            );
            router = router.route(
                "/llms-full.txt",
                get(move || async move { ([(header::CONTENT_TYPE, TEXT)], llms_full) }),
            );

            let sitemap = docs.generate_sitemap(site_url, base);
            router = router.route(
                "/sitemap-docs.xml",
                get(move || async move { ([(header::CONTENT_TYPE, XML)], sitemap) }),
            );
            sitemap_index_entries.push(format!("{site_url}/sitemap-docs.xml"));
        }

        if let Some((blog, base)) = &self.blog {
            // Raw Markdown for each post at `<base>/<slug>.md`.
            for post in blog.all_posts() {
                let markdown = post.raw_markdown.as_str();
                router = router.route(
                    &format!("{base}/{}.md", post.slug),
                    get(move || async move { ([(header::CONTENT_TYPE, MD)], markdown) }),
                );
            }

            let rss = blog.generate_rss(&self.site_title, site_url, base);
            router = router.route(
                &format!("{base}/rss.xml"),
                get(move || async move { ([(header::CONTENT_TYPE, RSS)], rss) }),
            );

            let sitemap = blog.generate_sitemap(site_url, base);
            router = router.route(
                "/sitemap-blog.xml",
                get(move || async move { ([(header::CONTENT_TYPE, XML)], sitemap) }),
            );
            sitemap_index_entries.push(format!("{site_url}/sitemap-blog.xml"));
        }

        let sitemap_index = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <sitemapindex xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n{}\
             </sitemapindex>\n",
            sitemap_index_entries
                .iter()
                .map(|loc| format!("<sitemap><loc>{loc}</loc></sitemap>\n"))
                .collect::<String>()
        );
        router = router.route(
            "/sitemap.xml",
            get(move || async move { ([(header::CONTENT_TYPE, XML)], sitemap_index) }),
        );

        let robots = robots_txt(site_url);
        router.route(
            "/robots.txt",
            get(move || async move { ([(header::CONTENT_TYPE, TEXT)], robots) }),
        )
    }
}

/// Allow-all robots.txt with explicit per-AI-crawler sections.
///
/// AI crawlers are listed explicitly so each can be controlled with a single
/// line: flip its `Allow: /` to `Disallow: /` to block that bot. Covers
/// training crawlers (GPTBot, ClaudeBot, anthropic-ai, Google-Extended, CCBot)
/// and live-retrieval/search agents (ChatGPT-User, OAI-SearchBot,
/// PerplexityBot).
fn robots_txt(site_url: &str) -> String {
    const AI_CRAWLERS: &[&str] = &[
        "GPTBot",
        "ChatGPT-User",
        "OAI-SearchBot",
        "ClaudeBot",
        "anthropic-ai",
        "Claude-Web",
        "Google-Extended",
        "PerplexityBot",
        "CCBot",
    ];

    let mut out = String::from("User-agent: *\nAllow: /\n");
    for bot in AI_CRAWLERS {
        out.push_str(&format!("\nUser-agent: {bot}\nAllow: /\n"));
    }
    out.push_str(&format!("\nSitemap: {site_url}/sitemap.xml\n"));
    out
}

#[cfg(test)]
mod tests {
    use super::{normalize_base, robots_txt};

    #[test]
    fn normalizes_base_paths() {
        assert_eq!(normalize_base("/docs"), "/docs");
        assert_eq!(normalize_base("docs/"), "/docs");
        assert_eq!(normalize_base("/"), "");
        assert_eq!(normalize_base(""), "");
    }

    #[test]
    fn robots_txt_lists_ai_crawlers_and_sitemap() {
        let out = robots_txt("https://example.com");
        assert!(out.starts_with("User-agent: *\nAllow: /\n"));
        assert!(out.contains("User-agent: GPTBot\nAllow: /\n"));
        assert!(out.contains("User-agent: ClaudeBot\nAllow: /\n"));
        assert!(out.ends_with("Sitemap: https://example.com/sitemap.xml\n"));
    }
}
