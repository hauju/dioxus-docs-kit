use dioxus::prelude::*;

use crate::DocsContext;
use crate::registry::DocsRegistry;

/// Page navigation (previous/next).
///
/// Page order follows `_nav.json`. API endpoint and Rust API item pages are
/// included in the ordering only if the owning spec's/model's nav group
/// contains a page named `<prefix>/overview` — dynamic pages are inserted
/// right after it. Without an overview page, they render without prev/next
/// links.
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

    let mut all_pages: Vec<String> = Vec::new();
    for group in &tab_groups {
        for page in &group.pages {
            all_pages.push(page.clone());
            // Insert a spec's endpoint pages right after its "<prefix>/overview"
            // page, so endpoints participate in prev/next ordering.
            if let Some(prefix) = page.strip_suffix("/overview") {
                if let Some(spec) = registry.get_api_spec(prefix) {
                    all_pages.extend(
                        spec.operations
                            .iter()
                            .map(|op| format!("{prefix}/{}", op.slug())),
                    );
                }
                // Same for a Rust API model's item pages, in sidebar order.
                all_pages.extend(
                    registry
                        .get_rust_sidebar_entries()
                        .iter()
                        .flat_map(|(_, entries)| entries.iter())
                        .filter(|e| e.prefix == prefix)
                        .map(|e| format!("{prefix}/{}", e.slug)),
                );
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
