//! Callout component for Tip, Note, Warning, and Info boxes.

use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::ld_icons::*};

use crate::parser::CalloutType;

/// Props for DocCallout component.
#[derive(Props, Clone, PartialEq)]
pub struct DocCalloutProps {
    /// Type of callout (Tip, Note, Warning, Info).
    pub callout_type: CalloutType,
    /// Content to display (rendered as markdown).
    pub content: String,
}

/// Callout box component styled with DaisyUI alerts.
#[component]
pub fn DocCallout(props: DocCalloutProps) -> Element {
    let (bg_class, border_class, icon_class, shadow_class) = match props.callout_type {
        CalloutType::Tip => (
            "bg-success/5",
            "border-success/40",
            "text-success",
            "shadow-success/5",
        ),
        CalloutType::Note => ("bg-info/5", "border-info/40", "text-info", "shadow-info/5"),
        CalloutType::Warning => (
            "bg-warning/5",
            "border-warning/40",
            "text-warning",
            "shadow-warning/5",
        ),
        CalloutType::Info => ("bg-info/5", "border-info/40", "text-info", "shadow-info/5"),
    };

    // Render markdown content
    let html = markdown::to_html_with_options(&props.content, &markdown::Options::gfm())
        .unwrap_or_else(|_| props.content.clone());

    rsx! {
        div {
            class: "my-6 px-4 py-4 rounded-lg border-l-4 {bg_class} {border_class} shadow-sm {shadow_class}",
            role: "alert",
            div { class: "flex gap-4",
                // Icon - slightly larger
                div { class: "{icon_class} mt-0.5 shrink-0",
                    match props.callout_type {
                        CalloutType::Tip => rsx! { Icon { class: "size-5", icon: LdLightbulb } },
                        CalloutType::Note => rsx! { Icon { class: "size-5", icon: LdInfo } },
                        CalloutType::Warning => rsx! { Icon { class: "size-5", icon: LdTriangleAlert } },
                        CalloutType::Info => rsx! { Icon { class: "size-5", icon: LdInfo } },
                    }
                }
                // Content
                div { class: "flex-1 min-w-0",
                    // Label - inline with better weight
                    span { class: "font-semibold {icon_class} text-sm uppercase tracking-wide",
                        "{props.callout_type.as_str()}"
                    }
                    // Content (markdown rendered) - better spacing
                    div {
                        class: "prose prose-sm max-w-none text-base-content/85 mt-1.5 [&>p:first-child]:mt-0 [&>p:last-child]:mb-0",
                        dangerous_inner_html: html,
                    }
                }
            }
        }
    }
}
