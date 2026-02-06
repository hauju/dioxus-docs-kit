//! Two-column Mintlify-style endpoint page component.

use dioxus::prelude::*;

use crate::parser::{ApiOperation, OpenApiSpec, highlight_code};

use super::method_badge::MethodBadge;
use super::parameters_list::ParametersList;
use super::request_body::RequestBodySection;
use super::responses_list::ResponsesList;

/// Props for EndpointPage component.
#[derive(Props, Clone, PartialEq)]
pub struct EndpointPageProps {
    /// The operation to display.
    pub operation: ApiOperation,
    /// The full OpenAPI spec (for base URL).
    pub spec: OpenApiSpec,
}

/// Full-page two-column layout for a single API endpoint.
///
/// Left column: method badge, path, summary, description, parameters, request body, responses.
/// Right column (sticky): curl example, response JSON example.
#[component]
pub fn EndpointPage(props: EndpointPageProps) -> Element {
    let op = &props.operation;
    let spec = &props.spec;

    let base_url = spec
        .servers
        .first()
        .map(|s| s.url.as_str())
        .unwrap_or("https://api.example.com");

    let curl = op.generate_curl(base_url);
    let curl_highlighted = highlight_code(&curl, Some("bash"));

    let response_example = op.generate_response_example();

    let method_bg = op.method.bg_class();

    rsx! {
        div { class: "flex flex-col lg:flex-row gap-0",
            // Left column — scrollable content
            div { class: "flex-1 min-w-0 px-8 py-12 lg:px-12",
                div { class: "max-w-2xl",
                    // Method + Path header
                    div { class: "flex items-center gap-3 mb-6",
                        span {
                            class: "px-3 py-1.5 rounded-lg font-mono text-sm font-bold border {method_bg}",
                            "{op.method.as_str()}"
                        }
                        code { class: "font-mono text-lg text-base-content",
                            "{op.path}"
                        }
                        if op.deprecated {
                            span { class: "badge badge-warning badge-sm", "deprecated" }
                        }
                    }

                    // Summary as heading
                    if let Some(summary) = &op.summary {
                        h1 { class: "text-3xl font-bold tracking-tight mb-3",
                            "{summary}"
                        }
                    }

                    // Description
                    if let Some(desc) = &op.description {
                        p { class: "text-base text-base-content/70 mb-6 leading-relaxed",
                            "{desc}"
                        }
                    }

                    // Base URL
                    div { class: "mb-8 flex items-center gap-2",
                        span { class: "text-xs text-base-content/50 font-semibold uppercase tracking-wider",
                            "Base URL"
                        }
                        code { class: "text-sm font-mono text-base-content/70 bg-base-200 px-2 py-1 rounded",
                            "{base_url}"
                        }
                    }

                    // Parameters section
                    if !op.parameters.is_empty() {
                        div { class: "mb-8",
                            h2 { class: "text-lg font-semibold mb-4 pb-2 border-b border-base-300",
                                "Parameters"
                            }
                            ParametersList { parameters: op.parameters.clone() }
                        }
                    }

                    // Request Body section
                    if let Some(body) = &op.request_body {
                        div { class: "mb-8",
                            h2 { class: "text-lg font-semibold mb-4 pb-2 border-b border-base-300",
                                "Request Body"
                            }
                            RequestBodySection { body: body.clone() }
                        }
                    }

                    // Responses section
                    if !op.responses.is_empty() {
                        div { class: "mb-8",
                            h2 { class: "text-lg font-semibold mb-4 pb-2 border-b border-base-300",
                                "Responses"
                            }
                            ResponsesList { responses: op.responses.clone() }
                        }
                    }
                }
            }

            // Right column — sticky code examples
            aside { class: "lg:w-[45%] lg:shrink-0 lg:border-l border-base-300 bg-base-200/20",
                div { class: "lg:sticky lg:top-16 lg:h-[calc(100vh-4rem)] lg:overflow-y-auto p-6 space-y-6",
                    // Request example
                    div {
                        h3 { class: "text-sm font-semibold text-base-content/70 uppercase tracking-wider mb-3",
                            "Request"
                        }
                        div { class: "rounded-lg border border-base-300 overflow-hidden",
                            div { class: "px-3 py-2 bg-base-300/50 border-b border-base-300 flex items-center gap-2",
                                MethodBadge { method: op.method }
                                code { class: "text-xs font-mono text-base-content/70 truncate",
                                    "{op.path}"
                                }
                            }
                            pre { class: "bg-base-300/30 p-4 overflow-x-auto syntax-highlight",
                                code {
                                    class: "text-sm font-mono leading-relaxed",
                                    dangerous_inner_html: "{curl_highlighted}",
                                }
                            }
                        }
                    }

                    // Response example
                    if let Some((status_code, response_json)) = &response_example {
                        {
                            let json_highlighted = highlight_code(response_json, Some("json"));
                            let status_color = if status_code.starts_with('2') {
                                "badge-success"
                            } else if status_code.starts_with('3') {
                                "badge-info"
                            } else {
                                "badge-ghost"
                            };
                            rsx! {
                                div {
                                    h3 { class: "text-sm font-semibold text-base-content/70 uppercase tracking-wider mb-3",
                                        "Response"
                                    }
                                    div { class: "rounded-lg border border-base-300 overflow-hidden",
                                        div { class: "px-3 py-2 bg-base-300/50 border-b border-base-300 flex items-center gap-2",
                                            span { class: "badge {status_color} badge-sm font-mono font-bold",
                                                "{status_code}"
                                            }
                                            span { class: "text-xs text-base-content/50",
                                                "application/json"
                                            }
                                        }
                                        pre { class: "bg-base-300/30 p-4 overflow-x-auto syntax-highlight max-h-[60vh]",
                                            code {
                                                class: "text-sm font-mono leading-relaxed",
                                                dangerous_inner_html: "{json_highlighted}",
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
    }
}
