use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::LdMenu;

use crate::DocsContext;
use crate::registry::DocsRegistry;

/// Layout offset values computed by `DocsLayout` and consumed by child components
/// (e.g. `DocsPageContent`) via context.
///
/// The values are determined by `show_header` and whether tabs exist, so that the
/// TOC sidebar and heading scroll targets align with the actual header height.
#[derive(Clone, Debug)]
pub struct LayoutOffsets {
    /// Tailwind sticky top class for sidebars/TOC (e.g. `"top-[6.5rem]"`, `"top-16"`, `"top-0"`).
    pub sticky_top: &'static str,
    /// Tailwind scroll-margin-top class for heading anchors (e.g. `"scroll-mt-[6.5rem]"`).
    pub scroll_mt: &'static str,
    /// Tailwind height calc for sidebar (e.g. `"h-[calc(100vh-6.5rem)]"`).
    pub sidebar_height: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct CurrentTheme(pub Signal<String>);

/// Newtype wrapper for the drawer-open signal, so it can coexist with
/// `Signal<bool>` (used for `search_open`) in the context system.
///
/// Consumers can provide this before rendering `DocsLayout` to control
/// the mobile drawer from a custom header.
#[derive(Clone, Copy)]
pub struct DrawerOpen(pub Signal<bool>);

use super::mobile_drawer::MobileDrawer;
use super::search_modal::SearchModal;
use super::sidebar::DocsSidebar;
use super::theme_toggle::ThemeToggle;

/// Documentation layout shell.
///
/// Renders the tab bar, sidebar, content area, search modal, and mobile drawer.
/// The consumer wraps this in their own navbar via Dioxus route layouts.
///
/// # Context requirements
///
/// - `&'static DocsRegistry` — provided by consumer
/// - `DocsContext` — provided by consumer
///
/// # Props
///
/// - `header`: Optional element to render inside the docs area header (e.g. branding + search button).
///   If not provided, a default header with search button and hamburger is rendered.
/// - `show_header`: Whether to render the internal header area (default header/custom header *and*
///   tab bar). Defaults to `true`. Set to `false` when the consumer provides their own header
///   and tab bar outside of `DocsLayout`.
/// - `children`: The routed page content (from `Outlet` or explicit child).
#[component]
pub fn DocsLayout(
    header: Option<Element>,
    #[props(default = true)] show_header: bool,
    children: Element,
) -> Element {
    let registry = use_context::<&'static DocsRegistry>();
    let ctx = use_context::<DocsContext>();
    let nav = &registry.nav;

    // Check if consumer already provided context (lookups, not hooks)
    let parent_search: Option<Signal<bool>> = try_use_context();
    let parent_drawer: Option<DrawerOpen> = try_use_context();

    // Always create local fallback signals unconditionally
    let local_search = use_signal(|| false);
    let local_drawer = use_signal(|| false);

    // Use consumer-provided context if available, otherwise local
    let mut search_open = parent_search.unwrap_or(local_search);
    let mut drawer_open = parent_drawer.map(|d| d.0).unwrap_or(local_drawer);

    // Always provide context for children (SearchModal, MobileDrawer, etc.)
    use_context_provider(|| search_open);
    use_context_provider(|| DrawerOpen(drawer_open));

    // Theme state: hooks must be called unconditionally
    let theme_default = registry
        .theme
        .as_ref()
        .map(|t| t.default_theme.clone())
        .unwrap_or_default();
    let theme_storage_key = registry
        .theme
        .as_ref()
        .map(|t| t.storage_key.clone())
        .unwrap_or_default();
    let has_theme = registry.theme.is_some();

    let mut current_theme = use_signal(|| theme_default.clone());
    use_context_provider(|| CurrentTheme(current_theme));

    // On mount: read stored preference and apply data-theme
    use_effect(move || {
        if !has_theme {
            return;
        }
        let key = theme_storage_key.clone();
        let fallback = theme_default.clone();
        spawn(async move {
            let mut eval = document::eval(&format!(
                r#"
                let theme = null;
                try {{ theme = localStorage.getItem('{key}'); }} catch(e) {{}}
                theme = theme || '{fallback}';
                document.documentElement.setAttribute('data-theme', theme);
                dioxus.send(theme);
                "#
            ));
            if let Ok(stored) = eval.recv::<String>().await {
                current_theme.set(stored);
            }
        });
    });

    // Active tab state
    let mut active_tab = use_signal(|| nav.tabs.first().cloned().unwrap_or_default());
    use_context_provider(|| active_tab);

    // Sync active tab from current path
    let current_path = ctx.current_path;
    let registry_for_effect = registry;
    use_effect(move || {
        let path = current_path();
        if let Some(tab) = registry_for_effect.tab_for_path(&path) {
            active_tab.set(tab);
        }
    });

    // Keyboard shortcut: Cmd/Ctrl+K to toggle search
    use_effect(move || {
        spawn(async move {
            let mut eval = document::eval(
                r#"
                document.addEventListener('keydown', (e) => {
                    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
                        e.preventDefault();
                        dioxus.send(true);
                    }
                });
                while (true) { await new Promise(r => setTimeout(r, 1000000)); }
                "#,
            );
            loop {
                if let Ok(_) = eval.recv::<bool>().await {
                    search_open.toggle();
                }
            }
        });
    });

    let has_tabs = nav.has_tabs();
    let offsets = if !show_header {
        LayoutOffsets {
            sticky_top: "top-0",
            scroll_mt: "scroll-mt-0",
            sidebar_height: "h-screen",
        }
    } else if has_tabs {
        LayoutOffsets {
            sticky_top: "top-[6.5rem]",
            scroll_mt: "scroll-mt-[6.5rem]",
            sidebar_height: "h-[calc(100vh-6.5rem)]",
        }
    } else {
        LayoutOffsets {
            sticky_top: "top-16",
            scroll_mt: "scroll-mt-16",
            sidebar_height: "h-[calc(100vh-4rem)]",
        }
    };
    use_context_provider(|| offsets.clone());

    rsx! {
        div { class: "min-h-screen bg-base-100",
            // Top area
            if show_header {
                div { class: "sticky top-0 z-50",
                    // Header (consumer-provided or default)
                    if let Some(hdr) = header {
                        {hdr}
                    } else {
                        // Default minimal header
                        div { class: "navbar bg-base-200 border-b border-base-300 px-4 lg:px-8",
                            div { class: "flex-1 gap-2",
                                button {
                                    class: "btn btn-ghost btn-sm btn-square lg:hidden",
                                    onclick: move |_| drawer_open.toggle(),
                                    Icon { class: "size-5", icon: LdMenu }
                                }
                            }
                            div { class: "flex-none gap-1",
                                SearchButton { search_open }
                                ThemeToggle {}
                            }
                        }
                    }

                    // Tab bar (below header)
                    if has_tabs {
                        div { class: "bg-base-200/80 backdrop-blur border-b border-base-300 px-4 lg:px-8",
                            div { class: "flex items-center justify-between",
                                div { class: "flex gap-6",
                                    for tab in nav.tabs.iter() {
                                        {
                                            let is_active = *tab == active_tab();
                                            let tab_clone = tab.clone();
                                            let style = if is_active {
                                                "text-primary border-b-2 border-primary font-medium"
                                            } else {
                                                "text-base-content/60 hover:text-base-content border-b-2 border-transparent"
                                            };
                                            rsx! {
                                                button {
                                                    class: "px-1 py-2.5 text-sm transition-colors -mb-px {style}",
                                                    onclick: move |_| {
                                                        active_tab.set(tab_clone.clone());
                                                        let groups = nav.groups_for_tab(&tab_clone);
                                                        if let Some(first_page) = groups.first().and_then(|g| g.pages.first()) {
                                                            (ctx.navigate)(first_page.clone());
                                                        }
                                                    },
                                                    "{tab}"
                                                }
                                            }
                                        }
                                    }
                                }
                                div { class: "flex items-center gap-1",
                                    SearchButton { search_open }
                                    if has_theme {
                                        ThemeToggle {}
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Main docs content with sidebar
            div { class: "flex",
                // Sidebar
                aside { class: "w-64 shrink-0 border-r border-base-300 bg-base-200/30 hidden lg:block",
                    div { class: "sticky {offsets.sticky_top} {offsets.sidebar_height} overflow-y-auto p-6",
                        DocsSidebar {}
                    }
                }

                // Main content area
                div { class: "flex-1 min-w-0",
                    {children}
                }
            }
        }

        // Overlays
        MobileDrawer { open: drawer_open }
        SearchModal {}
    }
}

/// Reusable search button component for headers.
#[component]
pub fn SearchButton(search_open: Signal<bool>) -> Element {
    use dioxus_free_icons::icons::ld_icons::LdSearch;

    rsx! {
        button {
            class: "btn btn-ghost btn-sm gap-2",
            onclick: move |_| search_open.set(true),
            Icon { class: "size-4", icon: LdSearch }
            span { class: "hidden sm:inline text-base-content/60 text-sm", "Search" }
            kbd { class: "kbd kbd-xs hidden sm:inline-flex", "\u{2318}K" }
        }
    }
}
