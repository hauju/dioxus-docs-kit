//! Inline Lucide icons, from the SVG data vendored in `dioxus-mdx`.

use dioxus_mdx::lucide::LucideIcon;
use topcoat::{
    Result,
    view::{Unescaped, View, component, view},
};

/// Renders a Lucide icon as an inline 24x24 stroke `<svg>`.
#[component]
pub async fn icon(glyph: LucideIcon, class: &str) -> Result<impl View> {
    Ok(view! {
        <svg
            class=(class)
            width="20"
            height="20"
            viewBox="0 0 24 24"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            // Vendored, trusted SVG markup.
            (Unescaped::new_unchecked(glyph.svg_inner()))
        </svg>
    })
}
