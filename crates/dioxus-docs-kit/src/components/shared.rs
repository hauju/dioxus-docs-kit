//! Hooks and components shared between the docs and blog surfaces.

use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdMoon, LdSun};

use super::docs_layout::CurrentTheme;
use crate::config::ThemeConfig;

/// Provide the [`CurrentTheme`] context and apply the persisted theme on mount.
///
/// Reads the stored preference from localStorage (falling back to the config's
/// default theme), sets `data-theme` on `<html>`, and keeps the returned signal
/// in sync. Called by `DocsLayout`/`BlogLayout`; call it yourself when you
/// render kit components (e.g. [`ThemeToggle`](super::ThemeToggle)) outside
/// those layouts, e.g. in a landing-page navbar.
pub fn use_theme_provider(theme: Option<ThemeConfig>) -> Signal<String> {
    let theme_default = theme
        .as_ref()
        .map(|t| t.default_theme.clone())
        .unwrap_or_default();
    let storage_key = theme
        .as_ref()
        .map(|t| t.storage_key.clone())
        .unwrap_or_default();
    let has_theme = theme.is_some();

    let mut current_theme = use_signal(|| theme_default.clone());
    use_context_provider(|| CurrentTheme(current_theme));

    // On mount: read stored preference and apply data-theme
    use_effect(move || {
        if !has_theme {
            return;
        }
        let key = storage_key.clone();
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

    current_theme
}

/// Register a document-level Cmd/Ctrl+K listener that toggles `search_open`.
///
/// The handler is stored on `window` and any previous one is removed before
/// registering, so layout remounts never accumulate listeners.
pub(crate) fn use_search_hotkey(mut search_open: Signal<bool>) {
    use_effect(move || {
        spawn(async move {
            let mut eval = document::eval(
                r#"
                if (window.__dkSearchHotkey) {
                    document.removeEventListener('keydown', window.__dkSearchHotkey);
                }
                window.__dkSearchHotkey = (e) => {
                    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
                        e.preventDefault();
                        dioxus.send(true);
                    }
                };
                document.addEventListener('keydown', window.__dkSearchHotkey);
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
}

/// Light/dark theme toggle button shared by [`ThemeToggle`](super::ThemeToggle)
/// and [`BlogThemeToggle`](super::blog::BlogThemeToggle).
///
/// Renders nothing if `theme` has no `toggle_themes` configured.
#[component]
pub(crate) fn ThemeToggleButton(theme: Option<ThemeConfig>) -> Element {
    let toggle = match theme.as_ref().and_then(|t| t.toggle_themes.as_ref()) {
        Some(t) => t.clone(),
        None => return rsx! {},
    };

    let storage_key = theme
        .as_ref()
        .map(|t| t.storage_key.clone())
        .unwrap_or_default();

    let CurrentTheme(mut current_theme) = use_context::<CurrentTheme>();

    let (light, dark) = toggle;
    let is_dark = current_theme() == dark;

    rsx! {
        button {
            class: "btn btn-ghost btn-sm btn-square",
            title: if is_dark { "Switch to light mode" } else { "Switch to dark mode" },
            onclick: move |_| {
                let new_theme = if (current_theme)() == dark { light.clone() } else { dark.clone() };
                current_theme.set(new_theme.clone());
                let key = storage_key.clone();
                spawn(async move {
                    let _ = document::eval(&format!(
                        r#"document.documentElement.setAttribute('data-theme', '{new_theme}');
                        try {{ localStorage.setItem('{key}', '{new_theme}'); }} catch(e) {{}}"#
                    ));
                });
            },
            if is_dark {
                Icon { class: "size-5", icon: LdSun }
            } else {
                Icon { class: "size-5", icon: LdMoon }
            }
        }
    }
}
