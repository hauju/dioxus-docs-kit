//! Light/dark theme switching without a runtime: a tiny head script applies the
//! stored theme before first paint, and the header button flips it.
//!
//! Both set `data-theme` (the DaisyUI theme) and `data-dk-mode` (`light` or
//! `dark`) on `<html>`; the button's icons key off the latter, so the server
//! never needs to know which theme the visitor picked.

use dioxus_mdx::lucide::{LdMoon, LdSun};
use docs_kit_core::DocsRegistry;
use topcoat::{
    Result,
    view::{Unescaped, View, component, view},
};

use crate::icon::icon;

/// Reads its settings from the script tag's own `data-*` attributes, so no
/// value is ever spliced into JavaScript source.
const APPLY_STORED_THEME: &str = "(function(){var s=document.currentScript.dataset,d=document.documentElement,t=null;try{t=localStorage.getItem(s.key)}catch(e){}t=t||s.default;d.setAttribute('data-theme',t);if(s.dark)d.setAttribute('data-dk-mode',t===s.dark?'dark':'light')})()";

const TOGGLE_THEME: &str = "var s=this.dataset,d=document.documentElement,n=d.getAttribute('data-theme')===s.dark?s.light:s.dark;d.setAttribute('data-theme',n);d.setAttribute('data-dk-mode',n===s.dark?'dark':'light');try{localStorage.setItem(s.key,n)}catch(e){}";

/// Applies the visitor's stored theme (or the configured default) before the
/// page paints. Place it early in `<head>`. Renders nothing unless the
/// registry was built with `with_theme` or `with_theme_toggle`.
#[component]
pub async fn docs_theme_script(registry: &DocsRegistry) -> Result<impl View> {
    Ok(view! {
        if let Some(theme) = &registry.theme {
            <script
                data-key=(theme.storage_key.as_str())
                data-default=(theme.default_theme.as_str())
                data-dark=(theme.toggle_themes.as_ref().map(|(_, dark)| dark.as_str()))
            >
                // Static, trusted source.
                (Unescaped::new_unchecked(APPLY_STORED_THEME))
            </script>
        }
    })
}

/// The header's light/dark button; nothing unless `with_theme_toggle` is set.
#[component]
pub(crate) async fn theme_toggle(registry: &DocsRegistry) -> Result<impl View> {
    let theme = registry.theme.as_ref();
    let toggle = theme.and_then(|t| t.toggle_themes.as_ref());
    Ok(view! {
        if let (Some(theme), Some((light, dark))) = (theme, toggle) {
            <button
                type="button"
                class="dk-theme-toggle btn btn-ghost btn-sm btn-square shrink-0"
                aria-label="Toggle light and dark theme"
                data-key=(theme.storage_key.as_str())
                data-light=(light.as_str())
                data-dark=(dark.as_str())
                onclick=(TOGGLE_THEME)
            >
                // Sun in dark mode (switch to light), moon otherwise.
                icon(glyph: LdSun, class: "size-5 hidden [[data-dk-mode=dark]_&]:block")
                icon(glyph: LdMoon, class: "size-5 [[data-dk-mode=dark]_&]:hidden")
            </button>
        }
    })
}
