use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::LdX;

use crate::BlogContext;
use crate::blog::registry::BlogRegistry;

use super::tag_filter::TagFilter;

/// Slide-in drawer for mobile screens (blog variant).
#[component]
pub fn BlogMobileDrawer(mut open: Signal<bool>) -> Element {
    let ctx = use_context::<BlogContext>();
    let registry = use_context::<&'static BlogRegistry>();

    let current_slug = ctx.current_slug;
    use_effect(move || {
        let _ = current_slug();
        open.set(false);
    });

    let is_open = open();

    let backdrop_class = if is_open {
        "fixed inset-0 z-[60] bg-black/50 lg:hidden transition-opacity duration-300 opacity-100"
    } else {
        "fixed inset-0 z-[60] bg-black/50 lg:hidden transition-opacity duration-300 opacity-0 pointer-events-none"
    };

    let panel_class = if is_open {
        "fixed left-0 top-0 bottom-0 z-[70] w-72 bg-base-200 lg:hidden transition-transform duration-300 translate-x-0 shadow-2xl"
    } else {
        "fixed left-0 top-0 bottom-0 z-[70] w-72 bg-base-200 lg:hidden transition-transform duration-300 -translate-x-full shadow-2xl"
    };

    rsx! {
        div {
            class: "{backdrop_class}",
            onclick: move |_| open.set(false),
        }

        div {
            class: "{panel_class}",
            onclick: move |e| e.stop_propagation(),

            div { class: "flex items-center justify-between px-4 py-3 border-b border-base-300",
                span { class: "font-semibold text-sm", "Blog" }
                button {
                    class: "btn btn-ghost btn-xs btn-square",
                    onclick: move |_| open.set(false),
                    Icon { class: "size-4", icon: LdX }
                }
            }

            div { class: "overflow-y-auto h-[calc(100%-3.5rem)] p-4",
                if !registry.all_tags().is_empty() {
                    div { class: "mb-6",
                        h3 { class: "font-semibold text-sm text-base-content/70 uppercase tracking-wider mb-3",
                            "Tags"
                        }
                        TagFilter {}
                    }
                }

                div {
                    h3 { class: "font-semibold text-sm text-base-content/70 uppercase tracking-wider mb-3",
                        "Recent Posts"
                    }
                    ul { class: "space-y-1",
                        for post in registry.all_posts().iter().take(10) {
                            {
                                let href = format!("{}/{}", ctx.base_path, post.slug);
                                let is_active = current_slug() == post.slug;
                                let active_class = if is_active {
                                    "bg-primary/10 text-primary font-medium"
                                } else {
                                    "text-base-content/70 hover:text-base-content hover:bg-base-200"
                                };
                                rsx! {
                                    li {
                                        Link {
                                            to: NavigationTarget::Internal(href),
                                            class: "block px-3 py-2 text-sm rounded-lg transition-colors {active_class}",
                                            "{post.frontmatter.title}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
