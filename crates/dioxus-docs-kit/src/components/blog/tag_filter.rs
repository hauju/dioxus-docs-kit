use dioxus::prelude::*;

use crate::BlogContext;
use crate::blog::hooks::{ActiveTag, CurrentPage};
use crate::blog::registry::BlogRegistry;

/// Horizontal tag filter bar.
#[component]
pub fn TagFilter() -> Element {
    let registry = use_context::<&'static BlogRegistry>();
    let ActiveTag(mut active_tag) = use_context::<ActiveTag>();
    let CurrentPage(mut current_page) = use_context::<CurrentPage>();

    let ctx = use_context::<BlogContext>();
    let category_slug = ctx.current_category.map(|slug| slug()).unwrap_or_default();
    if registry.category_base_path().is_some() {
        return rsx! {
            nav { class: "flex flex-wrap gap-2", aria_label: "Blog categories",
                Link {
                    to: NavigationTarget::Internal(ctx.base_path.clone()),
                    class: if category_slug.is_empty() && (ctx.current_slug)().is_empty() { "btn btn-sm btn-primary" } else { "btn btn-sm btn-ghost" },
                    "All"
                }
                for category in registry.categories() {
                    if let Some(href) = registry.category_url(&category.slug, 1) {
                        Link {
                            to: NavigationTarget::Internal(href),
                            class: if category_slug == category.slug { "btn btn-sm btn-primary" } else { "btn btn-sm btn-ghost" },
                            aria_current: if category_slug == category.slug { Some("page") } else { None },
                            "{category.title}"
                            span { class: "ml-1 text-xs opacity-50", "{registry.tag_count(&category.tag)}" }
                        }
                    }
                }
            }
        };
    }

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
                            span { class: "ml-1 text-xs opacity-50", "{count}" }
                        }
                    }
                }
            }
        }
    }
}
