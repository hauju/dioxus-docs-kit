//! Tabs component for documentation.

use dioxus::prelude::*;

use crate::components::DocNodeRenderer;
use crate::parser::{DocNode, TabsNode};

/// Props for DocTabs component.
#[derive(Props, Clone, PartialEq)]
pub struct DocTabsProps {
    /// Tabs data.
    pub tabs: TabsNode,
}

/// Tabbed content component using DaisyUI tabs.
#[component]
pub fn DocTabs(props: DocTabsProps) -> Element {
    let mut active_tab = use_signal(|| 0usize);

    rsx! {
        div { class: "my-6",
            // Tab headers - refined underline style
            div { class: "flex border-b border-base-content/10 mb-4",
                for (i, tab) in props.tabs.tabs.iter().enumerate() {
                    button {
                        key: "{i}",
                        class: if active_tab() == i {
                            "px-4 py-2.5 text-sm font-medium text-primary border-b-2 border-primary -mb-px transition-colors"
                        } else {
                            "px-4 py-2.5 text-sm font-medium text-base-content/60 hover:text-base-content border-b-2 border-transparent -mb-px transition-colors"
                        },
                        onclick: move |_| active_tab.set(i),
                        "{tab.title}"
                    }
                }
            }

            // Tab content - cleaner without heavy background
            div { class: "p-4 bg-base-200/50 rounded-lg border border-base-content/5",
                if let Some(tab) = props.tabs.tabs.get(active_tab()) {
                    TabContent { content: tab.content.clone() }
                }
            }
        }
    }
}

/// Props for TabContent.
#[derive(Props, Clone, PartialEq)]
struct TabContentProps {
    content: Vec<DocNode>,
}

/// Individual tab content renderer.
#[component]
fn TabContent(props: TabContentProps) -> Element {
    rsx! {
        div {
            class: "doc-tab-content",
            for (i, node) in props.content.iter().enumerate() {
                DocNodeRenderer { key: "{i}", node: node.clone() }
            }
        }
    }
}
