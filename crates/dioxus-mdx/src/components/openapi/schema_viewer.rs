//! Schema viewer component for displaying type definitions.

use dioxus::prelude::*;
use dioxus_free_icons::{icons::ld_icons::*, Icon};

use crate::parser::SchemaDefinition;

/// Props for SchemaViewer component.
#[derive(Props, Clone, PartialEq)]
pub struct SchemaViewerProps {
    /// The schema to display.
    pub schema: SchemaDefinition,
    /// Nesting depth for indentation.
    #[props(default = 0)]
    pub depth: usize,
    /// Whether this is initially expanded.
    #[props(default = false)]
    pub expanded: bool,
    /// Property name (for nested properties).
    #[props(default)]
    pub name: Option<String>,
    /// Whether this property is required.
    #[props(default = false)]
    pub required: bool,
}

/// Recursive schema viewer with expand/collapse for complex types.
#[component]
pub fn SchemaViewer(props: SchemaViewerProps) -> Element {
    let mut is_expanded = use_signal(|| props.expanded || props.depth == 0);
    let schema = &props.schema;

    let is_complex = schema.is_complex();
    let type_display = schema.display_type();

    let indent_class = if props.depth > 0 {
        "ml-4 border-l-2 border-base-300 pl-3"
    } else {
        ""
    };

    rsx! {
        div { class: "py-1.5 {indent_class}",
            // Header row with name, type, and expand button
            div { class: "flex items-center gap-2 flex-wrap",
                // Expand/collapse for complex types
                if is_complex && !schema.properties.is_empty() {
                    button {
                        class: "p-0.5 hover:bg-base-300 rounded transition-colors",
                        onclick: move |_| is_expanded.set(!is_expanded()),
                        Icon {
                            class: if is_expanded() { "size-4 text-base-content/50 transform rotate-90 transition-transform" } else { "size-4 text-base-content/50 transition-transform" },
                            icon: LdChevronRight
                        }
                    }
                }

                // Property name
                if let Some(name) = &props.name {
                    code { class: "font-mono font-semibold text-primary text-sm",
                        "{name}"
                    }
                }

                // Type badge
                span { class: "text-xs px-2 py-0.5 rounded-full bg-base-300 text-base-content/70",
                    "{type_display}"
                }

                // Required indicator
                if props.required {
                    span { class: "text-xs px-2 py-0.5 rounded-full bg-error/20 text-error",
                        "required"
                    }
                }

                // Nullable indicator
                if schema.nullable {
                    span { class: "text-xs px-2 py-0.5 rounded-full bg-base-300 text-base-content/50",
                        "nullable"
                    }
                }

                // Format
                if let Some(format) = &schema.format {
                    span { class: "text-xs text-base-content/50",
                        "({format})"
                    }
                }
            }

            // Description
            if let Some(desc) = &schema.description {
                p { class: "text-sm text-base-content/70 mt-1",
                    "{desc}"
                }
            }

            // Enum values
            if !schema.enum_values.is_empty() {
                div { class: "mt-1 flex items-center gap-2 flex-wrap",
                    span { class: "text-xs text-base-content/50", "Enum:" }
                    for value in &schema.enum_values {
                        code { class: "text-xs px-1.5 py-0.5 rounded bg-base-300 font-mono",
                            "{value}"
                        }
                    }
                }
            }

            // Default value
            if let Some(default) = &schema.default {
                div { class: "mt-1",
                    span { class: "text-xs text-base-content/50", "Default: " }
                    code { class: "text-xs font-mono text-primary",
                        "{default}"
                    }
                }
            }

            // Example value
            if let Some(example) = &schema.example {
                div { class: "mt-1",
                    span { class: "text-xs text-base-content/50", "Example: " }
                    code { class: "text-xs font-mono text-secondary",
                        "{example}"
                    }
                }
            }

            // Nested properties for objects
            if is_expanded() && !schema.properties.is_empty() {
                div { class: "mt-2",
                    for (name, prop_schema) in &schema.properties {
                        SchemaViewer {
                            key: "{name}",
                            schema: prop_schema.clone(),
                            depth: props.depth + 1,
                            name: Some(name.clone()),
                            required: schema.required.contains(name),
                        }
                    }
                }
            }

            // Array items
            if is_expanded() {
                if let Some(items) = &schema.items {
                    if items.is_complex() {
                        div { class: "mt-2",
                            span { class: "text-xs text-base-content/50 ml-4", "Array items:" }
                            SchemaViewer {
                                schema: (**items).clone(),
                                depth: props.depth + 1,
                            }
                        }
                    }
                }
            }

            // OneOf/AnyOf/AllOf
            if is_expanded() {
                if !schema.one_of.is_empty() {
                    div { class: "mt-2 ml-4",
                        span { class: "text-xs text-base-content/50 font-semibold", "One of:" }
                        for (i, variant) in schema.one_of.iter().enumerate() {
                            SchemaViewer {
                                key: "{i}",
                                schema: variant.clone(),
                                depth: props.depth + 1,
                            }
                        }
                    }
                }
                if !schema.any_of.is_empty() {
                    div { class: "mt-2 ml-4",
                        span { class: "text-xs text-base-content/50 font-semibold", "Any of:" }
                        for (i, variant) in schema.any_of.iter().enumerate() {
                            SchemaViewer {
                                key: "{i}",
                                schema: variant.clone(),
                                depth: props.depth + 1,
                            }
                        }
                    }
                }
                if !schema.all_of.is_empty() {
                    div { class: "mt-2 ml-4",
                        span { class: "text-xs text-base-content/50 font-semibold", "All of:" }
                        for (i, variant) in schema.all_of.iter().enumerate() {
                            SchemaViewer {
                                key: "{i}",
                                schema: variant.clone(),
                                depth: props.depth + 1,
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Props for SchemaTypeLabel component.
#[derive(Props, Clone, PartialEq)]
pub struct SchemaTypeLabelProps {
    /// The schema to display type for.
    pub schema: SchemaDefinition,
}

/// Simple type label without expand/collapse.
#[component]
pub fn SchemaTypeLabel(props: SchemaTypeLabelProps) -> Element {
    let type_display = props.schema.display_type();

    rsx! {
        span { class: "text-xs px-2 py-0.5 rounded-full bg-base-300 text-base-content/70 font-mono",
            "{type_display}"
        }
    }
}
