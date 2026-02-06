//! ParamField component for API parameter documentation.

use dioxus::prelude::*;

use crate::components::DocNodeRenderer;
use crate::parser::ParamFieldNode;

/// Props for DocParamField component.
#[derive(Props, Clone, PartialEq)]
pub struct DocParamFieldProps {
    /// The parameter field to render.
    pub field: ParamFieldNode,
}

/// API parameter documentation field.
#[component]
pub fn DocParamField(props: DocParamFieldProps) -> Element {
    let field = &props.field;

    rsx! {
        div { class: "border-b border-base-300 py-4 first:pt-0 last:border-b-0",
            div { class: "flex items-center gap-3 flex-wrap",
                // Parameter name - primary colored monospace
                code { class: "font-mono font-semibold text-primary",
                    "{field.name}"
                }
                // Type badge - subtle gray
                span { class: "text-xs px-2 py-0.5 rounded-full bg-base-300 text-base-content/70",
                    "{field.param_type}"
                }
                // Required indicator
                if field.required {
                    span { class: "text-xs px-2 py-0.5 rounded-full bg-error/20 text-error",
                        "required"
                    }
                }
                // Default value - styled as code badge
                if let Some(default) = &field.default {
                    span { class: "text-xs px-2 py-0.5 rounded-full bg-base-300 text-base-content/70 font-mono",
                        "default:"
                        span { class: "text-primary", "\"{default}\"" }
                    }
                }
            }
            // Description - render nested content
            if !field.content.is_empty() {
                div {
                    class: "mt-2 text-base-content/70",
                    for (i, node) in field.content.iter().enumerate() {
                        DocNodeRenderer { key: "{i}", node: node.clone() }
                    }
                }
            }
        }
    }
}
