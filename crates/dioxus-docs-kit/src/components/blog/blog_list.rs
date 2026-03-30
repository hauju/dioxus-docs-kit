use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdChevronLeft, LdChevronRight};

use crate::blog::registry::BlogRegistry;

use super::blog_card::BlogCard;
use super::tag_filter::TagFilter;

/// Blog listing page with cards grid, tag filter, and pagination.
#[component]
pub fn BlogList(hero: Option<Element>) -> Element {
    let registry = use_context::<&'static BlogRegistry>();
    let active_tag = use_context::<Signal<Option<String>>>();
    let mut current_page = use_context::<Signal<usize>>();

    let posts = use_memo(move || {
        let tag = active_tag();
        let page = current_page();
        match tag.as_deref() {
            Some(tag) => registry
                .posts_page_by_tag(tag, page)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>(),
            None => registry.posts_page(page).to_vec(),
        }
    });

    let total_pages = use_memo(move || {
        let tag = active_tag();
        match tag.as_deref() {
            Some(tag) => registry.total_pages_for_tag(tag),
            None => registry.total_pages(),
        }
    });

    rsx! {
        div { class: "max-w-6xl mx-auto px-4 py-12",
            if let Some(hero) = hero {
                {hero}
            }

            if !registry.all_tags().is_empty() {
                div { class: "mb-8",
                    TagFilter {}
                }
            }

            if posts.read().is_empty() {
                div { class: "text-center py-16 text-base-content/50",
                    p { class: "text-lg", "No posts found." }
                }
            } else {
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                    for post in posts.read().iter() {
                        BlogCard { key: "{post.slug}", post: post.clone() }
                    }
                }
            }

            if total_pages() > 1 {
                nav { class: "flex items-center justify-center gap-2 mt-12",
                    button {
                        class: "btn btn-ghost btn-sm",
                        disabled: current_page() == 0,
                        onclick: move |_| {
                            if current_page() > 0 {
                                current_page -= 1;
                            }
                        },
                        Icon { class: "size-4", icon: LdChevronLeft }
                        "Prev"
                    }
                    for page in 0..total_pages() {
                        {
                            let is_active = page == current_page();
                            let class = if is_active {
                                "btn btn-sm btn-primary"
                            } else {
                                "btn btn-sm btn-ghost"
                            };
                            rsx! {
                                button {
                                    class: "{class}",
                                    onclick: move |_| current_page.set(page),
                                    "{page + 1}"
                                }
                            }
                        }
                    }
                    button {
                        class: "btn btn-ghost btn-sm",
                        disabled: current_page() + 1 >= total_pages(),
                        onclick: move |_| {
                            if current_page() + 1 < total_pages() {
                                current_page += 1;
                            }
                        },
                        "Next"
                        Icon { class: "size-4", icon: LdChevronRight }
                    }
                }
            }
        }
    }
}
