//! Icon mapping utilities for MDX components.
//!
//! Maps common icon names from Mintlify/FontAwesome style to Lucide icons.

use crate::lucide::*;
use dioxus::prelude::*;

/// Render an icon by name.
///
/// Maps common icon names (e.g., "code", "folder", "star") to Lucide icons.
/// Returns a default icon if the name is not recognized.
#[component]
pub fn MdxIcon(
    /// Icon name (e.g., "code", "brain-circuit", "folder").
    name: String,
    /// CSS classes to apply (default: "size-5").
    #[props(default = "size-5".to_string())]
    class: String,
) -> Element {
    rsx! { Icon { class, icon: icon_for_name(&name) } }
}

/// Render a callout-specific icon.
#[component]
pub fn CalloutIcon(
    /// Callout type: "tip", "note", "warning", or "info".
    callout_type: String,
    /// CSS classes (default: "size-5").
    #[props(default = "size-5".to_string())]
    class: String,
) -> Element {
    match callout_type.to_lowercase().as_str() {
        "tip" => rsx! { Icon { class, icon: LdLightbulb } },
        "note" => rsx! { Icon { class, icon: LdInfo } },
        "warning" => rsx! { Icon { class, icon: LdTriangleAlert } },
        "info" => rsx! { Icon { class, icon: LdInfo } },
        _ => rsx! { Icon { class, icon: LdInfo } },
    }
}
