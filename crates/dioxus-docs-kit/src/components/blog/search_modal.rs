use dioxus::prelude::*;

use crate::BlogContext;
use crate::blog::registry::BlogRegistry;
use crate::components::search_shell::{SearchHit, SearchModalShell};

/// Blog search modal triggered by Cmd/Ctrl+K or the search button.
#[component]
pub fn BlogSearchModal() -> Element {
    let ctx = use_context::<BlogContext>();
    let registry = use_context::<&'static BlogRegistry>();

    let search = use_callback(move |query: String| {
        registry
            .search_posts(&query)
            .into_iter()
            .map(|entry| SearchHit {
                target: entry.slug.clone(),
                title: entry.title.clone(),
                badge: None,
                meta: entry.date.clone(),
                tags: entry.tags.clone(),
            })
            .collect()
    });

    let on_select = use_callback(move |slug: String| (ctx.navigate)(slug));

    rsx! {
        SearchModalShell {
            placeholder: "Search posts...",
            search,
            on_select,
        }
    }
}
