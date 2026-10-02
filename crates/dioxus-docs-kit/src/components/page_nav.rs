use dioxus::prelude::*;

use crate::DocsContext;
use crate::registry::DocsRegistry;

/// Page navigation (previous/next).
///
/// Page order follows `_nav.json`. API endpoint pages are included in the
/// ordering only if the owning spec's nav group contains a page named
/// `<prefix>/overview` — endpoints are inserted right after it. Without an
/// overview page, endpoint pages render without prev/next links.
#[component]
pub fn DocsPageNav(current_path: String) -> Element {
    let registry = use_context::<&'static DocsRegistry>();
    let ctx = use_context::<DocsContext>();
    let (prev_page, next_page) = registry.page_neighbors(&current_path);

    rsx! {
        nav { class: "dk-pagination mt-16 pt-8 border-t border-base-300 flex justify-between gap-4",
            // Previous link
            div { class: "flex-1",
                if let Some(prev) = prev_page {
                    {
                        let title = registry.get_sidebar_title(&prev).unwrap_or_else(|| prev.clone());
                        let href = format!("{}/{}", ctx.base_path, prev);
                        rsx! {
                            Link {
                                to: NavigationTarget::Internal(href),
                                class: "dk-page-prev group flex flex-col p-4 rounded-lg border border-base-300 hover:border-primary/50 hover:bg-base-200/50 transition-all",
                                span { class: "text-xs text-base-content/50 mb-1", "Previous" }
                                span { class: "font-medium group-hover:text-primary transition-colors",
                                    "{title}"
                                }
                            }
                        }
                    }
                }
            }

            // Next link
            div { class: "flex-1 text-right",
                if let Some(next) = next_page {
                    {
                        let title = registry.get_sidebar_title(&next).unwrap_or_else(|| next.clone());
                        let href = format!("{}/{}", ctx.base_path, next);
                        rsx! {
                            Link {
                                to: NavigationTarget::Internal(href),
                                class: "dk-page-next group flex flex-col p-4 rounded-lg border border-base-300 hover:border-primary/50 hover:bg-base-200/50 transition-all items-end",
                                span { class: "text-xs text-base-content/50 mb-1", "Next" }
                                span { class: "font-medium group-hover:text-primary transition-colors",
                                    "{title}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
