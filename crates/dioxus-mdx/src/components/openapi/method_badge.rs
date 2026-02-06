//! HTTP method badge component.

use dioxus::prelude::*;

use crate::parser::HttpMethod;

/// Props for MethodBadge component.
#[derive(Props, Clone, PartialEq)]
pub struct MethodBadgeProps {
    /// The HTTP method.
    pub method: HttpMethod,
    /// Optional additional classes.
    #[props(default)]
    pub class: String,
}

/// Colored badge for HTTP methods.
#[component]
pub fn MethodBadge(props: MethodBadgeProps) -> Element {
    let badge_class = props.method.badge_class();
    let method_str = props.method.as_str();

    rsx! {
        span {
            class: "badge {badge_class} badge-sm font-mono font-bold {props.class}",
            "{method_str}"
        }
    }
}
