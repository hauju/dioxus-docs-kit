//! Code block components for documentation.
//!
//! Syntax highlighting comes from [`hl_lite`]: the code is lexed into spans and
//! rendered as `<span class="hl-{kind}">`, so the colors are a stylesheet
//! concern (`--dk-hl-*` in the docs kit), not a Rust one.

use crate::lucide::*;
use dioxus::prelude::*;
use hl_lite::{Kind, Lang};

#[cfg(feature = "mermaid")]
use super::mermaid::MermaidDiagram;
use crate::parser::{CodeBlockNode, CodeGroupNode};

/// Props for DocCodeBlock component.
#[derive(Props, Clone, PartialEq)]
pub struct DocCodeBlockProps {
    /// Code block data.
    pub block: CodeBlockNode,
}

/// Single code block with syntax highlighting and copy button.
#[component]
pub fn DocCodeBlock(props: DocCodeBlockProps) -> Element {
    // Mermaid blocks are rendered as diagrams, not syntax-highlighted code
    #[cfg(feature = "mermaid")]
    if props.block.language.as_deref() == Some("mermaid") {
        return rsx! { MermaidDiagram { code: props.block.code.clone() } };
    }

    let copied = use_signal(|| false);
    let code = props.block.code.clone();
    let code_for_copy = code.clone();

    rsx! {
        // `not-prose` opts the whole block out of Tailwind Typography: a consumer's
        // `prose-code:*` / `prose-pre:*` utilities otherwise target the inner
        // `<pre class="dk-code"><code>`, painting the inline-code pill background
        // as a box behind every wrapped line.
        div { class: "dk-code-block not-prose my-6 relative group inline-block max-w-full rounded-lg border border-base-content/10 overflow-hidden",
            // Language label and filename - refined header
            if props.block.language.is_some() || props.block.filename.is_some() {
                div { class: "flex items-center justify-between bg-base-200/80 px-4 py-2.5 border-b border-base-content/10 text-sm",
                    span { class: "text-base-content/60 font-mono text-xs tracking-wide",
                        if let Some(filename) = &props.block.filename {
                            "{filename}"
                        } else if let Some(lang) = &props.block.language {
                            "{lang}"
                        }
                    }
                    // Copy button - always visible with subtle opacity
                    CopyButton {
                        code: code_for_copy.clone(),
                        copied: copied,
                    }
                }
            }

            // Code content with syntax highlighting
            div {
                class: if props.block.language.is_some() || props.block.filename.is_some() {
                    "dk-code-block-body bg-base-200"
                } else {
                    "dk-code-block-body dk-code-block-body--bare bg-base-200 relative"
                },
                HighlightedCode {
                    code: code.clone(),
                    language: props.block.language.clone(),
                    filename: props.block.filename.clone(),
                }
                // Copy button for blocks without header
                if props.block.language.is_none() && props.block.filename.is_none() {
                    div { class: "absolute top-3 right-3",
                        CopyButton {
                            code: code_for_copy,
                            copied: copied,
                        }
                    }
                }
            }
        }
    }
}

/// Props for DocCodeGroup component.
#[derive(Props, Clone, PartialEq)]
pub struct DocCodeGroupProps {
    /// Code group data.
    pub group: CodeGroupNode,
}

/// Code group with multiple language variants in tabs.
#[component]
pub fn DocCodeGroup(props: DocCodeGroupProps) -> Element {
    let mut active_tab = use_signal(|| 0usize);

    rsx! {
        div { class: "dk-code-block not-prose my-6 inline-block max-w-full rounded-lg border border-base-content/10 overflow-hidden",
            // Tab headers - refined styling with subtle shadows
            div { class: "flex items-center bg-base-200/80 border-b border-base-content/10",
                for (i, block) in props.group.blocks.iter().enumerate() {
                    button {
                        key: "{i}",
                        class: if active_tab() == i {
                            "px-4 py-2.5 text-sm font-medium text-primary border-b-2 border-primary -mb-px bg-base-200/60 transition-colors"
                        } else {
                            "px-4 py-2.5 text-sm font-medium text-base-content/60 hover:text-base-content hover:bg-base-300/20 transition-colors"
                        },
                        onclick: move |_| active_tab.set(i),
                        if let Some(filename) = &block.filename {
                            "{filename}"
                        } else if let Some(lang) = &block.language {
                            "{lang}"
                        } else {
                            "Code"
                        }
                    }
                }
            }

            // Active code block
            if let Some(block) = props.group.blocks.get(active_tab()) {
                CodeGroupBlock { block: block.clone() }
            }
        }
    }
}

/// Props for CodeGroupBlock.
#[derive(Props, Clone, PartialEq)]
struct CodeGroupBlockProps {
    block: CodeBlockNode,
}

/// Code block within a code group (no top border radius).
#[component]
fn CodeGroupBlock(props: CodeGroupBlockProps) -> Element {
    let copied = use_signal(|| false);
    let code = props.block.code.clone();

    rsx! {
        div { class: "dk-code-group-block bg-base-200 relative group",
            HighlightedCode {
                code: code.clone(),
                language: props.block.language.clone(),
                filename: props.block.filename.clone(),
            }
            div { class: "absolute top-3 right-3",
                CopyButton {
                    code: code.clone(),
                    copied: copied,
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct HighlightedCodeProps {
    code: String,
    language: Option<String>,
    filename: Option<String>,
}

#[component]
fn HighlightedCode(props: HighlightedCodeProps) -> Element {
    let language = code_language(props.language.as_deref(), props.filename.as_deref());
    code_block(&props.code, language)
}

/// Render `<pre class="dk-code"><code>` with one `<span class="hl-{kind}">` per
/// classified token and bare text for everything unclassified.
///
/// An unknown language produces the same markup with no spans at all.
///
/// The base layout (padding, `overflow`, monospace font, tab size) is inlined on
/// the `<pre>` so a block still reads correctly with no stylesheet linked; the
/// token colors come from the `.hl-*` classes, which the docs kit wires to its
/// `--dk-hl-*` tokens.
pub(crate) fn code_block(code: &str, language: Option<Lang>) -> Element {
    // `highlight` borrows the source, so materialise the spans before rsx.
    let spans: Vec<(Option<&'static str>, String)> = match language {
        Some(lang) => hl_lite::highlight(lang, code)
            .into_iter()
            .map(|span| {
                let class = (span.kind != Kind::Plain).then(|| span.kind.class());
                (class, span.text.to_string())
            })
            .collect(),
        None => vec![(None, code.to_string())],
    };

    rsx! {
        pre {
            class: "dk-code",
            margin: "0",
            padding: "1rem",
            overflow: "auto",
            tab_size: "4",
            font_family: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, \"Liberation Mono\", monospace",
            font_size: "0.9rem",
            line_height: "1.55",
            code {
                for (i, (class, text)) in spans.into_iter().enumerate() {
                    if let Some(class) = class {
                        span { key: "{i}", class: "hl-{class}", "{text}" }
                    } else {
                        "{text}"
                    }
                }
            }
        }
    }
}

/// Resolve a fence's language: its slug first, then the filename shown in the
/// block header. `None` renders the block as plain text.
pub(crate) fn code_language(language: Option<&str>, filename: Option<&str>) -> Option<Lang> {
    language
        .map(str::trim)
        .and_then(Lang::from_slug)
        .or_else(|| filename.and_then(Lang::from_path))
}

/// Props for CopyButton.
#[derive(Props, Clone, PartialEq)]
struct CopyButtonProps {
    code: String,
    copied: Signal<bool>,
}

/// Copy to clipboard button.
#[component]
fn CopyButton(props: CopyButtonProps) -> Element {
    #[allow(unused_mut)]
    let mut copied = props.copied;
    let code = props.code.clone();

    rsx! {
        button {
            class: "btn btn-ghost btn-xs opacity-60 hover:opacity-100 group-hover:opacity-100 transition-all duration-150 hover:bg-base-content/10",
            "aria-label": if copied() { "Copied" } else { "Copy code" },
            "data-code": "{code}",
            onclick: move |_| {
                // Use JavaScript for clipboard (client-side only)
                #[cfg(target_arch = "wasm32")]
                {
                    use dioxus::prelude::*;
                    let code = code.clone();
                    spawn(async move {
                        // Use eval to copy to clipboard
                        let js = format!(
                            "navigator.clipboard.writeText({}).catch(console.error)",
                            serde_json::to_string(&code).unwrap_or_default()
                        );
                        let _ = document::eval(&js);
                        copied.set(true);
                        gloo_timers::future::TimeoutFuture::new(2000).await;
                        copied.set(false);
                    });
                }
            },
            if copied() {
                Icon { class: "size-4 text-success", icon: LdCheck }
            } else {
                Icon { class: "size-4", icon: LdCopy }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fence_slug_wins() {
        assert_eq!(code_language(Some("rs"), None), Some(Lang::Rust));
        assert_eq!(code_language(Some(" TOML "), None), Some(Lang::Toml));
        assert_eq!(
            code_language(Some("python"), Some("Cargo.toml")),
            Some(Lang::Python)
        );
    }

    /// A fence with no usable slug falls back to the filename in its header.
    #[test]
    fn filename_is_the_fallback() {
        assert_eq!(code_language(None, Some("src/main.rs")), Some(Lang::Rust));
        assert_eq!(
            code_language(Some("brainfuck"), Some("Dockerfile")),
            Some(Lang::Dockerfile)
        );
    }

    /// Nothing to go on, or nothing recognised: the block renders as plain text.
    #[test]
    fn unknown_language_renders_plain() {
        assert_eq!(code_language(None, None), None);
        assert_eq!(code_language(Some("brainfuck"), None), None);
        assert_eq!(code_language(Some("c++"), Some("LICENSE")), None);
    }
}
