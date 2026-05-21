use dioxus::prelude::*;

use crate::BlogContext;
use crate::blog::registry::BlogRegistry;

fn join_site_url(site_url: &str, base_path: &str, slug: Option<&str>) -> String {
    let mut url = site_url.trim_end_matches('/').to_string();

    if !base_path.is_empty() {
        if !base_path.starts_with('/') {
            url.push('/');
        }
        url.push_str(base_path.trim_end_matches('/'));
    }

    if let Some(slug) = slug
        && !slug.is_empty()
    {
        url.push('/');
        url.push_str(slug.trim_start_matches('/'));
    }

    url
}

/// Build a schema.org Article JSON-LD string, with `</` escaped to `<\/` so the
/// payload cannot break out of its `<script>` container.
fn build_article_jsonld(
    title: &str,
    description: &str,
    url: &str,
    date: &str,
    author_name: &str,
    image: Option<&str>,
) -> String {
    let mut payload = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "Article",
        "headline": title,
        "description": description,
        "datePublished": date,
        "mainEntityOfPage": {
            "@type": "WebPage",
            "@id": url,
        },
    });

    if !author_name.is_empty() {
        payload["author"] = serde_json::json!({
            "@type": "Person",
            "name": author_name,
        });
    }
    if let Some(image) = image {
        payload["image"] = serde_json::Value::String(image.to_string());
    }

    serde_json::to_string(&payload)
        .unwrap_or_default()
        .replace("</", "<\\/")
}

/// Injects Open Graph / SEO meta tags and document title for a single blog post.
#[component]
pub fn BlogPostMeta(slug: String, site_url: String) -> Element {
    let registry = use_context::<&'static BlogRegistry>();
    let ctx = use_context::<BlogContext>();

    let post = match registry.get_post(&slug) {
        Some(p) => p,
        None => return rsx! {},
    };

    let title = &post.frontmatter.title;
    let description = post.frontmatter.description.as_deref().unwrap_or("");
    let url = join_site_url(&site_url, &ctx.base_path, Some(&slug));
    let date = &post.frontmatter.date;
    let author_name = registry
        .get_author(&post.frontmatter.author)
        .map(|a| a.name.as_str())
        .unwrap_or("");

    let json_ld = build_article_jsonld(
        title,
        description,
        &url,
        date,
        author_name,
        post.frontmatter.cover_image.as_deref(),
    );

    rsx! {
        document::Title { "{title}" }
        document::Meta { name: "description", content: "{description}" }
        document::Link { rel: "canonical", href: "{url}" }

        // Open Graph
        document::Meta { property: "og:title", content: "{title}" }
        document::Meta { property: "og:description", content: "{description}" }
        document::Meta { property: "og:type", content: "article" }
        document::Meta { property: "og:url", content: "{url}" }
        if let Some(ref cover) = post.frontmatter.cover_image {
            document::Meta { property: "og:image", content: "{cover}" }
        }

        // Twitter Card
        document::Meta { name: "twitter:card", content: "summary_large_image" }
        document::Meta { name: "twitter:title", content: "{title}" }
        document::Meta { name: "twitter:description", content: "{description}" }
        if let Some(ref cover) = post.frontmatter.cover_image {
            document::Meta { name: "twitter:image", content: "{cover}" }
        }

        // Article metadata
        document::Meta { property: "article:published_time", content: "{date}" }
        if !author_name.is_empty() {
            document::Meta { property: "article:author", content: "{author_name}" }
        }
        for tag in post.frontmatter.tags.iter() {
            document::Meta { property: "article:tag", content: "{tag}" }
        }

        // schema.org Article JSON-LD for rich-result eligibility.
        document::Script { r#type: "application/ld+json", "{json_ld}" }
    }
}

/// Injects basic SEO meta tags for the blog index/listing page.
#[component]
pub fn BlogIndexMeta(title: String, description: String, site_url: String) -> Element {
    let ctx = use_context::<BlogContext>();
    let url = join_site_url(&site_url, &ctx.base_path, None);

    rsx! {
        document::Title { "{title}" }
        document::Meta { name: "description", content: "{description}" }
        document::Link { rel: "canonical", href: "{url}" }
        document::Meta { property: "og:title", content: "{title}" }
        document::Meta { property: "og:description", content: "{description}" }
        document::Meta { property: "og:type", content: "website" }
        document::Meta { property: "og:url", content: "{url}" }
        document::Meta { name: "twitter:card", content: "summary" }
        document::Meta { name: "twitter:title", content: "{title}" }
        document::Meta { name: "twitter:description", content: "{description}" }
    }
}

#[cfg(test)]
mod tests {
    use super::{build_article_jsonld, join_site_url};

    #[test]
    fn jsonld_includes_required_fields() {
        let out = build_article_jsonld(
            "Hello",
            "A post",
            "https://example.com/blog/hello",
            "2026-05-21",
            "Jane",
            Some("https://example.com/cover.png"),
        );
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["@context"], "https://schema.org");
        assert_eq!(parsed["@type"], "Article");
        assert_eq!(parsed["headline"], "Hello");
        assert_eq!(parsed["description"], "A post");
        assert_eq!(parsed["datePublished"], "2026-05-21");
        assert_eq!(parsed["author"]["@type"], "Person");
        assert_eq!(parsed["author"]["name"], "Jane");
        assert_eq!(parsed["image"], "https://example.com/cover.png");
        assert_eq!(
            parsed["mainEntityOfPage"]["@id"],
            "https://example.com/blog/hello"
        );
    }

    #[test]
    fn jsonld_omits_author_and_image_when_missing() {
        let out = build_article_jsonld(
            "Hello",
            "A post",
            "https://example.com/blog/hello",
            "2026-05-21",
            "",
            None,
        );
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert!(parsed.get("author").is_none());
        assert!(parsed.get("image").is_none());
    }

    #[test]
    fn jsonld_escapes_script_close_sequence() {
        // A title containing `</script>` must not break out of the <script> tag.
        let out = build_article_jsonld(
            "evil </script><script>alert(1)</script>",
            "",
            "https://example.com/",
            "2026-05-21",
            "",
            None,
        );
        assert!(
            !out.contains("</script"),
            "expected </ sequences to be escaped, got: {out}"
        );
        assert!(out.contains("<\\/script"));
    }

    #[test]
    fn joins_site_url_without_duplicate_slashes() {
        assert_eq!(
            join_site_url("https://example.com/", "/blog/", Some("hello-world")),
            "https://example.com/blog/hello-world"
        );
        assert_eq!(
            join_site_url("https://example.com", "blog", Some("/hello-world")),
            "https://example.com/blog/hello-world"
        );
        assert_eq!(
            join_site_url("https://example.com/", "/blog/", None),
            "https://example.com/blog"
        );
    }
}
