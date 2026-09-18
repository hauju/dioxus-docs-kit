//! Steps component for sequential documentation guides.

use dioxus::prelude::*;

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

/// Clean up step title by removing a redundant `Step N:` / `Step N.` prefix.
///
/// Hand-rolled rather than a regex (`^Step\s+\d+[:.]\s*`) so the renderer links
/// no regex engine: with parsing moved to build time this was the last one left
/// in the browser.
fn clean_step_title(title: &str) -> String {
    let Some(rest) = title.strip_prefix("Step") else {
        return title.trim().to_string();
    };
    let rest = rest.trim_start_matches([' ', '\t']);
    if rest.len() == title.len() - 4 {
        // No whitespace after "Step" — `\s+` requires at least one.
        return title.trim().to_string();
    }
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    let after = &rest[digits..];
    match (digits, after.as_bytes().first()) {
        (0, _) | (_, None) => title.trim().to_string(),
        (_, Some(b':' | b'.')) => after[1..].trim().to_string(),
        _ => title.trim().to_string(),
    }
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

#[cfg(test)]
mod tests {
    use super::clean_step_title;

    #[test]
    fn step_number_prefix_is_stripped() {
        assert_eq!(clean_step_title("Step 1: Install"), "Install");
        assert_eq!(clean_step_title("Step 12.  Install"), "Install");
        assert_eq!(clean_step_title("Step\t3: Install"), "Install");
    }

    #[test]
    fn other_titles_are_left_alone() {
        assert_eq!(clean_step_title("Install"), "Install");
        assert_eq!(clean_step_title("Stepping stones"), "Stepping stones");
        assert_eq!(clean_step_title("Step one"), "Step one");
        assert_eq!(clean_step_title("Step1: x"), "Step1: x");
        assert_eq!(clean_step_title("Step 1"), "Step 1");
    }
}
