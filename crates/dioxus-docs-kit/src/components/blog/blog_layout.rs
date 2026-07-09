use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::LdMenu;

use crate::blog::registry::BlogRegistry;
use crate::components::docs_layout::{DrawerOpen, SearchOpen};

use super::mobile_drawer::BlogMobileDrawer;
use super::search_modal::BlogSearchModal;
use super::theme_toggle::BlogThemeToggle;

/// Blog layout shell.
///
/// # Context requirements
///
/// - `&'static BlogRegistry` — provided by consumer
/// - `BlogContext` — provided by consumer
#[component]
pub fn BlogLayout(
    header: Option<Element>,
    #[props(default = true)] show_header: bool,
    children: Element,
) -> Element {
    let registry = use_context::<&'static BlogRegistry>();

    let parent_search: Option<SearchOpen> = try_use_context();
    let parent_drawer: Option<DrawerOpen> = try_use_context();

    let local_search = use_signal(|| false);
    let local_drawer = use_signal(|| false);

    let search_open = parent_search.map(|s| s.0).unwrap_or(local_search);
    let mut drawer_open = parent_drawer.map(|d| d.0).unwrap_or(local_drawer);

    use_context_provider(|| SearchOpen(search_open));
    use_context_provider(|| DrawerOpen(drawer_open));

    // Theme state
    crate::components::shared::use_theme_provider(registry.theme.clone());

    // Keyboard shortcut: Cmd/Ctrl+K
    crate::components::shared::use_search_hotkey(search_open);

    rsx! {
        div { class: "min-h-screen bg-base-100",
            if show_header {
                div { class: "sticky top-0 z-50",
                    if let Some(hdr) = header {
                        {hdr}
                    } else {
                        div { class: "navbar bg-base-200 border-b border-base-300 px-4 lg:px-8",
                            div { class: "flex-1 gap-2",
                                button {
                                    class: "btn btn-ghost btn-sm btn-square lg:hidden",
                                    onclick: move |_| drawer_open.toggle(),
                                    Icon { class: "size-5", icon: LdMenu }
                                }
                            }
                            div { class: "flex-none gap-1",
                                BlogSearchButton { search_open }
                                BlogThemeToggle {}
                            }
                        }
                    }
                }
            }

            div { class: "flex-1 min-w-0",
                {children}
            }
        }

        BlogMobileDrawer { open: drawer_open }
        BlogSearchModal {}
    }
}

/// Reusable search button for blog headers (same component as
/// [`SearchButton`](crate::components::SearchButton), re-exported under the
/// blog naming convention).
pub use crate::components::docs_layout::SearchButton as BlogSearchButton;
