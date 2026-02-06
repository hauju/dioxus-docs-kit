//! API example containers for request and response code samples.

use dioxus::prelude::*;

use crate::components::DocCodeGroup;
use crate::parser::{CodeGroupNode, RequestExampleNode, ResponseExampleNode};

/// Props for DocRequestExample.
#[derive(Props, Clone, PartialEq)]
pub struct DocRequestExampleProps {
    /// The request example to render.
    pub example: RequestExampleNode,
}

/// Container for API request examples with tabs.
#[component]
pub fn DocRequestExample(props: DocRequestExampleProps) -> Element {
    rsx! {
        div { class: "my-6",
            h4 { class: "text-sm font-semibold text-base-content/70 uppercase tracking-wide mb-2",
                "Request"
            }
            DocCodeGroup {
                group: CodeGroupNode { blocks: props.example.blocks.clone() }
            }
        }
    }
}

/// Props for DocResponseExample.
#[derive(Props, Clone, PartialEq)]
pub struct DocResponseExampleProps {
    /// The response example to render.
    pub example: ResponseExampleNode,
}

/// Container for API response examples with tabs.
#[component]
pub fn DocResponseExample(props: DocResponseExampleProps) -> Element {
    rsx! {
        div { class: "my-6",
            h4 { class: "text-sm font-semibold text-base-content/70 uppercase tracking-wide mb-2",
                "Response"
            }
            DocCodeGroup {
                group: CodeGroupNode { blocks: props.example.blocks.clone() }
            }
        }
    }
}
