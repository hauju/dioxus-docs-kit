//! ResponseField and Expandable components for API response documentation.

use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::ld_icons::*};

use crate::parser::{ExpandableNode, ResponseFieldNode};

/// Props for DocResponseField component.
#[derive(Props, Clone, PartialEq)]
pub struct DocResponseFieldProps {
    /// The response field to render.
    pub field: ResponseFieldNode,
    /// Nesting depth for indentation.
    #[props(default = 0)]
    pub depth: usize,
}

/// API response field documentation.
#[component]
pub fn DocResponseField(props: DocResponseFieldProps) -> Element {
    let field = &props.field;

    let description_html = if !field.content.is_empty() {
        markdown::to_html_with_options(&field.content, &markdown::Options::gfm())
            .unwrap_or_else(|_| field.content.clone())
    } else {
        String::new()
    };

    let indent_class = if props.depth > 0 {
        "ml-4 border-l-2 border-base-300 pl-4"
    } else {
        ""
    };

    rsx! {
        div { class: "py-3 {indent_class}",
            div { class: "flex items-start gap-2 flex-wrap",
                // Field name
                code { class: "font-mono font-semibold text-base-content bg-base-300 px-2 py-0.5 rounded",
                    "{field.name}"
                }
                // Type badge
                span { class: "badge badge-outline badge-sm",
                    "{field.field_type}"
                }
                // Required indicator
                if field.required {
                    span { class: "badge badge-success badge-sm",
                        "required"
                    }
                }
            }
            // Description
            if !description_html.is_empty() {
                div {
                    class: "prose prose-sm max-w-none mt-2 text-base-content/80",
                    dangerous_inner_html: description_html,
                }
            }
            // Nested expandable
            if let Some(expandable) = &field.expandable {
                DocExpandable {
                    expandable: expandable.clone(),
                    depth: props.depth + 1,
                }
            }
        }
    }
}

/// Props for DocExpandable component.
#[derive(Props, Clone, PartialEq)]
pub struct DocExpandableProps {
    /// The expandable section to render.
    pub expandable: ExpandableNode,
    /// Nesting depth for indentation.
    #[props(default = 0)]
    pub depth: usize,
}

/// Expandable section for nested fields.
#[component]
pub fn DocExpandable(props: DocExpandableProps) -> Element {
    let mut expanded = use_signal(|| false);
    let expandable = &props.expandable;

    let chevron_class = if expanded() {
        "size-4 text-base-content/50 transform rotate-90 transition-transform"
    } else {
        "size-4 text-base-content/50 transition-transform"
    };

    rsx! {
        div { class: "mt-3 border border-base-300 rounded-lg overflow-hidden",
            // Header
            button {
                class: "w-full flex items-center gap-2 px-3 py-2 text-left hover:bg-base-200 transition-colors text-sm",
                onclick: move |_| expanded.set(!expanded()),
                Icon { class: chevron_class, icon: LdChevronRight }
                span { class: "font-medium text-base-content/70",
                    "{expandable.title}"
                }
            }
            // Content
            if expanded() {
                div { class: "px-3 pb-3 border-t border-base-300 bg-base-200/30",
                    for (i, field) in expandable.fields.iter().enumerate() {
                        DocResponseField {
                            key: "{i}",
                            field: field.clone(),
                            depth: props.depth,
                        }
                    }
                }
            }
        }
    }
}
