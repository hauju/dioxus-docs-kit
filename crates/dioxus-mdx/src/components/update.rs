//! Update (changelog) entry component.

use dioxus::prelude::*;

use crate::components::DocNodeRenderer;
use crate::parser::UpdateNode;
use crate::text::slugify;

/// Props for DocUpdate component.
#[derive(Props, Clone, PartialEq)]
pub struct DocUpdateProps {
    /// The update entry to render.
    pub update: UpdateNode,
}

/// Changelog version update entry.
#[component]
pub fn DocUpdate(props: DocUpdateProps) -> Element {
    let update = &props.update;

    rsx! {
        div { class: "grid grid-cols-[120px_1fr] gap-x-8 py-8 border-b border-base-content/10 last:border-b-0 max-sm:grid-cols-1 max-sm:gap-y-3",
            // Left column: version badge + date
            div { class: "flex flex-col items-start gap-1.5",
                // Anchor for the `### {label}` heading the raw markdown
                // synthesises for the table of contents. `h-auto`/`py-1` let a
                // long label ("v0.3.x - v0.4.x") grow the pill instead of
                // spilling out of DaisyUI's fixed badge height.
                span {
                    id: slugify(&update.label),
                    class: "badge badge-primary font-mono h-auto whitespace-nowrap py-1",
                    "{update.label}"
                }
                if !update.description.is_empty() {
                    span { class: "text-xs text-base-content/50",
                        "{update.description}"
                    }
                }
            }
            // Right column: changelog content
            div {
                class: "prose prose-sm max-w-none",
                for (i, node) in update.content.iter().enumerate() {
                    DocNodeRenderer { key: "{i}", node: node.clone() }
                }
            }
        }
    }
}
