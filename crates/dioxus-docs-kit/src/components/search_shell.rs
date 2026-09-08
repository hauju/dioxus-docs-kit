//! Generic search modal shell shared by the docs and blog search modals.

use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdSearch, LdX};

use crate::search::SnippetSegment;

/// A single result row in the search modal.
#[derive(Clone, PartialEq)]
pub(crate) struct SearchHit {
    /// Value passed to `on_select` when this row (or Enter on the active row) is
    /// chosen. Docs section hits encode the anchor as `path#anchor`.
    pub target: String,
    /// Prominent text: the section heading for section hits, else the page/post title.
    pub title: String,
    /// Small context shown before the title (e.g. page title for a section hit).
    pub context: Option<String>,
    /// Optional `(label, badge classes)` rendered before the title (e.g. HTTP method).
    pub badge: Option<(&'static str, &'static str)>,
    /// Meta line rendered under the title (e.g. breadcrumb or date).
    pub meta: String,
    /// Optional tag badges rendered after the meta text.
    pub tags: Vec<String>,
    /// Highlighted snippet segments rendered under the meta (empty = none).
    pub snippet: Vec<SnippetSegment>,
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
    let mut selected = use_signal(|| 0usize);
    let dialog_id = use_hook(|| format!("dk-search-{}", dioxus::core::current_scope_id().0));
    let results_id = format!("{dialog_id}-results");

    let results = use_memo(move || search(query()));

    // Native modal dialogs contain focus, make the background inert, and
    // restore focus to the invoking control when closed. Keep the element
    // mounted so close() runs before any route change removes the layout.
    let effect_id = dialog_id.clone();
    use_effect(move || {
        let open = search_open();
        if !open {
            query.set(String::new());
            selected.set(0);
        }
        let id = serde_json::to_string(&effect_id).unwrap();
        let _ = document::eval(&format!(
            "const dialog = document.getElementById({id});\n\
             if (dialog) {{ if ({open} && !dialog.open) dialog.showModal();\n\
             else if (!{open} && dialog.open) dialog.close(); }}"
        ));
    });
    let cleanup_id = dialog_id.clone();
    use_drop(move || {
        let id = serde_json::to_string(&cleanup_id).unwrap();
        let _ = document::eval(&format!("document.getElementById({id})?.close();"));
    });

    // Keep the active descendant visible while focus stays in the input.
    let scroll_id = results_id.clone();
    use_effect(move || {
        let index = selected();
        if !search_open() || results.read().is_empty() {
            return;
        }
        let id = serde_json::to_string(&format!("{scroll_id}-{index}")).unwrap();
        let _ = document::eval(&format!(
            "document.getElementById({id})?.scrollIntoView({{block: 'nearest'}});"
        ));
    });

    let on_keydown = move |e: KeyboardEvent| {
        if e.is_composing() {
            return;
        }
        if matches!(e.key(), Key::ArrowDown | Key::ArrowUp) {
            e.prevent_default();
            let count = results.read().len();
            let next = next_selection(selected(), count, e.key() == Key::ArrowDown);
            selected.set(next);
        } else if e.key() == Key::Enter {
            e.prevent_default();
            let target = results.read().get(selected()).map(|hit| hit.target.clone());
            if let Some(target) = target {
                search_open.set(false);
                on_select(target);
            }
        }
    };

    let active_id = (!results.read().is_empty()).then(|| format!("{results_id}-{}", selected()));
    let display = if search_open() { "flex" } else { "none" };
    let focus_id = serde_json::to_string(&dialog_id).unwrap();

    rsx! {
        // Backdrop
        dialog {
            id: dialog_id,
            class: "dk-search-backdrop fixed inset-0 z-[100] bg-black/50 flex items-start justify-center pt-[15vh]",
            style: "display: {display}; width: 100vw; height: 100dvh; max-width: none; max-height: none; margin: 0; border: 0; padding-left: 0; padding-right: 0; color: inherit;",
            aria_label: placeholder,
            aria_modal: "true",
            onkeydown: move |e: KeyboardEvent| {
                if e.is_composing() {
                    return;
                }
                if e.key() == Key::Escape {
                    e.prevent_default();
                    e.stop_propagation();
                    search_open.set(false);
                } else if e.key() == Key::Tab {
                    // Explicit wrapping also avoids a stop in browser chrome
                    // after the last control in a native dialog.
                    e.prevent_default();
                    let backward = e.modifiers().shift();
                    let _ = document::eval(&format!(
                        "const dialog = document.getElementById({focus_id});\n\
                         const controls = [...dialog.querySelectorAll('input, button:not([tabindex=\"-1\"])')];\n\
                         const index = controls.indexOf(document.activeElement);\n\
                         const next = (index + ({backward} ? -1 : 1) + controls.length) % controls.length;\n\
                         controls[next]?.focus();"
                    ));
                }
            },
            oncancel: move |e| {
                e.prevent_default();
                search_open.set(false);
            },
            onclick: move |_| {
                search_open.set(false);
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
                        aria_label: placeholder,
                        role: "combobox",
                        aria_autocomplete: "list",
                        aria_expanded: search_open().to_string(),
                        aria_controls: results_id.clone(),
                        aria_activedescendant: active_id,
                        autofocus: true,
                        value: "{query}",
                        oninput: move |e| {
                            selected.set(0);
                            query.set(e.value());
                        },
                        onkeydown: on_keydown,
                    }
                    button {
                        class: "btn btn-ghost btn-xs btn-square",
                        r#type: "button",
                        aria_label: "Close search",
                        onclick: move |_| {
                            search_open.set(false);
                        },
                        Icon { class: "size-4", icon: LdX }
                    }
                }

                // Results list
                div {
                    class: "sr-only",
                    role: "status",
                    aria_live: "polite",
                    if !query().trim().is_empty() {
                        "{results.read().len()} search results"
                    }
                }
                div {
                    class: "dk-search-results max-h-80 overflow-y-auto",
                    id: results_id.clone(),
                    role: "listbox",
                    aria_label: "Search results",
                    if query().trim().is_empty() {
                        div { class: "px-4 py-8 text-center text-base-content/50 text-sm",
                            "Type to search..."
                        }
                    } else if results.read().is_empty() {
                        div { class: "px-4 py-8 text-center text-base-content/50 text-sm",
                            "No results for \"{query}\""
                        }
                    } else {
                        for (index, hit) in results.read().iter().enumerate() {
                            SearchResultRow {
                                key: "{hit.target}",
                                id: format!("{results_id}-{index}"),
                                hit: hit.clone(),
                                active: selected() == index,
                                on_select,
                                search_open,
                            }
                        }
                    }
                }

                // Footer
                div { class: "px-4 py-2 border-t border-base-300 text-xs text-base-content/40 flex justify-between",
                    span { "Esc to close" }
                    span { "↑↓ to select · Enter to navigate" }
                }
            }
        }
    }
}

#[component]
fn SearchResultRow(
    id: String,
    hit: SearchHit,
    active: bool,
    on_select: Callback<String>,
    mut search_open: Signal<bool>,
) -> Element {
    let target = hit.target.clone();

    rsx! {
        button {
            id,
            r#type: "button",
            role: "option",
            aria_selected: active.to_string(),
            tabindex: "-1",
            "data-active": active.to_string(),
            "data-target": hit.target.clone(),
            class: "dk-search-result w-full text-left px-4 py-3 hover:bg-base-300/50 transition-colors flex items-center gap-3 border-b border-base-300/50 last:border-b-0",
            onclick: move |_| {
                search_open.set(false);
                on_select(target.clone());
            },
            div { class: "flex-1 min-w-0",
                div { class: "flex items-center gap-2",
                    if let Some((label, badge_class)) = hit.badge {
                        span { class: "badge badge-xs font-mono {badge_class}", "{label}" }
                    }
                    if let Some(context) = hit.context.as_ref() {
                        span { class: "dk-search-context text-xs text-base-content/50 truncate shrink-0",
                            "{context}"
                        }
                        span { class: "text-xs text-base-content/30 shrink-0", "›" }
                    }
                    span { class: "font-medium text-sm truncate", "{hit.title}" }
                }
                div { class: "flex items-center gap-2 mt-0.5",
                    span { class: "text-xs text-base-content/50 truncate", "{hit.meta}" }
                    for tag in hit.tags.iter() {
                        span { class: "badge badge-xs badge-outline badge-primary", "{tag}" }
                    }
                }
                if !hit.snippet.is_empty() {
                    div { class: "dk-search-snippet text-xs text-base-content/60 mt-1 line-clamp-2",
                        for (i, seg) in hit.snippet.iter().enumerate() {
                            if seg.highlight {
                                mark {
                                    key: "{i}",
                                    class: "dk-search-mark bg-primary/20 text-primary rounded-sm px-0.5",
                                    "{seg.text}"
                                }
                            } else {
                                span { key: "{i}", "{seg.text}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn next_selection(current: usize, count: usize, forward: bool) -> usize {
    if count == 0 {
        return 0;
    }
    let current = current.min(count - 1);
    if forward {
        (current + 1) % count
    } else {
        (current + count - 1) % count
    }
}

#[cfg(test)]
mod tests {
    use super::next_selection;

    #[test]
    fn keyboard_selection_wraps_and_handles_empty_or_shortened_results() {
        assert_eq!(next_selection(0, 3, true), 1);
        assert_eq!(next_selection(2, 3, true), 0);
        assert_eq!(next_selection(0, 3, false), 2);
        assert_eq!(next_selection(0, 0, false), 0);
        assert_eq!(next_selection(4, 1, true), 0);
        assert_eq!(next_selection(4, 2, false), 0);
    }
}
