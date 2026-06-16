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

/// Build a schema.org JSON-LD `@graph` for a docs page: a `TechArticle` plus an
/// optional `BreadcrumbList`. `</` is escaped to `<\/` so the payload cannot
/// break out of its `<script>` container.
///
/// `breadcrumbs` is an ordered `(name, url)` list, root first; a `None` url
/// emits a name-only `ListItem` (used for the current page as the last crumb).
fn build_docs_jsonld(
    title: &str,
    description: &str,
    canonical: Option<&str>,
    breadcrumbs: &[(String, Option<String>)],
) -> String {
    let mut tech_article = serde_json::json!({
        "@type": "TechArticle",
        "headline": title,
        "description": description,
    });
    if let Some(url) = canonical {
        tech_article["mainEntityOfPage"] = serde_json::json!({
            "@type": "WebPage",
            "@id": url,
        });
    }

    let mut graph = vec![tech_article];

    if !breadcrumbs.is_empty() {
        let items: Vec<serde_json::Value> = breadcrumbs
            .iter()
            .enumerate()
            .map(|(i, (name, url))| {
                let mut item = serde_json::json!({
                    "@type": "ListItem",
                    "position": i + 1,
                    "name": name,
                });
                if let Some(url) = url {
                    item["item"] = serde_json::Value::String(url.clone());
                }
                item
            })
            .collect();
        graph.push(serde_json::json!({
            "@type": "BreadcrumbList",
            "itemListElement": items,
        }));
    }

    let payload = serde_json::json!({
        "@context": "https://schema.org",
        "@graph": graph,
    });

    serde_json::to_string(&payload)
        .unwrap_or_default()
        .replace("</", "<\\/")
}

/// Injects SEO meta tags and document title for a single docs page (MDX or API endpoint).
///
/// Reads `auto_meta` and `site_url` from [`DocsContext`]. When `auto_meta` is
/// off, emits nothing. Otherwise pulls title/description from the registry —
/// frontmatter for MDX pages, the OpenAPI operation's `summary`/`description`
/// for API endpoint pages — and emits `<title>`, `<meta name="description">`,
/// Open Graph, Twitter Card, and schema.org `TechArticle` JSON-LD tags.
/// Canonical, `og:url`, the JSON-LD `@id`, and a `BreadcrumbList` are only
/// emitted when `site_url` is also set.
#[component]
pub fn DocsPageMeta(path: String) -> Element {
    let registry = use_context::<&'static DocsRegistry>();
    let ctx = use_context::<DocsContext>();

    if !ctx.auto_meta {
        return rsx! {};
    }

    // `is_mdx` gates the raw-Markdown alternate link: OpenAPI endpoint pages are
    // rendered dynamically and have no `.md` source.
    let (title, description, is_mdx) = if let Some(op) = registry.get_api_operation(&path) {
        let title = op
            .summary
            .clone()
            .unwrap_or_else(|| op.slug().replace('-', " "));
        (title, op.description.clone().unwrap_or_default(), false)
    } else if let Some(doc) = registry.get_parsed_doc(&path) {
        (
            doc.frontmatter.title.clone(),
            doc.frontmatter.description.clone().unwrap_or_default(),
            true,
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

    // Root-relative `<base_path>/<path>.md`; `join_site_url` with an empty origin
    // yields the path portion only.
    let markdown_href = (is_mdx && ctx.markdown_alternate)
        .then(|| format!("{}.md", join_site_url("", &ctx.base_path, &path)));

    // Breadcrumb trail: Docs root → group → current page. Built only when
    // `site_url` is set (schema.org breadcrumb items need absolute URLs). The
    // group's URL points at its first page, since groups have no landing page.
    let breadcrumbs: Vec<(String, Option<String>)> = match ctx.site_url.as_deref() {
        Some(origin) => {
            let root_label = registry
                .tab_for_path(&path)
                .unwrap_or_else(|| "Docs".to_string());
            let mut trail = vec![(root_label, Some(join_site_url(origin, &ctx.base_path, "")))];
            if let Some(group) = registry
                .nav
                .groups
                .iter()
                .find(|g| g.pages.iter().any(|p| p == &path))
                && let Some(first) = group.pages.first()
            {
                trail.push((
                    group.group.clone(),
                    Some(join_site_url(origin, &ctx.base_path, first)),
                ));
            }
            trail.push((title.clone(), canonical.clone()));
            trail
        }
        None => Vec::new(),
    };

    let json_ld = build_docs_jsonld(&title, &description, canonical.as_deref(), &breadcrumbs);

    rsx! {
        document::Title { "{title}" }
        document::Meta { name: "description", content: "{description}" }
        if let Some(ref url) = canonical {
            document::Link { rel: "canonical", href: "{url}" }
        }
        if let Some(ref href) = markdown_href {
            document::Link { rel: "alternate", r#type: "text/markdown", href: "{href}" }
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

        // schema.org TechArticle (+ BreadcrumbList) JSON-LD for rich results.
        document::Script { r#type: "application/ld+json", "{json_ld}" }
    }
}

#[cfg(test)]
mod tests {
    use super::{build_docs_jsonld, join_site_url};

    #[test]
    fn jsonld_emits_techarticle_with_id_when_canonical_present() {
        let out = build_docs_jsonld(
            "Introduction",
            "Get started",
            Some("https://example.com/docs/getting-started/intro"),
            &[],
        );
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["@context"], "https://schema.org");
        let graph = parsed["@graph"].as_array().unwrap();
        assert_eq!(graph.len(), 1);
        assert_eq!(graph[0]["@type"], "TechArticle");
        assert_eq!(graph[0]["headline"], "Introduction");
        assert_eq!(
            graph[0]["mainEntityOfPage"]["@id"],
            "https://example.com/docs/getting-started/intro"
        );
    }

    #[test]
    fn jsonld_omits_id_without_canonical_and_breadcrumb_when_empty() {
        let out = build_docs_jsonld("Introduction", "Get started", None, &[]);
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        let graph = parsed["@graph"].as_array().unwrap();
        assert_eq!(graph.len(), 1);
        assert!(graph[0].get("mainEntityOfPage").is_none());
    }

    #[test]
    fn jsonld_appends_positioned_breadcrumb_list() {
        let crumbs = vec![
            (
                "Docs".to_string(),
                Some("https://example.com/docs".to_string()),
            ),
            (
                "Getting Started".to_string(),
                Some("https://example.com/docs/getting-started/intro".to_string()),
            ),
            ("Introduction".to_string(), None),
        ];
        let out = build_docs_jsonld("Introduction", "Get started", None, &crumbs);
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        let graph = parsed["@graph"].as_array().unwrap();
        assert_eq!(graph.len(), 2);
        let bc = &graph[1];
        assert_eq!(bc["@type"], "BreadcrumbList");
        let items = bc["itemListElement"].as_array().unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0]["position"], 1);
        assert_eq!(items[1]["name"], "Getting Started");
        assert_eq!(items[2]["position"], 3);
        // Last crumb (current page) has a name but no `item` URL.
        assert!(items[2].get("item").is_none());
    }

    #[test]
    fn jsonld_escapes_script_close_sequence() {
        let out = build_docs_jsonld(
            "evil </script><script>alert(1)</script>",
            "",
            Some("https://example.com/"),
            &[],
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
