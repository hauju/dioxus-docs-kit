//! Generic search modal shell shared by the docs and blog search modals.

use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdSearch, LdX};

/// A single result row in the search modal.
#[derive(Clone, PartialEq)]
pub(crate) struct SearchHit {
    /// Value passed to `on_select` when this row (or Enter on the first row) is chosen.
    pub target: String,
    pub title: String,
    /// Optional `(label, badge classes)` rendered before the title (e.g. HTTP method).
    pub badge: Option<(&'static str, &'static str)>,
    /// Meta line rendered under the title (e.g. breadcrumb or date).
    pub meta: String,
    /// Optional tag badges rendered after the meta text.
    pub tags: Vec<String>,
}

/// Modal shell: backdrop, input row, result list, and footer.
///
/// Reads the open state from the `Signal<bool>` context provided by the layout.
/// Emits the stable `dk-search-*` classes for consumer CSS hooks.
#[component]
pub(crate) fn SearchModalShell(
    placeholder: &'static str,
    search: Callback<String, Vec<SearchHit>>,
    on_select: Callback<String>,
) -> Element {
    let super::docs_layout::SearchOpen(mut search_open) = use_context();
    let mut query = use_signal(String::new);

    let results = use_memo(move || search(query()));

    let on_keydown = move |e: KeyboardEvent| {
        if e.key() == Key::Enter {
            let results = results.read();
            if let Some(hit) = results.first() {
                on_select(hit.target.clone());
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
        // Backdrop
        div {
            class: "dk-search-backdrop fixed inset-0 z-[100] bg-black/50 flex items-start justify-center pt-[15vh]",
            onclick: move |_| {
                search_open.set(false);
                query.set(String::new());
            },

            // Modal container
            div {
                class: "dk-search-dialog bg-base-200 rounded-xl w-full max-w-lg mx-4 border border-base-300 shadow-2xl overflow-hidden",
                onclick: move |e| e.stop_propagation(),

                // Search input row
                div { class: "flex items-center gap-3 px-4 py-3 border-b border-base-300",
                    Icon { class: "size-5 text-base-content/50 shrink-0", icon: LdSearch }
                    input {
                        class: "dk-search-input flex-1 bg-transparent outline-none text-base placeholder:text-base-content/40",
                        placeholder,
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

                // Results list
                div { class: "dk-search-results max-h-80 overflow-y-auto",
                    if query().trim().is_empty() {
                        div { class: "px-4 py-8 text-center text-base-content/50 text-sm",
                            "Type to search..."
                        }
                    } else if results.read().is_empty() {
                        div { class: "px-4 py-8 text-center text-base-content/50 text-sm",
                            "No results for \"{query}\""
                        }
                    } else {
                        for hit in results.read().iter() {
                            SearchResultRow {
                                hit: hit.clone(),
                                on_select,
                                search_open,
                                query,
                            }
                        }
                    }
                }

                // Footer
                div { class: "px-4 py-2 border-t border-base-300 text-xs text-base-content/40 flex justify-between",
                    span { "Esc to close" }
                    span { "Enter to navigate" }
                }
            }
        }
    }
}

#[component]
fn SearchResultRow(
    hit: SearchHit,
    on_select: Callback<String>,
    mut search_open: Signal<bool>,
    mut query: Signal<String>,
) -> Element {
    let target = hit.target.clone();

    rsx! {
        button {
            class: "dk-search-result w-full text-left px-4 py-3 hover:bg-base-300/50 transition-colors flex items-center gap-3 border-b border-base-300/50 last:border-b-0",
            onclick: move |_| {
                on_select(target.clone());
                search_open.set(false);
                query.set(String::new());
            },
            div { class: "flex-1 min-w-0",
                div { class: "flex items-center gap-2",
                    if let Some((label, badge_class)) = hit.badge {
                        span { class: "badge badge-xs font-mono {badge_class}", "{label}" }
                    }
                    span { class: "font-medium text-sm truncate", "{hit.title}" }
                }
                div { class: "flex items-center gap-2 mt-0.5",
                    span { class: "text-xs text-base-content/50 truncate", "{hit.meta}" }
                    for tag in hit.tags.iter() {
                        span { class: "badge badge-xs badge-outline badge-primary", "{tag}" }
                    }
                }
            }
        }
    }
}
