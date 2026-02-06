//! Main documentation renderer component.

use dioxus::prelude::*;

use super::slugify;
use crate::components::{
    DocAccordionGroup, DocCallout, DocCardGroup, DocCodeBlock, DocCodeGroup, DocExpandable,
    DocParamField, DocRequestExample, DocResponseExample, DocResponseField, DocSteps, DocTabs,
    DocUpdate, OpenApiViewer,
};
use crate::parser::{parse_mdx, CardGroupNode, DocNode};

/// Inject `id` attributes into heading tags so TOC anchor links work.
fn inject_heading_ids(html: &str) -> String {
    let re = regex::Regex::new(r"<(h[2-4])>(.*?)</h[2-4]>").unwrap();
    re.replace_all(html, |caps: &regex::Captures| {
        let tag = &caps[1];
        let inner = &caps[2];
        // Strip any inner HTML tags to get plain text for the slug
        let plain = regex::Regex::new(r"<[^>]+>")
            .unwrap()
            .replace_all(inner, "");
        let id = slugify(&plain);
        format!("<{tag} id=\"{id}\">{inner}</{tag}>")
    })
    .into_owned()
}

/// Props for DocNodeRenderer component.
#[derive(Props, Clone, PartialEq)]
pub struct DocNodeRendererProps {
    /// The DocNode to render.
    pub node: DocNode,
}

/// Render a single DocNode.
#[component]
pub fn DocNodeRenderer(props: DocNodeRendererProps) -> Element {
    match &props.node {
        DocNode::Markdown(md) => {
            let html = markdown::to_html_with_options(md, &markdown::Options::gfm())
                .unwrap_or_else(|_| md.clone());
            let html = inject_heading_ids(&html);
            rsx! {
                div {
                    class: "prose-content",
                    dangerous_inner_html: html,
                }
            }
        }
        DocNode::Callout(callout) => {
            rsx! {
                DocCallout {
                    callout_type: callout.callout_type,
                    content: callout.content.clone(),
                }
            }
        }
        DocNode::Card(card) => {
            // Wrap single card in a group
            rsx! {
                DocCardGroup {
                    group: CardGroupNode {
                        cols: 1,
                        cards: vec![card.clone()],
                    }
                }
            }
        }
        DocNode::CardGroup(group) => {
            rsx! {
                DocCardGroup { group: group.clone() }
            }
        }
        DocNode::Tabs(tabs) => {
            rsx! {
                DocTabs { tabs: tabs.clone() }
            }
        }
        DocNode::Steps(steps) => {
            rsx! {
                DocSteps { steps: steps.clone() }
            }
        }
        DocNode::AccordionGroup(group) => {
            rsx! {
                DocAccordionGroup { group: group.clone() }
            }
        }
        DocNode::CodeBlock(block) => {
            rsx! {
                DocCodeBlock { block: block.clone() }
            }
        }
        DocNode::CodeGroup(group) => {
            rsx! {
                DocCodeGroup { group: group.clone() }
            }
        }
        DocNode::ParamField(field) => {
            rsx! {
                DocParamField { field: field.clone() }
            }
        }
        DocNode::ResponseField(field) => {
            rsx! {
                DocResponseField { field: field.clone() }
            }
        }
        DocNode::Expandable(expandable) => {
            rsx! {
                DocExpandable { expandable: expandable.clone() }
            }
        }
        DocNode::RequestExample(example) => {
            rsx! {
                DocRequestExample { example: example.clone() }
            }
        }
        DocNode::ResponseExample(example) => {
            rsx! {
                DocResponseExample { example: example.clone() }
            }
        }
        DocNode::Update(update) => {
            rsx! {
                DocUpdate { update: update.clone() }
            }
        }
        DocNode::OpenApi(openapi) => {
            rsx! {
                OpenApiViewer {
                    spec: openapi.spec.clone(),
                    tags: openapi.tags.clone(),
                    show_schemas: openapi.show_schemas,
                }
            }
        }
    }
}

/// Props for DocContent component.
#[derive(Props, Clone, PartialEq)]
pub struct DocContentProps {
    /// Parsed documentation nodes.
    pub nodes: Vec<DocNode>,
}

/// Render a list of DocNodes.
#[component]
pub fn DocContent(props: DocContentProps) -> Element {
    rsx! {
        div { class: "doc-content",
            for (i, node) in props.nodes.iter().enumerate() {
                DocNodeRenderer { key: "{i}", node: node.clone() }
            }
        }
    }
}

/// Props for MdxContent component.
#[derive(Props, Clone, PartialEq)]
pub struct MdxContentProps {
    /// Raw MDX content to parse and render.
    pub content: String,
}

/// Parse and render MDX content.
///
/// This is the main entry point for rendering MDX in Dioxus applications.
///
/// # Example
///
/// ```rust,ignore
/// use dioxus::prelude::*;
/// use dioxus_mdx::MdxContent;
///
/// #[component]
/// fn DocsPage(content: String) -> Element {
///     rsx! {
///         MdxContent { content }
///     }
/// }
/// ```
#[component]
pub fn MdxContent(props: MdxContentProps) -> Element {
    let nodes = parse_mdx(&props.content);

    rsx! {
        DocContent { nodes: nodes }
    }
}

/// Parse and render MDX content (legacy alias).
#[component]
pub fn MdxRenderer(content: String) -> Element {
    let nodes = parse_mdx(&content);

    rsx! {
        DocContent { nodes: nodes }
    }
}
