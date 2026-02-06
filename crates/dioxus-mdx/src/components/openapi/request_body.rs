//! Request body component for API endpoint documentation.

use dioxus::prelude::*;

use crate::parser::ApiRequestBody;

use super::schema_viewer::SchemaViewer;

/// Props for RequestBodySection component.
#[derive(Props, Clone, PartialEq)]
pub struct RequestBodySectionProps {
    /// The request body to display.
    pub body: ApiRequestBody,
}

/// Request body schema viewer.
#[component]
pub fn RequestBodySection(props: RequestBodySectionProps) -> Element {
    let body = &props.body;

    rsx! {
        div { class: "space-y-3",
            // Required indicator
            div { class: "flex items-center gap-2",
                if body.required {
                    span { class: "text-xs px-2 py-0.5 rounded-full bg-error/20 text-error",
                        "required"
                    }
                } else {
                    span { class: "text-xs px-2 py-0.5 rounded-full bg-base-300 text-base-content/50",
                        "optional"
                    }
                }
            }

            // Description
            if let Some(desc) = &body.description {
                p { class: "text-sm text-base-content/70",
                    "{desc}"
                }
            }

            // Content by media type
            for content in &body.content {
                div { class: "border border-base-300 rounded-lg overflow-hidden",
                    // Media type header
                    div { class: "px-3 py-2 bg-base-200 border-b border-base-300",
                        code { class: "text-xs font-mono text-base-content/70",
                            "{content.media_type}"
                        }
                    }

                    // Schema
                    if let Some(schema) = &content.schema {
                        div { class: "p-3",
                            SchemaViewer {
                                schema: schema.clone(),
                                expanded: true,
                            }
                        }
                    }

                    // Example
                    if let Some(example) = &content.example {
                        div { class: "px-3 py-2 border-t border-base-300 bg-base-200/50",
                            span { class: "text-xs text-base-content/50 font-semibold", "Example" }
                            pre { class: "mt-1 text-xs font-mono text-secondary overflow-x-auto",
                                "{example}"
                            }
                        }
                    }
                }
            }
        }
    }
}
