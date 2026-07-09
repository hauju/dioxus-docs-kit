use dioxus::prelude::*;

use crate::blog::registry::BlogRegistry;
use crate::components::shared::ThemeToggleButton;

/// Light/dark theme toggle for blog layouts.
#[component]
pub fn BlogThemeToggle() -> Element {
    let registry = use_context::<&'static BlogRegistry>();

    rsx! {
        ThemeToggleButton { theme: registry.theme.clone() }
    }
}
