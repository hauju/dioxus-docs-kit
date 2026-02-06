//! Tag group component for grouping endpoints by tag.

use dioxus::prelude::*;
use dioxus_free_icons::{icons::ld_icons::*, Icon};

use crate::parser::{ApiOperation, ApiTag};

use super::endpoint_card::EndpointCard;

/// Props for TagGroup component.
#[derive(Props, Clone, PartialEq)]
pub struct TagGroupProps {
    /// The tag metadata.
    pub tag: ApiTag,
    /// Operations belonging to this tag.
    pub operations: Vec<ApiOperation>,
}

/// Group of endpoints under a tag heading.
#[component]
pub fn TagGroup(props: TagGroupProps) -> Element {
    let mut is_expanded = use_signal(|| true);
    let tag = &props.tag;

    rsx! {
        div { class: "my-6",
            // Tag header
            button {
                class: "w-full flex items-center gap-2 py-2 text-left group",
                onclick: move |_| is_expanded.set(!is_expanded()),

                Icon {
                    class: if is_expanded() { "size-5 text-base-content/50 transform rotate-90 transition-transform" } else { "size-5 text-base-content/50 transition-transform" },
                    icon: LdChevronRight
                }

                h3 { class: "text-lg font-semibold text-base-content",
                    "{tag.name}"
                }

                span { class: "badge badge-ghost badge-sm",
                    "{props.operations.len()}"
                }
            }

            // Tag description
            if let Some(desc) = &tag.description {
                if is_expanded() {
                    p { class: "text-sm text-base-content/70 ml-7 mb-3",
                        "{desc}"
                    }
                }
            }

            // Endpoints
            if is_expanded() {
                div { class: "ml-4",
                    for op in &props.operations {
                        EndpointCard {
                            key: "{op.method.as_str()}-{op.path}",
                            operation: op.clone(),
                        }
                    }
                }
            }
        }
    }
}

/// Props for UngroupedEndpoints component.
#[derive(Props, Clone, PartialEq)]
pub struct UngroupedEndpointsProps {
    /// Operations without tags.
    pub operations: Vec<ApiOperation>,
}

/// Endpoints that don't belong to any tag.
#[component]
pub fn UngroupedEndpoints(props: UngroupedEndpointsProps) -> Element {
    if props.operations.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "my-6",
            h3 { class: "text-lg font-semibold text-base-content mb-3",
                "Other Endpoints"
            }
            for op in &props.operations {
                EndpointCard {
                    key: "{op.method.as_str()}-{op.path}",
                    operation: op.clone(),
                }
            }
        }
    }
}
