//! Responses list component for API endpoint documentation.

use dioxus::prelude::*;
use dioxus_free_icons::{icons::ld_icons::*, Icon};

use crate::parser::ApiResponse;

use super::schema_viewer::SchemaViewer;

/// Props for ResponsesList component.
#[derive(Props, Clone, PartialEq)]
pub struct ResponsesListProps {
    /// The responses to display.
    pub responses: Vec<ApiResponse>,
}

/// List of API responses with status codes.
#[component]
pub fn ResponsesList(props: ResponsesListProps) -> Element {
    if props.responses.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "space-y-2",
            for response in &props.responses {
                ResponseItem { key: "{response.status_code}", response: response.clone() }
            }
        }
    }
}

/// Props for ResponseItem component.
#[derive(Props, Clone, PartialEq)]
pub struct ResponseItemProps {
    /// The response to display.
    pub response: ApiResponse,
}

/// Single response item with collapsible content.
#[component]
pub fn ResponseItem(props: ResponseItemProps) -> Element {
    let mut is_expanded = use_signal(|| false);
    let response = &props.response;
    let badge_class = response.status_badge_class();

    let has_content = !response.content.is_empty();

    rsx! {
        div { class: "border border-base-300 rounded-lg overflow-hidden",
            // Header
            button {
                class: "w-full flex items-center gap-3 px-3 py-2 text-left hover:bg-base-200 transition-colors",
                disabled: !has_content,
                onclick: move |_| {
                    if has_content {
                        is_expanded.set(!is_expanded());
                    }
                },

                // Expand icon
                if has_content {
                    Icon {
                        class: if is_expanded() { "size-4 text-base-content/50 transform rotate-90 transition-transform" } else { "size-4 text-base-content/50 transition-transform" },
                        icon: LdChevronRight
                    }
                }

                // Status code badge
                span { class: "badge {badge_class} badge-sm font-mono font-bold",
                    "{response.status_code}"
                }

                // Description
                span { class: "text-sm text-base-content/70 flex-1",
                    "{response.description}"
                }
            }

            // Content
            if is_expanded() && has_content {
                div { class: "border-t border-base-300 bg-base-200/30",
                    for content in &response.content {
                        div { class: "p-3",
                            // Media type
                            div { class: "mb-2",
                                code { class: "text-xs font-mono text-base-content/50",
                                    "{content.media_type}"
                                }
                            }

                            // Schema
                            if let Some(schema) = &content.schema {
                                SchemaViewer {
                                    schema: schema.clone(),
                                    expanded: true,
                                }
                            }

                            // Example
                            if let Some(example) = &content.example {
                                div { class: "mt-3 p-2 bg-base-300 rounded",
                                    span { class: "text-xs text-base-content/50 font-semibold", "Example" }
                                    pre { class: "mt-1 text-xs font-mono text-secondary overflow-x-auto whitespace-pre-wrap",
                                        "{example}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
