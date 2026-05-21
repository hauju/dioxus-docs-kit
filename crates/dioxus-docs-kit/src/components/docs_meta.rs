use dioxus::prelude::*;

use crate::DocsContext;
use crate::registry::DocsRegistry;

fn join_site_url(site_url: &str, base_path: &str, path: &str) -> String {
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

/// Injects SEO meta tags and document title for a single docs page (MDX or API endpoint).
///
/// Reads `auto_meta` and `site_url` from [`DocsContext`]. When `auto_meta` is
/// off, emits nothing. Otherwise pulls title/description from the registry —
/// frontmatter for MDX pages, the OpenAPI operation's `summary`/`description`
/// for API endpoint pages — and emits `<title>`, `<meta name="description">`,
/// Open Graph and Twitter Card tags. Canonical and `og:url` are only emitted
/// when `site_url` is also set.
#[component]
pub fn DocsPageMeta(path: String) -> Element {
    let registry = use_context::<&'static DocsRegistry>();
    let ctx = use_context::<DocsContext>();

    if !ctx.auto_meta {
        return rsx! {};
    }

    let (title, description) = if let Some(op) = registry.get_api_operation(&path) {
        let title = op
            .summary
            .clone()
            .unwrap_or_else(|| op.slug().replace('-', " "));
        (title, op.description.clone().unwrap_or_default())
    } else if let Some(doc) = registry.get_parsed_doc(&path) {
        (
            doc.frontmatter.title.clone(),
            doc.frontmatter.description.clone().unwrap_or_default(),
        )
    } else {
        return rsx! {};
    };

    if title.is_empty() {
        return rsx! {};
    }

    let canonical = ctx
        .site_url
        .as_deref()
        .map(|origin| join_site_url(origin, &ctx.base_path, &path));

    rsx! {
        document::Title { "{title}" }
        document::Meta { name: "description", content: "{description}" }
        if let Some(ref url) = canonical {
            document::Link { rel: "canonical", href: "{url}" }
        }

        // Open Graph
        document::Meta { property: "og:title", content: "{title}" }
        document::Meta { property: "og:description", content: "{description}" }
        document::Meta { property: "og:type", content: "article" }
        if let Some(ref url) = canonical {
            document::Meta { property: "og:url", content: "{url}" }
        }

        // Twitter Card
        document::Meta { name: "twitter:card", content: "summary" }
        document::Meta { name: "twitter:title", content: "{title}" }
        document::Meta { name: "twitter:description", content: "{description}" }
    }
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
