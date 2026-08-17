//! Rust API reference page — renders one item from a distilled rustdoc model.

use dioxus::prelude::*;

use crate::parser::{CodeBlockNode, RustApiItem, RustApiMember, RustMemberKind, parse_mdx};

use super::code::DocCodeBlock;

/// Props for [`RustApiItemPage`].
#[derive(Props, Clone, PartialEq)]
pub struct RustApiItemPageProps {
    /// The item to display.
    pub item: RustApiItem,
    /// Crate name for the breadcrumb (`dioxus_docs_kit`).
    pub crate_name: String,
}

/// Full page for a single Rust API item: kind badge and module breadcrumb,
/// declaration, documentation, implemented traits, and member sections
/// (variants, fields, methods, associated items).
///
/// # Stable public classes
///
/// Uses `dk-rust-item`, `dk-rust-item-header`, `dk-rust-member`, and
/// `dk-rust-member-docs` for consumer CSS hooks.
#[component]
pub fn RustApiItemPage(props: RustApiItemPageProps) -> Element {
    let item = &props.item;

    let module_path = if item.path.is_empty() {
        props.crate_name.clone()
    } else {
        format!("{}::{}", props.crate_name, item.path.join("::"))
    };

    let signature_block = CodeBlockNode {
        language: Some("rust".to_string()),
        code: item.signature.clone(),
        filename: None,
    };

    let kind_badge = item.kind.badge_class();

    rsx! {
        article { class: "dk-rust-item max-w-3xl mx-auto",
            header { class: "dk-rust-item-header mb-6",
                div { class: "flex items-center gap-3 mb-3",
                    span { class: "badge badge-sm font-semibold {kind_badge}",
                        "{item.kind.label()}"
                    }
                    code { class: "text-sm text-base-content/60 font-mono", "{module_path}" }
                }
                h1 { class: "dk-article-title text-4xl font-bold tracking-tight font-mono",
                    "{item.name}"
                }
            }

            DocCodeBlock { block: signature_block }

            if let Some(docs) = &item.docs {
                // Rustdoc convention starts doc headings at h1 ("# Example"),
                // so h1/h2 are scaled down to section size here.
                div { class: "dk-rust-item-docs prose prose-base max-w-none mt-6
                    prose-h1:text-2xl prose-h1:font-semibold prose-h1:mt-8 prose-h1:mb-3
                    prose-h2:text-xl prose-h2:font-semibold prose-h2:mt-6 prose-h2:mb-2
                    prose-p:text-base-content/80 prose-p:leading-relaxed
                    prose-a:text-primary prose-a:no-underline hover:prose-a:underline
                    prose-code:bg-base-200 prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:text-sm",
                    DocContentBlock { markdown: docs.clone() }
                }
            }

            if !item.trait_impls.is_empty() {
                section { class: "mt-8",
                    h2 { class: "text-lg font-semibold mb-3", "Implements" }
                    div { class: "flex flex-wrap gap-2",
                        for trait_name in item.trait_impls.iter() {
                            code { class: "badge badge-ghost badge-lg font-mono text-xs",
                                "{trait_name}"
                            }
                        }
                    }
                }
            }

            for kind in RustMemberKind::ALL {
                {member_section(item, kind)}
            }
        }
    }
}

/// One member section ("Methods", "Fields", …), or nothing when empty.
fn member_section(item: &RustApiItem, kind: RustMemberKind) -> Element {
    let members = item.members_of(kind);
    if members.is_empty() {
        return rsx! {};
    }
    let section_id = kind.section_label().to_lowercase().replace(' ', "-");
    rsx! {
        section { class: "mt-10",
            h2 { id: "{section_id}", class: "text-2xl font-semibold mb-4 scroll-mt-24",
                "{kind.section_label()}"
            }
            for member in members {
                MemberCard { member: member.clone() }
            }
        }
    }
}

/// A single member: monospace signature with anchored name, docs below.
#[component]
fn MemberCard(member: RustApiMember) -> Element {
    rsx! {
        div { id: "member.{member.name}", class: "dk-rust-member mb-5 scroll-mt-24",
            div { class: "rounded-lg border border-base-300 bg-base-200/50 px-4 py-3 overflow-x-auto",
                code { class: "font-mono text-sm whitespace-pre", "{member.signature}" }
            }
            if let Some(docs) = &member.docs {
                // Member docs demote every rustdoc heading ("# Panics") to
                // label size so they don't compete with the page structure.
                div { class: "dk-rust-member-docs prose prose-sm max-w-none mt-2 px-1
                    prose-headings:text-sm prose-headings:font-semibold prose-headings:mt-3 prose-headings:mb-1
                    prose-p:text-base-content/70
                    prose-code:bg-base-200 prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:text-xs",
                    DocContentBlock { markdown: docs.clone() }
                }
            }
        }
    }
}

/// Parse a markdown string and render it with the standard node renderer.
#[component]
fn DocContentBlock(markdown: String) -> Element {
    let nodes = parse_mdx(&markdown);
    rsx! {
        super::renderer::DocContent { nodes }
    }
}
