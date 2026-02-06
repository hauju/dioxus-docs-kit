//! Main OpenAPI specification viewer component.

use std::collections::BTreeMap;

use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::ld_icons::*};

use crate::parser::{ApiOperation, ApiTag, OpenApiSpec, SchemaDefinition};

use super::schema_viewer::SchemaViewer;
use super::tag_group::{TagGroup, UngroupedEndpoints};

/// Props for OpenApiViewer component.
#[derive(Props, Clone, PartialEq)]
pub struct OpenApiViewerProps {
    /// The parsed OpenAPI specification.
    pub spec: OpenApiSpec,
    /// Optional filter to show only specific tags.
    #[props(default)]
    pub tags: Option<Vec<String>>,
    /// Whether to show schema definitions section.
    #[props(default = true)]
    pub show_schemas: bool,
}

/// Main OpenAPI specification viewer.
#[component]
pub fn OpenApiViewer(props: OpenApiViewerProps) -> Element {
    let spec = &props.spec;

    // Group operations by tag
    let (grouped_ops, ungrouped_ops) = group_operations_by_tag(&spec.operations, &spec.tags);

    // Filter tags if specified
    let filtered_groups: Vec<_> = if let Some(filter_tags) = &props.tags {
        grouped_ops
            .into_iter()
            .filter(|(tag, _)| {
                filter_tags
                    .iter()
                    .any(|t| t.eq_ignore_ascii_case(&tag.name))
            })
            .collect()
    } else {
        grouped_ops
    };

    rsx! {
        div { class: "openapi-viewer",
            // API Info header
            ApiInfoHeader { info: spec.info.clone(), servers: spec.servers.clone() }

            // Endpoints grouped by tag
            div { class: "mt-6",
                for (tag, ops) in filtered_groups {
                    TagGroup {
                        key: "{tag.name}",
                        tag: tag.clone(),
                        operations: ops,
                    }
                }

                // Ungrouped endpoints (only show if no tag filter)
                if props.tags.is_none() {
                    UngroupedEndpoints { operations: ungrouped_ops }
                }
            }

            // Schema definitions
            if props.show_schemas && !spec.schemas.is_empty() {
                SchemaDefinitions { schemas: spec.schemas.clone() }
            }
        }
    }
}

/// Group operations by their tags.
fn group_operations_by_tag(
    operations: &[ApiOperation],
    tags: &[ApiTag],
) -> (Vec<(ApiTag, Vec<ApiOperation>)>, Vec<ApiOperation>) {
    let mut grouped: BTreeMap<String, Vec<ApiOperation>> = BTreeMap::new();
    let mut ungrouped = Vec::new();

    for op in operations {
        if op.tags.is_empty() {
            ungrouped.push(op.clone());
        } else {
            for tag_name in &op.tags {
                grouped
                    .entry(tag_name.clone())
                    .or_default()
                    .push(op.clone());
            }
        }
    }

    // Convert to vec with tag metadata, preserving tag order from spec
    let mut result = Vec::new();
    for tag in tags {
        if let Some(ops) = grouped.remove(&tag.name) {
            result.push((tag.clone(), ops));
        }
    }

    // Add any remaining tags that weren't in the spec's tag list
    for (tag_name, ops) in grouped {
        result.push((
            ApiTag {
                name: tag_name,
                description: None,
            },
            ops,
        ));
    }

    (result, ungrouped)
}

/// Props for ApiInfoHeader component.
#[derive(Props, Clone, PartialEq)]
pub struct ApiInfoHeaderProps {
    /// API info metadata.
    pub info: crate::parser::ApiInfo,
    /// Server URLs.
    pub servers: Vec<crate::parser::ApiServer>,
}

/// API information header with title, version, and servers.
#[component]
pub fn ApiInfoHeader(props: ApiInfoHeaderProps) -> Element {
    let info = &props.info;

    rsx! {
        div { class: "border-b border-base-300 pb-4 mb-4",
            // Title and version
            div { class: "flex items-center gap-3 flex-wrap",
                h2 { class: "text-2xl font-bold text-base-content",
                    "{info.title}"
                }
                span { class: "badge badge-primary badge-outline",
                    "v{info.version}"
                }
            }

            // Description
            if let Some(desc) = &info.description {
                p { class: "mt-2 text-base-content/70",
                    "{desc}"
                }
            }

            // Servers
            if !props.servers.is_empty() {
                div { class: "mt-4",
                    span { class: "text-sm font-semibold text-base-content/60 flex items-center gap-2",
                        Icon { class: "size-4", icon: LdServer }
                        "Servers"
                    }
                    div { class: "mt-2 space-y-1",
                        for server in &props.servers {
                            div { class: "flex items-center gap-2",
                                code { class: "text-sm font-mono text-primary bg-base-200 px-2 py-1 rounded",
                                    "{server.url}"
                                }
                                if let Some(desc) = &server.description {
                                    span { class: "text-sm text-base-content/50",
                                        "- {desc}"
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

/// Props for SchemaDefinitions component.
#[derive(Props, Clone, PartialEq)]
pub struct SchemaDefinitionsProps {
    /// Schema definitions by name.
    pub schemas: BTreeMap<String, SchemaDefinition>,
}

/// Schema definitions section.
#[component]
pub fn SchemaDefinitions(props: SchemaDefinitionsProps) -> Element {
    let mut is_expanded = use_signal(|| false);

    rsx! {
        div { class: "mt-8 border-t border-base-300 pt-4",
            // Header
            button {
                class: "w-full flex items-center gap-2 py-2 text-left",
                onclick: move |_| is_expanded.set(!is_expanded()),

                Icon {
                    class: if is_expanded() { "size-5 text-base-content/50 transform rotate-90 transition-transform" } else { "size-5 text-base-content/50 transition-transform" },
                    icon: LdChevronRight
                }

                h3 { class: "text-lg font-semibold text-base-content flex items-center gap-2",
                    Icon { class: "size-5", icon: LdBraces }
                    "Schema Definitions"
                }

                span { class: "badge badge-ghost badge-sm",
                    "{props.schemas.len()}"
                }
            }

            // Schema list
            if is_expanded() {
                div { class: "mt-4 space-y-4",
                    for (name, schema) in &props.schemas {
                        div { class: "border border-base-300 rounded-lg overflow-hidden",
                            // Schema name header
                            div { class: "px-4 py-2 bg-base-200 border-b border-base-300",
                                code { class: "font-mono font-semibold text-primary",
                                    "{name}"
                                }
                            }
                            // Schema content
                            div { class: "p-4",
                                SchemaViewer {
                                    schema: schema.clone(),
                                    expanded: true,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
