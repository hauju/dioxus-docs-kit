use dioxus::prelude::*;
use dioxus_mdx::HttpMethod;

use super::search_shell::{SearchHit, SearchModalShell};
use crate::DocsContext;
use crate::registry::DocsRegistry;
use crate::search::{MAX_RESULTS, SNIPPET_WINDOW, build_snippet, split_terms};

/// Full-screen search modal triggered by Cmd/Ctrl+K or the search button.
#[component]
pub fn SearchModal() -> Element {
    let ctx = use_context::<DocsContext>();
    let registry = use_context::<&'static DocsRegistry>();

    let search = use_callback(move |query: String| {
        let terms = split_terms(&query);
        registry
            .search_docs(&query)
            .into_iter()
            // Cap before building hits: snippet extraction is the expensive part.
            .take(MAX_RESULTS)
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
    });

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
