use dioxus::prelude::*;
use dioxus_mdx::HttpMethod;

use super::search_shell::{SearchHit, SearchModalShell};
use crate::DocsContext;
use crate::registry::DocsRegistry;

/// Full-screen search modal triggered by Cmd/Ctrl+K or the search button.
#[component]
pub fn SearchModal() -> Element {
    let ctx = use_context::<DocsContext>();
    let registry = use_context::<&'static DocsRegistry>();

    let search = use_callback(move |query: String| {
        registry
            .search_docs(&query)
            .into_iter()
            .map(|entry| SearchHit {
                target: entry.path.clone(),
                title: entry.title.clone(),
                badge: entry.api_method.map(method_badge),
                meta: entry.breadcrumb.clone(),
                tags: Vec::new(),
            })
            .collect()
    });

    let on_select = use_callback(move |path: String| (ctx.navigate)(path));

    rsx! {
        SearchModalShell {
            placeholder: "Search documentation...",
            search,
            on_select,
        }
    }
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
