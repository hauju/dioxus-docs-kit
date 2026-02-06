//! Accordion component for collapsible documentation sections.

use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::ld_icons::*};

use crate::components::{DocNodeRenderer, MdxIcon};
use crate::parser::{AccordionGroupNode, DocNode};

/// Props for DocAccordionGroup component.
#[derive(Props, Clone, PartialEq)]
pub struct DocAccordionGroupProps {
    /// Accordion group data.
    pub group: AccordionGroupNode,
}

/// Accordion group component with collapsible sections.
#[component]
pub fn DocAccordionGroup(props: DocAccordionGroupProps) -> Element {
    rsx! {
        div { class: "my-6 space-y-3",
            for (i, item) in props.group.items.iter().enumerate() {
                DocAccordionItem {
                    key: "{i}",
                    title: item.title.clone(),
                    icon: item.icon.clone(),
                    content: item.content.clone(),
                }
            }
        }
    }
}

/// Props for DocAccordionItem.
#[derive(Props, Clone, PartialEq)]
pub struct DocAccordionItemProps {
    title: String,
    icon: Option<String>,
    content: Vec<DocNode>,
}

/// Single accordion item.
#[component]
pub fn DocAccordionItem(props: DocAccordionItemProps) -> Element {
    let mut expanded = use_signal(|| false);

    rsx! {
        div {
            class: if expanded() {
                "border border-base-content/15 rounded-lg overflow-hidden shadow-sm"
            } else {
                "border border-base-content/10 rounded-lg overflow-hidden hover:border-base-content/20 transition-colors"
            },
            // Header (clickable)
            button {
                class: if expanded() {
                    "w-full flex items-center gap-3 px-4 py-3.5 text-left bg-base-200/50 transition-colors"
                } else {
                    "w-full flex items-center gap-3 px-4 py-3.5 text-left hover:bg-base-200/30 transition-colors"
                },
                onclick: move |_| expanded.set(!expanded()),
                // Icon (if provided)
                if let Some(icon) = &props.icon {
                    div { class: "text-primary shrink-0",
                        MdxIcon { name: icon.clone(), class: "size-5".to_string() }
                    }
                }
                // Title
                span { class: "flex-1 font-medium text-base-content",
                    "{props.title}"
                }
                // Expand/collapse indicator with smooth rotation
                Icon {
                    class: if expanded() {
                        "size-5 text-base-content/50 transform rotate-180 transition-transform duration-200"
                    } else {
                        "size-5 text-base-content/50 transition-transform duration-200"
                    },
                    icon: LdChevronDown
                }
            }

            // Content (collapsible) with animation class
            if expanded() {
                div { class: "px-4 pb-4 border-t border-base-content/10 bg-base-200/30 accordion-content-enter",
                    AccordionContent { content: props.content.clone() }
                }
            }
        }
    }
}

/// Props for AccordionContent.
#[derive(Props, Clone, PartialEq)]
struct AccordionContentProps {
    content: Vec<DocNode>,
}

/// Render accordion content which may contain nested MDX components.
#[component]
fn AccordionContent(props: AccordionContentProps) -> Element {
    rsx! {
        div { class: "prose prose-sm max-w-none pt-4",
            for (i, node) in props.content.iter().enumerate() {
                DocNodeRenderer { key: "{i}", node: node.clone() }
            }
        }
    }
}
