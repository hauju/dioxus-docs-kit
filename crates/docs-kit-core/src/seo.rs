//! Shared SEO helpers for the docs and blog meta components.

/// Join a site origin, base path, and page path into an absolute URL,
/// normalizing duplicate slashes. An empty `path` yields the base URL only;
/// an empty `site_url` yields the root-relative path portion.
pub fn join_site_url(site_url: &str, base_path: &str, path: &str) -> String {
    let mut url = site_url.trim_end_matches('/').to_string();

    if !base_path.is_empty() {
        if !base_path.starts_with('/') {
            url.push('/');
        }
        url.push_str(base_path.trim_end_matches('/'));
    }

    if !path.is_empty() {
        url.push('/');
        url.push_str(path.trim_start_matches('/'));
    }

    url
}

/// Serialize a JSON-LD payload for embedding in a `<script>` tag.
///
/// `</` is escaped to `<\/` so the payload cannot break out of its
/// `<script>` container.
pub fn jsonld_to_string(payload: &serde_json::Value) -> String {
    serde_json::to_string(payload)
        .unwrap_or_default()
        .replace("</", "<\\/")
}

/// Escape text for interpolation into an XML element or attribute.
///
/// RSS and sitemap output is parsed strictly: a single unescaped `&` in a post
/// title makes readers reject the whole feed, not just that item.
pub fn xml_escape(value: &str) -> String {
    // `&` first, or the ampersands introduced by the later replacements get
    // escaped a second time.
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Build a schema.org Article JSON-LD string, with `</` escaped to `<\/` so the
/// payload cannot break out of its `<script>` container.
pub fn article_jsonld(
    title: &str,
    description: &str,
    url: Option<&str>,
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
    });

    if let Some(url) = url {
        payload["mainEntityOfPage"] = serde_json::json!({
            "@type": "WebPage",
            "@id": url,
        });
    }
    if !author_name.is_empty() {
        payload["author"] = serde_json::json!({
            "@type": "Person",
            "name": author_name,
        });
    }
    if let Some(image) = image {
        payload["image"] = serde_json::Value::String(image.to_string());
    }

    jsonld_to_string(&payload)
}

#[cfg(test)]
mod tests {
    use super::join_site_url;

    #[test]
    fn joins_site_url_without_duplicate_slashes() {
        assert_eq!(
            join_site_url("https://example.com/", "/docs/", "getting-started/intro"),
            "https://example.com/docs/getting-started/intro"
        );
        assert_eq!(
            join_site_url("https://example.com", "docs", "/getting-started/intro"),
            "https://example.com/docs/getting-started/intro"
        );
        assert_eq!(
            join_site_url("https://example.com/", "/docs/", ""),
            "https://example.com/docs"
        );
    }

    #[test]
    fn joins_without_base_path() {
        assert_eq!(
            join_site_url("https://example.com", "", "page"),
            "https://example.com/page"
        );
    }
}

#[cfg(test)]
mod article_tests {
    use super::article_jsonld;

    #[test]
    fn jsonld_includes_required_fields() {
        let out = article_jsonld(
            "Hello",
            "A post",
            Some("https://example.com/blog/hello"),
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
        let out = article_jsonld("Hello", "A post", None, "2026-05-21", "", None);
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert!(parsed.get("author").is_none());
        assert!(parsed.get("image").is_none());
        assert!(parsed.get("mainEntityOfPage").is_none());
    }

    #[test]
    fn jsonld_escapes_script_close_sequence() {
        // A title containing `</script>` must not break out of the <script> tag.
        let out = article_jsonld(
            "evil </script><script>alert(1)</script>",
            "",
            Some("https://example.com/"),
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
}
