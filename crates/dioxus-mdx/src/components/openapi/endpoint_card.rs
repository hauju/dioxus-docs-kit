//! Endpoint card component for displaying a single API operation.

use dioxus::prelude::*;
use dioxus_free_icons::{icons::ld_icons::*, Icon};

use crate::parser::ApiOperation;

use super::method_badge::MethodBadge;
use super::parameters_list::ParametersList;
use super::request_body::RequestBodySection;
use super::responses_list::ResponsesList;

/// Props for EndpointCard component.
#[derive(Props, Clone, PartialEq)]
pub struct EndpointCardProps {
    /// The operation to display.
    pub operation: ApiOperation,
}

/// Collapsible card for an API endpoint.
#[component]
pub fn EndpointCard(props: EndpointCardProps) -> Element {
    let mut is_expanded = use_signal(|| false);
    let op = &props.operation;

    rsx! {
        div { class: "border border-base-300 rounded-lg overflow-hidden my-3",
            // Header - always visible
            button {
                class: "w-full flex items-center gap-3 px-4 py-3 text-left hover:bg-base-200/50 transition-colors",
                onclick: move |_| is_expanded.set(!is_expanded()),

                // Expand/collapse chevron
                Icon {
                    class: if is_expanded() { "size-4 text-base-content/50 transform rotate-90 transition-transform shrink-0" } else { "size-4 text-base-content/50 transition-transform shrink-0" },
                    icon: LdChevronRight
                }

                // Method badge
                MethodBadge { method: op.method }

                // Path
                code { class: "font-mono text-sm text-base-content",
                    "{op.path}"
                }

                // Deprecated indicator
                if op.deprecated {
                    span { class: "badge badge-warning badge-sm",
                        "deprecated"
                    }
                }

                // Summary (truncated)
                if let Some(summary) = &op.summary {
                    span { class: "text-sm text-base-content/60 truncate ml-auto max-w-[40%]",
                        "{summary}"
                    }
                }
            }

            // Expanded content
            if is_expanded() {
                div { class: "border-t border-base-300",
                    // Summary and description
                    div { class: "px-4 py-3 bg-base-200/30",
                        if let Some(summary) = &op.summary {
                            h4 { class: "font-semibold text-base-content",
                                "{summary}"
                            }
                        }
                        if let Some(desc) = &op.description {
                            p { class: "mt-2 text-sm text-base-content/70",
                                "{desc}"
                            }
                        }

                        // Operation ID
                        if let Some(op_id) = &op.operation_id {
                            div { class: "mt-2",
                                span { class: "text-xs text-base-content/50", "Operation ID: " }
                                code { class: "text-xs font-mono text-base-content/70",
                                    "{op_id}"
                                }
                            }
                        }
                    }

                    // Parameters section
                    if !op.parameters.is_empty() {
                        div { class: "px-4 py-3 border-t border-base-300",
                            h5 { class: "text-sm font-semibold text-base-content/80 mb-3 flex items-center gap-2",
                                Icon { class: "size-4", icon: LdSettings2 }
                                "Parameters"
                            }
                            ParametersList { parameters: op.parameters.clone() }
                        }
                    }

                    // Request body section
                    if let Some(body) = &op.request_body {
                        div { class: "px-4 py-3 border-t border-base-300",
                            h5 { class: "text-sm font-semibold text-base-content/80 mb-3 flex items-center gap-2",
                                Icon { class: "size-4", icon: LdUpload }
                                "Request Body"
                            }
                            RequestBodySection { body: body.clone() }
                        }
                    }

                    // Responses section
                    if !op.responses.is_empty() {
                        div { class: "px-4 py-3 border-t border-base-300",
                            h5 { class: "text-sm font-semibold text-base-content/80 mb-3 flex items-center gap-2",
                                Icon { class: "size-4", icon: LdDownload }
                                "Responses"
                            }
                            ResponsesList { responses: op.responses.clone() }
                        }
                    }
                }
            }
        }
    }
}
