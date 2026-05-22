//! Code block components for documentation.
//!
//! Features syntax highlighting for common programming languages.

use dioxus::prelude::*;
use dioxus_code::{Code, CodeTheme, Language, SourceCode, Theme};
use dioxus_free_icons::{Icon, icons::ld_icons::*};

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
        div { class: "dk-code-block my-6 relative group inline-block max-w-full rounded-lg border border-base-content/10 overflow-hidden",
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
        div { class: "dk-code-block my-6 inline-block max-w-full rounded-lg border border-base-content/10 overflow-hidden",
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
    let theme = CodeTheme::system(Theme::GITHUB_LIGHT, Theme::TOKYO_NIGHT);

    rsx! {
        Code {
            src: SourceCode::new(language, props.code),
            theme,
        }
    }
}

pub(crate) fn code_language(language: Option<&str>, filename: Option<&str>) -> Language {
    language
        .and_then(language_from_alias)
        .or_else(|| filename.and_then(Language::detect))
        .or_else(|| language.and_then(Language::detect))
        .unwrap_or(Language::Markdown)
}

fn language_from_alias(language: &str) -> Option<Language> {
    let normalized = language.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "bash" | "sh" | "shell" | "zsh" | "console" | "terminal" => Some(Language::Bash),
        "c++" | "cc" | "cxx" | "hpp" => Some(Language::Cpp),
        "c#" | "cs" => Some(Language::CSharp),
        "docker" | "dockerfile" | "containerfile" => Some(Language::Dockerfile),
        "html" | "htm" => Some(Language::Html),
        "js" | "javascript" | "jsx" | "mjs" | "cjs" => Some(Language::JavaScript),
        "json" | "jsonc" => Some(Language::Json),
        "markdown" | "md" | "mdx" => Some(Language::Markdown),
        "py" | "python" => Some(Language::Python),
        "rs" | "rust" => Some(Language::Rust),
        "ts" | "typescript" => Some(Language::TypeScript),
        "tsx" => Some(Language::Tsx),
        "toml" => Some(Language::Toml),
        "yaml" | "yml" => Some(Language::Yaml),
        other => Language::from_slug(other),
    }
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
