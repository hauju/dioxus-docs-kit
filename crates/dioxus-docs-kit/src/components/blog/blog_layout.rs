use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdMenu, LdSearch};

use crate::blog::registry::BlogRegistry;
use crate::components::docs_layout::{CurrentTheme, DrawerOpen};

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

    let parent_search: Option<Signal<bool>> = try_use_context();
    let parent_drawer: Option<DrawerOpen> = try_use_context();

    let local_search = use_signal(|| false);
    let local_drawer = use_signal(|| false);

    let mut search_open = parent_search.unwrap_or(local_search);
    let mut drawer_open = parent_drawer.map(|d| d.0).unwrap_or(local_drawer);

    use_context_provider(|| search_open);
    use_context_provider(|| DrawerOpen(drawer_open));

    // Theme state
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

    // Keyboard shortcut: Cmd/Ctrl+K
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
                if (eval.recv::<bool>().await).is_ok() {
                    search_open.toggle();
                }
            }
        });
    });

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

/// Reusable search button for blog headers.
#[component]
pub fn BlogSearchButton(search_open: Signal<bool>) -> Element {
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
