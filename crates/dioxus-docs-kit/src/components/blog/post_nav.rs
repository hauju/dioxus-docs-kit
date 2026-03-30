use dioxus::prelude::*;

use crate::BlogContext;
use crate::blog::registry::BlogRegistry;

/// Previous/Next post navigation.
#[component]
pub fn BlogPostNav(current_slug: String) -> Element {
    let registry = use_context::<&'static BlogRegistry>();
    let ctx = use_context::<BlogContext>();

    let prev = registry.prev_post(&current_slug);
    let next = registry.next_post(&current_slug);

    rsx! {
        nav { class: "mt-16 pt-8 border-t border-base-300 flex justify-between gap-4",
            div { class: "flex-1",
                if let Some(prev) = prev {
                    {
                        let href = format!("{}/{}", ctx.base_path, prev.slug);
                        rsx! {
                            Link {
                                to: NavigationTarget::Internal(href),
                                class: "group flex flex-col p-4 rounded-lg border border-base-300 hover:border-primary/50 hover:bg-base-200/50 transition-all",
                                span { class: "text-xs text-base-content/50 mb-1", "Older" }
                                span { class: "font-medium group-hover:text-primary transition-colors",
                                    "{prev.frontmatter.title}"
                                }
                            }
                        }
                    }
                }
            }

            div { class: "flex-1 text-right",
                if let Some(next) = next {
                    {
                        let href = format!("{}/{}", ctx.base_path, next.slug);
                        rsx! {
                            Link {
                                to: NavigationTarget::Internal(href),
                                class: "group flex flex-col p-4 rounded-lg border border-base-300 hover:border-primary/50 hover:bg-base-200/50 transition-all items-end",
                                span { class: "text-xs text-base-content/50 mb-1", "Newer" }
                                span { class: "font-medium group-hover:text-primary transition-colors",
                                    "{next.frontmatter.title}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
