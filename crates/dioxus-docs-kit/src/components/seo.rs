//! Shared SEO helpers for the docs and blog meta components.

/// Join a site origin, base path, and page path into an absolute URL,
/// normalizing duplicate slashes. An empty `path` yields the base URL only;
/// an empty `site_url` yields the root-relative path portion.
pub(crate) fn join_site_url(site_url: &str, base_path: &str, path: &str) -> String {
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
pub(crate) fn jsonld_to_string(payload: &serde_json::Value) -> String {
    serde_json::to_string(payload)
        .unwrap_or_default()
        .replace("</", "<\\/")
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
