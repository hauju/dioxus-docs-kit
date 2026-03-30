use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdSearch, LdX};

use crate::BlogContext;
use crate::blog::registry::BlogRegistry;

/// Blog search modal triggered by Cmd/Ctrl+K or the search button.
#[component]
pub fn BlogSearchModal() -> Element {
    let mut search_open = use_context::<Signal<bool>>();
    let mut query = use_signal(String::new);
    let ctx = use_context::<BlogContext>();
    let registry = use_context::<&'static BlogRegistry>();

    let results = use_memo(move || registry.search_posts(&query()));

    let on_keydown = move |e: KeyboardEvent| {
        if e.key() == Key::Enter {
            let results = results.read();
            if let Some(entry) = results.first() {
                (ctx.navigate)(entry.slug.clone());
                search_open.set(false);
                query.set(String::new());
            }
        } else if e.key() == Key::Escape {
            search_open.set(false);
            query.set(String::new());
        }
    };

    if !search_open() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "fixed inset-0 z-[100] bg-black/50 flex items-start justify-center pt-[15vh]",
            onclick: move |_| {
                search_open.set(false);
                query.set(String::new());
            },

            div {
                class: "bg-base-200 rounded-xl w-full max-w-lg mx-4 border border-base-300 shadow-2xl overflow-hidden",
                onclick: move |e| e.stop_propagation(),

                div { class: "flex items-center gap-3 px-4 py-3 border-b border-base-300",
                    Icon { class: "size-5 text-base-content/50 shrink-0", icon: LdSearch }
                    input {
                        class: "flex-1 bg-transparent outline-none text-base placeholder:text-base-content/40",
                        placeholder: "Search posts...",
                        autofocus: true,
                        value: "{query}",
                        oninput: move |e| query.set(e.value()),
                        onkeydown: on_keydown,
                    }
                    button {
                        class: "btn btn-ghost btn-xs btn-square",
                        onclick: move |_| {
                            search_open.set(false);
                            query.set(String::new());
                        },
                        Icon { class: "size-4", icon: LdX }
                    }
                }

                div { class: "max-h-80 overflow-y-auto",
                    if query().trim().is_empty() {
                        div { class: "px-4 py-8 text-center text-base-content/50 text-sm",
                            "Type to search..."
                        }
                    } else if results.read().is_empty() {
                        div { class: "px-4 py-8 text-center text-base-content/50 text-sm",
                            "No results for \"{query}\""
                        }
                    } else {
                        for entry in results.read().iter() {
                            {
                                let slug = entry.slug.clone();
                                let title = entry.title.clone();
                                let date = entry.date.clone();
                                let tags = entry.tags.clone();
                                rsx! {
                                    BlogSearchResultItem {
                                        slug,
                                        title,
                                        date,
                                        tags,
                                        search_open,
                                        query,
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "px-4 py-2 border-t border-base-300 text-xs text-base-content/40 flex justify-between",
                    span { "Esc to close" }
                    span { "Enter to navigate" }
                }
            }
        }
    }
}

#[component]
fn BlogSearchResultItem(
    slug: String,
    title: String,
    date: String,
    tags: Vec<String>,
    mut search_open: Signal<bool>,
    mut query: Signal<String>,
) -> Element {
    let ctx = use_context::<BlogContext>();
    let slug_for_click = slug.clone();

    rsx! {
        button {
            class: "w-full text-left px-4 py-3 hover:bg-base-300/50 transition-colors flex items-center gap-3 border-b border-base-300/50 last:border-b-0",
            onclick: move |_| {
                (ctx.navigate)(slug_for_click.clone());
                search_open.set(false);
                query.set(String::new());
            },
            div { class: "flex-1 min-w-0",
                div { class: "flex items-center gap-2",
                    span { class: "font-medium text-sm truncate", "{title}" }
                }
                div { class: "flex items-center gap-2 mt-0.5",
                    span { class: "text-xs text-base-content/50", "{date}" }
                    for tag in tags.iter() {
                        span { class: "badge badge-xs badge-outline badge-primary", "{tag}" }
                    }
                }
            }
        }
    }
}
