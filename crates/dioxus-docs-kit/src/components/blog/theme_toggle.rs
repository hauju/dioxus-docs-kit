use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdMoon, LdSun};

use crate::blog::registry::BlogRegistry;
use crate::components::docs_layout::CurrentTheme;

/// Light/dark theme toggle for blog layouts.
#[component]
pub fn BlogThemeToggle() -> Element {
    let registry = use_context::<&'static BlogRegistry>();

    let toggle = match registry
        .theme
        .as_ref()
        .and_then(|t| t.toggle_themes.as_ref())
    {
        Some(t) => t.clone(),
        None => return rsx! {},
    };

    let storage_key = registry
        .theme
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
