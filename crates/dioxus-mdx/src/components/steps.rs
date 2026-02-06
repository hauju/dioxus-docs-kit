//! Steps component for sequential documentation guides.

use dioxus::prelude::*;
use regex::Regex;

use crate::components::DocNodeRenderer;
use crate::parser::{DocNode, StepsNode};

/// Props for DocSteps component.
#[derive(Props, Clone, PartialEq)]
pub struct DocStepsProps {
    /// Steps data.
    pub steps: StepsNode,
}

/// Sequential steps component using DaisyUI steps or custom styling.
#[component]
pub fn DocSteps(props: DocStepsProps) -> Element {
    rsx! {
        div { class: "my-8",
            // Use div instead of ol to avoid default list numbering
            div { class: "relative border-l-2 border-primary/20 ml-5 space-y-8",
                for (i, step) in props.steps.steps.iter().enumerate() {
                    div { key: "{i}", class: "relative pl-10",
                        // Step number circle - positioned to overlap the border line
                        span {
                            class: "absolute left-0 top-0 -translate-x-1/2 flex items-center justify-center w-7 h-7 bg-primary text-primary-content rounded-full font-semibold text-sm shadow-sm",
                            "{i + 1}"
                        }
                        // Step content
                        div {
                            // Step title
                            h4 { class: "font-semibold text-base text-base-content mb-2",
                                // Clean up step title (remove "Step X:" prefix if present)
                                {clean_step_title(&step.title)}
                            }
                            // Step body (render as markdown with nested components)
                            StepContent { content: step.content.clone() }
                        }
                    }
                }
            }
        }
    }
}

/// Clean up step title by removing redundant prefixes.
fn clean_step_title(title: &str) -> String {
    // Remove "Step N:" or "Step N." prefix
    let re = Regex::new(r"^Step\s+\d+[:.]\s*").unwrap();
    re.replace(title, "").trim().to_string()
}

/// Props for StepContent.
#[derive(Props, Clone, PartialEq)]
struct StepContentProps {
    content: Vec<DocNode>,
}

/// Render step content which may contain nested MDX components.
#[component]
fn StepContent(props: StepContentProps) -> Element {
    rsx! {
        div { class: "prose prose-sm max-w-none",
            for (i, node) in props.content.iter().enumerate() {
                DocNodeRenderer { key: "{i}", node: node.clone() }
            }
        }
    }
}
