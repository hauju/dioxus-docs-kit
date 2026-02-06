use dioxus::prelude::*;

use crate::DocsContext;
use crate::registry::DocsRegistry;

/// Page navigation (previous/next).
#[component]
pub fn DocsPageNav(current_path: String) -> Element {
    let registry = use_context::<&'static DocsRegistry>();
    let ctx = use_context::<DocsContext>();
    let nav = &registry.nav;

    // Determine which tab the current page belongs to
    let current_tab = registry.tab_for_path(&current_path);

    // Build page list scoped to the current tab
    let tab_groups: Vec<_> = if let Some(ref tab) = current_tab {
        nav.groups_for_tab(tab)
    } else {
        nav.groups.iter().collect()
    };

    let api_prefix = registry.get_first_api_prefix();
    let overview_path = api_prefix.map(|p| format!("{p}/overview"));

    let mut all_pages: Vec<String> = Vec::new();
    for group in &tab_groups {
        for page in &group.pages {
            all_pages.push(page.clone());
            // Insert API endpoint pages right after overview
            if let Some(ref ov) = overview_path {
                if page == ov {
                    all_pages.extend(registry.get_api_endpoint_paths());
                }
            }
        }
    }

    let current_index = all_pages.iter().position(|p| *p == current_path);

    let prev_page = current_index.and_then(|i| {
        if i > 0 {
            Some(all_pages[i - 1].clone())
        } else {
            None
        }
    });

    let next_page = current_index.and_then(|i| {
        if i + 1 < all_pages.len() {
            Some(all_pages[i + 1].clone())
        } else {
            None
        }
    });

    rsx! {
        nav { class: "mt-16 pt-8 border-t border-base-300 flex justify-between gap-4",
            // Previous link
            div { class: "flex-1",
                if let Some(prev) = prev_page {
                    {
                        let title = registry.get_sidebar_title(&prev).unwrap_or_else(|| prev.clone());
                        let href = format!("{}/{}", ctx.base_path, prev);
                        rsx! {
                            Link {
                                to: NavigationTarget::Internal(href),
                                class: "group flex flex-col p-4 rounded-lg border border-base-300 hover:border-primary/50 hover:bg-base-200/50 transition-all",
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
                                class: "group flex flex-col p-4 rounded-lg border border-base-300 hover:border-primary/50 hover:bg-base-200/50 transition-all items-end",
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
