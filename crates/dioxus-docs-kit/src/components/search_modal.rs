use dioxus::prelude::*;
use dioxus_mdx::HttpMethod;

use super::search_shell::{SearchHit, SearchModalShell};
use crate::DocsContext;
use crate::registry::DocsRegistry;
use crate::search::{MAX_RESULTS, SNIPPET_WINDOW, build_snippet, split_terms};

/// Rank `query` against the docs index and build at most `limit` rendered hits.
///
/// The cap is applied *before* the map: the query re-runs on every keystroke,
/// and each hit costs a snippet scan plus a mounted component, so building hits
/// the modal cannot show is the dominant cost of a broad query.
fn docs_hits(registry: &'static DocsRegistry, query: &str, limit: usize) -> Vec<SearchHit> {
    let terms = split_terms(query);
    registry
        .search_docs(query)
        .into_iter()
        .take(limit)
        .map(|entry| {
            // Section hits deep-link via `path#anchor`; page-level hits use
            // the bare path.
            let target = if entry.anchor.is_empty() {
                entry.path.clone()
            } else {
                format!("{}#{}", entry.path, entry.anchor)
            };
            // Section hits show the heading with the page title as context.
            let (title, context) = if entry.heading.is_empty() {
                (entry.title.clone(), None)
            } else {
                (entry.heading.clone(), Some(entry.title.clone()))
            };
            let snippet_src = if entry.body.is_empty() {
                &entry.description
            } else {
                &entry.body
            };
            SearchHit {
                target,
                title,
                context,
                badge: entry.api_method.map(method_badge),
                meta: entry.breadcrumb.clone(),
                tags: Vec::new(),
                snippet: build_snippet(snippet_src, &terms, SNIPPET_WINDOW),
            }
        })
        .collect()
}

/// Full-screen search modal triggered by Cmd/Ctrl+K or the search button.
#[component]
pub fn SearchModal() -> Element {
    let ctx = use_context::<DocsContext>();
    let registry = use_context::<&'static DocsRegistry>();

    let search = use_callback(move |query: String| docs_hits(registry, &query, MAX_RESULTS));

    let on_select = use_callback(move |target: String| {
        // `target` is `path` or `path#anchor`.
        let mut parts = target.splitn(2, '#');
        let path = parts.next().unwrap_or_default().to_string();
        let anchor = parts.next().unwrap_or_default().to_string();

        (ctx.navigate)(path);

        // Same-page selection is a navigate no-op, so scroll regardless.
        #[cfg(target_arch = "wasm32")]
        if !anchor.is_empty() {
            scroll_to_anchor(anchor);
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = anchor;
    });

    rsx! {
        SearchModalShell {
            placeholder: "Search documentation...",
            search,
            on_select,
        }
    }
}

/// Scroll a freshly navigated page to a heading anchor.
///
/// The new page's DOM mounts *after* navigation returns, so retry over a few
/// animation frames until the element exists (mirrors the TOC scroll JS).
#[cfg(target_arch = "wasm32")]
fn scroll_to_anchor(anchor: String) {
    spawn(async move {
        let js = format!(
            r#"
            (function() {{
                const id = {};
                let attempts = 0;
                function tryScroll() {{
                    const el = document.getElementById(id);
                    if (el) {{
                        el.scrollIntoView({{ behavior: 'smooth', block: 'start' }});
                        history.replaceState(null, '', '#' + id);
                        return;
                    }}
                    if (attempts++ < 20) {{
                        requestAnimationFrame(tryScroll);
                    }}
                }}
                requestAnimationFrame(tryScroll);
            }})();
            "#,
            serde_json::to_string(&anchor).unwrap_or_default()
        );
        let _ = document::eval(&js);
    });
}

fn method_badge(method: HttpMethod) -> (&'static str, &'static str) {
    match method {
        HttpMethod::Get => ("GET", "badge-soft badge-success"),
        HttpMethod::Post => ("POST", "badge-soft badge-primary"),
        HttpMethod::Put => ("PUT", "badge-soft badge-warning"),
        HttpMethod::Delete => ("DEL", "badge-soft badge-error"),
        HttpMethod::Patch => ("PATCH", "badge-soft badge-info"),
        _ => ("???", "badge-soft badge-ghost"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DocsConfig;
    use std::collections::HashMap;

    /// Registry with `pages` pages that all match the query "widget".
    ///
    /// Leaks, which is fine in a test and is what lets us hand `docs_hits` the
    /// `&'static DocsRegistry` the component would get from context.
    fn wide_registry(pages: usize) -> &'static DocsRegistry {
        let mut nav_pages: Vec<String> = Vec::new();
        let mut map: HashMap<&'static str, &'static str> = HashMap::new();

        for i in 0..pages {
            let path: &'static str = Box::leak(format!("g/page-{i}").into_boxed_str());
            let body: &'static str = Box::leak(
                format!("---\ntitle: Widget {i}\n---\n\nThe widget keyword appears here.\n")
                    .into_boxed_str(),
            );
            nav_pages.push(format!("\"{path}\""));
            map.insert(path, body);
        }

        let nav: &'static str = Box::leak(
            format!(
                r#"{{ "groups": [ {{ "group": "G", "pages": [{}] }} ] }}"#,
                nav_pages.join(",")
            )
            .into_boxed_str(),
        );

        Box::leak(Box::new(DocsConfig::new(nav, map).build()))
    }

    #[test]
    fn docs_hits_caps_rendered_results() {
        let registry = wide_registry(40);
        // The fixture has to exceed the cap or the test proves nothing.
        assert!(
            registry.search_docs("widget").len() > 25,
            "fixture should match more than the cap"
        );
        assert_eq!(docs_hits(registry, "widget", 25).len(), 25);
    }

    #[test]
    fn docs_hits_returns_every_match_below_the_cap() {
        let registry = wide_registry(3);
        assert_eq!(docs_hits(registry, "widget", 25).len(), 3);
    }

    #[test]
    fn docs_hits_builds_snippets_only_for_returned_rows() {
        let registry = wide_registry(40);
        let hits = docs_hits(registry, "widget", 5);
        assert_eq!(hits.len(), 5);
        // Every returned row is fully built (the cap must not truncate work
        // that the rendered rows still need).
        assert!(hits.iter().all(|h| !h.snippet.is_empty()));
    }

    #[test]
    fn docs_hits_empty_query_yields_nothing() {
        let registry = wide_registry(3);
        assert!(docs_hits(registry, "", 25).is_empty());
    }
}
