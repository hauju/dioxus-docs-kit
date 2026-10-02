//! MDX content rendering: one Topcoat component per [`DocNode`] variant.
//!
//! The markup and Tailwind classes mirror `dioxus-mdx`'s components so the
//! two kits look the same. Everything renders on the server with no runtime
//! script: tabs are DaisyUI radio tabs, accordions and expandables are
//! `<details>`, and copy buttons use an inline `onclick`.

use dioxus_mdx::lucide::{
    LdChevronDown, LdChevronRight, LdCopy, LdInfo, LdLightbulb, LdTriangleAlert, icon_for_name,
};
use dioxus_mdx::{
    AccordionGroupNode, CalloutNode, CalloutType, CardNode, CodeBlockNode, DocNode, ExpandableNode,
    MERMAID_JS, ParamFieldNode, ResponseFieldNode, StepsNode, TabsNode, UpdateNode,
    clean_step_title, slugify,
};
use hl_lite::{Kind, Lang};
use topcoat::{
    Result,
    view::{BoxView, Unescaped, View, ViewExt, component, view},
};

use crate::icon::icon;
use crate::openapi::openapi_viewer;

/// Renders a page's parsed MDX nodes.
///
/// `base_path` is the docs mount point (e.g. `"/docs"`); relative card links
/// resolve against it. `key` must be unique among the `doc_content` calls on
/// one page: it namespaces the radio groups that drive tabs.
#[component]
pub async fn doc_content(
    nodes: &[DocNode],
    base_path: &str,
    #[default("c")] key: &str,
) -> Result<impl View> {
    Ok(view! {
        <div class="doc-content">
            doc_nodes(nodes: nodes, base_path: base_path, key: key)
        </div>
    })
}

/// A run of sibling nodes. Each child gets `<key>-<index>` as its own key, so
/// keys are unique across the whole tree.
#[component]
async fn doc_nodes(nodes: &[DocNode], base_path: &str, key: &str) -> Result<impl View> {
    Ok(view! {
        #[key(i)]
        for (i, node) in nodes.iter().enumerate() {
            let child_key = format!("{key}-{i}");
            doc_node(node: node, base_path: base_path, key: &child_key)
        }
    })
}

#[component]
async fn doc_node(node: &DocNode, base_path: &str, key: &str) -> Result<BoxView<'_>> {
    Ok(view! {
        match node {
            // Rendered from the author's Markdown at build time: trusted.
            DocNode::Html(html) => <div class="prose-content">(Unescaped::new_unchecked(html.clone()))</div>,
            DocNode::Callout(callout) => callout_box(callout: callout),
            DocNode::Card(card) => card_grid(cards: std::slice::from_ref(card), cols: 1, base_path: base_path),
            DocNode::CardGroup(group) => card_grid(cards: &group.cards, cols: group.cols, base_path: base_path),
            DocNode::Tabs(tabs) => tabs_box(tabs: tabs, base_path: base_path, key: key),
            DocNode::Steps(steps) => steps_list(steps: steps, base_path: base_path, key: key),
            DocNode::AccordionGroup(group) => accordion_group(group: group, base_path: base_path, key: key),
            DocNode::CodeBlock(block) if is_mermaid(block) => mermaid_diagram(code: &block.code),
            DocNode::CodeBlock(block) => code_block(block: block),
            DocNode::CodeGroup(group) => code_group(blocks: &group.blocks, key: key),
            DocNode::ParamField(field) => param_field(field: field, base_path: base_path, key: key),
            DocNode::ResponseField(field) => response_field(field: field, depth: 0),
            DocNode::Expandable(expandable) => expandable_fields(expandable: expandable, depth: 0),
            DocNode::RequestExample(example) => labeled_code_group(label: "Request", blocks: &example.blocks, key: key),
            DocNode::ResponseExample(example) => labeled_code_group(label: "Response", blocks: &example.blocks, key: key),
            DocNode::Update(update) => update_entry(update: update, base_path: base_path, key: key),
            DocNode::OpenApi(node) => openapi_viewer(node: node),
            // `DocNode` is non-exhaustive.
            _ => "",
        }
    }
    // Recursive through the container nodes above.
    .boxed())
}

#[component]
async fn callout_box(callout: &CalloutNode) -> Result<impl View> {
    let (tone, glyph) = match callout.callout_type {
        CalloutType::Tip => (
            "bg-success/5 border-success/40 shadow-success/5",
            LdLightbulb,
        ),
        CalloutType::Warning => (
            "bg-warning/5 border-warning/40 shadow-warning/5",
            LdTriangleAlert,
        ),
        _ => ("bg-info/5 border-info/40 shadow-info/5", LdInfo),
    };
    let text = match callout.callout_type {
        CalloutType::Tip => "text-success",
        CalloutType::Warning => "text-warning",
        _ => "text-info",
    };
    Ok(view! {
        <div class=(format!("my-6 px-4 py-4 rounded-lg border-l-4 shadow-sm {tone}")) role="alert">
            <div class="flex gap-4">
                <div class=(format!("{text} mt-0.5 shrink-0"))>icon(glyph: glyph, class: "size-5")</div>
                <div class="flex-1 min-w-0">
                    <span class=(format!("font-semibold {text} text-sm uppercase tracking-wide"))>
                        (callout.callout_type.as_str())
                    </span>
                    <div class="prose prose-sm max-w-none text-base-content/85 mt-1.5 [&>p:first-child]:mt-0 [&>p:last-child]:mb-0">
                        (Unescaped::new_unchecked(callout.content_html.clone()))
                    </div>
                </div>
            </div>
        </div>
    })
}

#[component]
async fn card_grid(cards: &[CardNode], cols: u8, base_path: &str) -> Result<impl View> {
    let grid = match cols {
        1 => "grid grid-cols-1 gap-4",
        2 => "grid grid-cols-1 md:grid-cols-2 gap-4",
        3 => "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
        _ => "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4",
    };
    Ok(view! {
        <div class=(format!("my-6 {grid}"))>
            #[key(i)]
            for (i, card) in cards.iter().enumerate() {
                card_item(card: card, base_path: base_path)
            }
        </div>
    })
}

#[component]
async fn card_item(card: &CardNode, base_path: &str) -> Result<impl View> {
    let body = view! {
        <div class="bg-base-300 hover:border-primary/50 transition-colors duration-150 border border-base-content/10 rounded-lg h-full">
            <div class="p-6">
                if let Some(name) = &card.icon {
                    <div class="text-primary mb-5">icon(glyph: icon_for_name(name), class: "size-6")</div>
                }
                <h3 class="font-semibold text-base-content mb-2 no-underline">(card.title.as_str())</h3>
                if !card.content_html.is_empty() {
                    <div class="text-sm text-base-content/60 leading-relaxed [&>p]:my-0 [&_a]:no-underline [&_a]:text-base-content/60">
                        (Unescaped::new_unchecked(card.content_html.clone()))
                    </div>
                }
            </div>
        </div>
    };
    Ok(view! {
        match card.href.as_deref() {
            Some(href) if href.starts_with("http://") || href.starts_with("https://") => {
                <a href=(href) target="_blank" rel="noopener noreferrer" class="block no-underline hover:no-underline not-prose">(body)</a>
            },
            Some(href) => {
                <a href=(doc_href(href, base_path)) class="block no-underline hover:no-underline not-prose">(body)</a>
            },
            None => (body),
        }
    })
}

/// Resolve a card's site-relative doc link against the docs mount point.
fn doc_href(href: &str, base_path: &str) -> String {
    if href.starts_with(base_path) {
        return href.to_string();
    }
    format!("{}/{}", base_path, href.trim_start_matches('/'))
}

#[component]
async fn tabs_box(tabs: &TabsNode, base_path: &str, key: &str) -> Result<impl View> {
    let group = format!("dk-tabs-{key}");
    Ok(view! {
        <div class="my-6 tabs tabs-border">
            #[key(i)]
            for (i, tab) in tabs.tabs.iter().enumerate() {
                <input
                    type="radio"
                    name=(group.as_str())
                    class="tab"
                    id=(slugify(&tab.title))
                    aria-label=(tab.title.as_str())
                    checked=(i == 0)
                >
                <div class="tab-content mt-4 p-4 bg-base-200/50 rounded-lg border border-base-content/5">
                    let tab_key = format!("{key}-{i}");
                    <div class="doc-tab-content">
                        doc_nodes(nodes: &tab.content, base_path: base_path, key: &tab_key)
                    </div>
                </div>
            }
        </div>
    })
}

#[component]
async fn steps_list(steps: &StepsNode, base_path: &str, key: &str) -> Result<impl View> {
    Ok(view! {
        <div class="my-8">
            <div class="relative border-l-2 border-primary/20 ml-5 space-y-8">
                #[key(i)]
                for (i, step) in steps.steps.iter().enumerate() {
                    let step_key = format!("{key}-{i}");
                    <div class="relative pl-10">
                        <span class="absolute left-0 top-0 -translate-x-1/2 flex items-center justify-center w-7 h-7 bg-primary text-primary-content rounded-full font-semibold text-sm shadow-sm">
                            (i + 1)
                        </span>
                        <div>
                            <h4 class="font-semibold text-base text-base-content mb-2">(clean_step_title(&step.title))</h4>
                            <div class="prose prose-sm max-w-none">
                                doc_nodes(nodes: &step.content, base_path: base_path, key: &step_key)
                            </div>
                        </div>
                    </div>
                }
            </div>
        </div>
    })
}

#[component]
async fn accordion_group(
    group: &AccordionGroupNode,
    base_path: &str,
    key: &str,
) -> Result<impl View> {
    Ok(view! {
        <div class="my-6 space-y-3">
            #[key(i)]
            for (i, item) in group.items.iter().enumerate() {
                let item_key = format!("{key}-{i}");
                <details class="group border border-base-content/10 rounded-lg overflow-hidden hover:border-base-content/20 open:border-base-content/15 open:shadow-sm transition-colors">
                    <summary class="w-full flex items-center gap-3 px-4 py-3.5 text-left cursor-pointer list-none [&::-webkit-details-marker]:hidden hover:bg-base-200/30 group-open:bg-base-200/50 transition-colors">
                        if let Some(name) = &item.icon {
                            <div class="text-primary shrink-0">icon(glyph: icon_for_name(name), class: "size-5")</div>
                        }
                        <span id=(slugify(&item.title)) class="flex-1 font-medium text-base-content">(item.title.as_str())</span>
                        icon(glyph: LdChevronDown, class: "size-5 text-base-content/50 transition-transform duration-200 group-open:rotate-180")
                    </summary>
                    <div class="px-4 pb-4 border-t border-base-content/10 bg-base-200/30">
                        <div class="prose prose-sm max-w-none pt-4">
                            doc_nodes(nodes: &item.content, base_path: base_path, key: &item_key)
                        </div>
                    </div>
                </details>
            }
        </div>
    })
}

#[component]
async fn param_field(field: &ParamFieldNode, base_path: &str, key: &str) -> Result<impl View> {
    Ok(view! {
        <div class="border-b border-base-300 py-4 first:pt-0 last:border-b-0">
            <div class="flex items-center gap-3 flex-wrap">
                <code class="font-mono font-semibold text-primary">(field.name.as_str())</code>
                <span class="text-xs px-2 py-0.5 rounded-full bg-base-300 text-base-content/70">(field.param_type.as_str())</span>
                if field.required {
                    <span class="text-xs px-2 py-0.5 rounded-full bg-error/20 text-error">"required"</span>
                }
                if let Some(default) = &field.default {
                    <span class="text-xs px-2 py-0.5 rounded-full bg-base-300 text-base-content/70 font-mono">
                        "default:" <span class="text-primary">"\"" (default.as_str()) "\""</span>
                    </span>
                }
            </div>
            if !field.content.is_empty() {
                <div class="mt-2 text-base-content/70">
                    doc_nodes(nodes: &field.content, base_path: base_path, key: key)
                </div>
            }
        </div>
    })
}

#[component]
async fn response_field(field: &ResponseFieldNode, depth: usize) -> Result<BoxView<'_>> {
    let indent = if depth > 0 {
        "py-3 ml-4 border-l-2 border-base-300 pl-4"
    } else {
        "py-3"
    };
    Ok(view! {
        <div class=(indent)>
            <div class="flex items-start gap-2 flex-wrap">
                <code class="font-mono font-semibold text-base-content bg-base-300 px-2 py-0.5 rounded">(field.name.as_str())</code>
                <span class="badge badge-outline badge-sm">(field.field_type.as_str())</span>
                if field.required {
                    <span class="badge badge-success badge-sm">"required"</span>
                }
            </div>
            if !field.content_html.is_empty() {
                <div class="prose prose-sm max-w-none mt-2 text-base-content/80">
                    (Unescaped::new_unchecked(field.content_html.clone()))
                </div>
            }
            if let Some(expandable) = &field.expandable {
                expandable_fields(expandable: expandable, depth: depth + 1)
            }
        </div>
    }
    // Recursive through `expandable_fields`.
    .boxed())
}

#[component]
async fn expandable_fields(expandable: &ExpandableNode, depth: usize) -> Result<impl View> {
    Ok(view! {
        <details class="group mt-3 border border-base-300 rounded-lg overflow-hidden">
            <summary class="w-full flex items-center gap-2 px-3 py-2 text-left cursor-pointer list-none [&::-webkit-details-marker]:hidden hover:bg-base-200 transition-colors text-sm">
                icon(glyph: LdChevronRight, class: "size-4 text-base-content/50 transition-transform group-open:rotate-90")
                <span class="font-medium text-base-content/70">(expandable.title.as_str())</span>
            </summary>
            <div class="px-3 pb-3 border-t border-base-300 bg-base-200/30">
                #[key(i)]
                for (i, field) in expandable.fields.iter().enumerate() {
                    response_field(field: field, depth: depth)
                }
            </div>
        </details>
    })
}

#[component]
async fn labeled_code_group(label: &str, blocks: &[CodeBlockNode], key: &str) -> Result<impl View> {
    Ok(view! {
        <div class="my-6">
            <h4 class="text-sm font-semibold text-base-content/70 uppercase tracking-wide mb-2">(label)</h4>
            code_group(blocks: blocks, key: key)
        </div>
    })
}

#[component]
async fn update_entry(update: &UpdateNode, base_path: &str, key: &str) -> Result<impl View> {
    Ok(view! {
        <div class="grid grid-cols-[120px_1fr] gap-x-8 py-8 border-b border-base-content/10 last:border-b-0 max-sm:grid-cols-1 max-sm:gap-y-3">
            <div class="flex flex-col items-start gap-1.5">
                <span id=(slugify(&update.label)) class="badge badge-primary font-mono h-auto whitespace-nowrap py-1">(update.label.as_str())</span>
                if !update.description.is_empty() {
                    <span class="text-xs text-base-content/50">(update.description.as_str())</span>
                }
            </div>
            <div class="prose prose-sm max-w-none">
                doc_nodes(nodes: &update.content, base_path: base_path, key: key)
            </div>
        </div>
    })
}

// ============================================================================
// Code
// ============================================================================

/// A ```` ```mermaid ```` fence, when the `mermaid` feature is on.
fn is_mermaid(block: &CodeBlockNode) -> bool {
    cfg!(feature = "mermaid") && block.language.as_deref().map(str::trim) == Some("mermaid")
}

/// A diagram the browser renders: the shared `MERMAID_JS` driver loads
/// mermaid.js from jsDelivr and turns every `pre.mermaid` into an SVG that
/// follows the light/dark theme. The script repeats per diagram but runs once,
/// after the document has parsed, so it sees every diagram on the page.
#[component]
async fn mermaid_diagram(code: &str) -> Result<impl View> {
    let script = format!(
        "(function(){{if(window.__dkMermaid)return;window.__dkMermaid=1;function go(){{{MERMAID_JS}}}if(document.readyState==='loading')document.addEventListener('DOMContentLoaded',go);else go()}})()"
    );
    Ok(view! {
        <div class="not-prose my-6 flex justify-center">
            <pre class="mermaid">(code)</pre>
        </div>
        // Static, trusted source.
        <script>(Unescaped::new_unchecked(script))</script>
    })
}

#[component]
async fn code_block(block: &CodeBlockNode) -> Result<impl View> {
    let label = block.filename.as_deref().or(block.language.as_deref());
    Ok(view! {
        <div class="dk-code-block not-prose my-6 relative group inline-block max-w-full rounded-lg border border-base-content/10 overflow-hidden">
            match label {
                Some(label) => {
                    <div class="flex items-center justify-between bg-base-200/80 px-4 py-2.5 border-b border-base-content/10 text-sm">
                        <span class="text-base-content/60 font-mono text-xs tracking-wide">(label)</span>
                        copy_button(code: &block.code)
                    </div>
                    <div class="dk-code-block-body bg-base-200">
                        highlighted(block: block)
                    </div>
                },
                None => {
                    <div class="dk-code-block-body dk-code-block-body--bare bg-base-200 relative">
                        highlighted(block: block)
                        <div class="absolute top-3 right-3">copy_button(code: &block.code)</div>
                    </div>
                },
            }
        </div>
    })
}

/// A set of code blocks behind DaisyUI radio tabs, one tab per block.
#[component]
async fn code_group(blocks: &[CodeBlockNode], key: &str) -> Result<impl View> {
    let group = format!("dk-code-{key}");
    Ok(view! {
        <div class="dk-code-block not-prose my-6 max-w-full rounded-lg border border-base-content/10 overflow-hidden">
            <div class="tabs tabs-border bg-base-200/80">
                #[key(i)]
                for (i, block) in blocks.iter().enumerate() {
                    let label = block.filename.as_deref().or(block.language.as_deref()).unwrap_or("Code");
                    <input type="radio" name=(group.as_str()) class="tab" aria-label=(label) checked=(i == 0)>
                    <div class="tab-content dk-code-group-block bg-base-200 relative group border-t border-base-content/10">
                        highlighted(block: block)
                        <div class="absolute top-3 right-3">copy_button(code: &block.code)</div>
                    </div>
                }
            </div>
        </div>
    })
}

/// `<pre class="dk-code">` with one `hl-*` span per token; colors come from the
/// `--dk-hl-*` CSS tokens. An unknown language renders the same markup, unspanned.
#[component]
pub(crate) async fn highlighted(block: &CodeBlockNode) -> Result<impl View> {
    let spans: Vec<(Option<String>, &str)> = match code_language(block) {
        Some(lang) => hl_lite::highlight(lang, &block.code)
            .into_iter()
            .map(|span| {
                (
                    (span.kind != Kind::Plain).then(|| format!("hl-{}", span.kind.class())),
                    span.text,
                )
            })
            .collect(),
        None => vec![(None, block.code.as_str())],
    };
    Ok(view! {
        <pre
            class="dk-code"
            style="margin:0;padding:1rem;overflow:auto;tab-size:4;font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;font-size:0.9rem;line-height:1.55"
        >
            <code>
                for (class, text) in spans {
                    match class {
                        Some(class) => <span class=(class)>(text)</span>,
                        None => (text),
                    }
                }
            </code>
        </pre>
    })
}

/// Same resolution as `dioxus-mdx`: the fence slug wins, the filename is the fallback.
fn code_language(block: &CodeBlockNode) -> Option<Lang> {
    block
        .language
        .as_deref()
        .map(str::trim)
        .and_then(Lang::from_slug)
        .or_else(|| block.filename.as_deref().and_then(Lang::from_path))
}

#[component]
async fn copy_button(code: &str) -> Result<impl View> {
    Ok(view! {
        <button
            type="button"
            class="btn btn-ghost btn-xs opacity-60 hover:opacity-100 group-hover:opacity-100 transition-all duration-150 hover:bg-base-content/10"
            aria-label="Copy code"
            data-code=(code)
            onclick="navigator.clipboard.writeText(this.dataset.code)"
        >
            icon(glyph: LdCopy, class: "size-4")
        </button>
    })
}
