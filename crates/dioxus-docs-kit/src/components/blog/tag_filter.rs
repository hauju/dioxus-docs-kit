use dioxus::prelude::*;

use crate::blog::registry::BlogRegistry;

/// Horizontal tag filter bar.
#[component]
pub fn TagFilter() -> Element {
    let registry = use_context::<&'static BlogRegistry>();
    let mut active_tag = use_context::<Signal<Option<String>>>();
    let mut current_page = use_context::<Signal<usize>>();

    rsx! {
        div { class: "flex flex-wrap gap-2",
            {
                let is_active = active_tag().is_none();
                let class = if is_active {
                    "btn btn-sm btn-primary"
                } else {
                    "btn btn-sm btn-ghost"
                };
                rsx! {
                    button {
                        class: "{class}",
                        onclick: move |_| {
                            active_tag.set(None);
                            current_page.set(0);
                        },
                        "All"
                    }
                }
            }
            for tag in registry.all_tags().iter() {
                {
                    let is_active = active_tag().as_deref() == Some(tag.as_str());
                    let count = registry.tag_count(tag);
                    let class = if is_active {
                        "btn btn-sm btn-primary"
                    } else {
                        "btn btn-sm btn-ghost"
                    };
                    let tag_clone = tag.clone();
                    rsx! {
                        button {
                            class: "{class}",
                            onclick: move |_| {
                                if is_active {
                                    active_tag.set(None);
                                } else {
                                    active_tag.set(Some(tag_clone.clone()));
                                }
                                current_page.set(0);
                            },
                            "{tag}"
                            span { class: "badge badge-xs ml-1", "{count}" }
                        }
                    }
                }
            }
        }
    }
}
