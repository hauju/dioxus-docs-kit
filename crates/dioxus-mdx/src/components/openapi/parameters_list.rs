//! Parameters list component for API endpoint documentation.

use dioxus::prelude::*;

use crate::parser::ApiParameter;

use super::schema_viewer::SchemaViewer;

/// Props for ParametersList component.
#[derive(Props, Clone, PartialEq)]
pub struct ParametersListProps {
    /// The parameters to display.
    pub parameters: Vec<ApiParameter>,
}

/// List of API parameters with type info.
#[component]
pub fn ParametersList(props: ParametersListProps) -> Element {
    if props.parameters.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "space-y-1",
            for param in &props.parameters {
                ParameterItem { key: "{param.name}", parameter: param.clone() }
            }
        }
    }
}

/// Props for ParameterItem component.
#[derive(Props, Clone, PartialEq)]
pub struct ParameterItemProps {
    /// The parameter to display.
    pub parameter: ApiParameter,
}

/// Single parameter item.
#[component]
pub fn ParameterItem(props: ParameterItemProps) -> Element {
    let param = &props.parameter;
    let location_badge = param.location.badge_class();
    let location_str = param.location.as_str();

    rsx! {
        div { class: "border-b border-base-300 py-3 first:pt-0 last:border-b-0",
            div { class: "flex items-center gap-2 flex-wrap",
                // Parameter name
                code { class: "font-mono font-semibold text-primary",
                    "{param.name}"
                }

                // Location badge
                span { class: "badge {location_badge} badge-sm badge-outline",
                    "{location_str}"
                }

                // Type from schema
                if let Some(schema) = &param.schema {
                    span { class: "text-xs px-2 py-0.5 rounded-full bg-base-300 text-base-content/70",
                        "{schema.display_type()}"
                    }
                }

                // Required indicator
                if param.required {
                    span { class: "text-xs px-2 py-0.5 rounded-full bg-error/20 text-error",
                        "required"
                    }
                }

                // Deprecated indicator
                if param.deprecated {
                    span { class: "text-xs px-2 py-0.5 rounded-full bg-warning/20 text-warning line-through",
                        "deprecated"
                    }
                }
            }

            // Description
            if let Some(desc) = &param.description {
                p { class: "mt-2 text-sm text-base-content/70",
                    "{desc}"
                }
            }

            // Schema details (for complex types)
            if let Some(schema) = &param.schema {
                if schema.is_complex() {
                    div { class: "mt-2",
                        SchemaViewer {
                            schema: schema.clone(),
                            depth: 1,
                        }
                    }
                }
            }

            // Example
            if let Some(example) = &param.example {
                div { class: "mt-2",
                    span { class: "text-xs text-base-content/50", "Example: " }
                    code { class: "text-xs font-mono text-secondary",
                        "{example}"
                    }
                }
            }
        }
    }
}
