use dioxus::prelude::*;

use super::shared::ThemeToggleButton;
use crate::registry::DocsRegistry;

/// Light/dark theme toggle button.
///
/// Reads theme configuration from `DocsRegistry` context and current theme from
/// the [`CurrentTheme`](super::docs_layout::CurrentTheme) context (provided by
/// `DocsLayout` or [`use_theme_provider`](super::shared::use_theme_provider)).
///
/// Renders nothing if the registry has no `toggle_themes` configured.
#[component]
pub fn ThemeToggle() -> Element {
    let registry = use_context::<&'static DocsRegistry>();

    rsx! {
        ThemeToggleButton { theme: registry.theme.clone() }
    }
}
