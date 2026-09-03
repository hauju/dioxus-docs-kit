//! Code block components for documentation.
//!
//! Features syntax highlighting for common programming languages.

use dioxus::prelude::*;
#[cfg(feature = "highlight")]
use dioxus_code::{Code, CodeTheme, Language, SourceCode, Theme};
use dioxus_free_icons::{Icon, icons::ld_icons::*};

#[cfg(feature = "mermaid")]
use super::mermaid::MermaidDiagram;
use crate::parser::{CodeBlockNode, CodeGroupNode};

/// Reactive override for the syntax-highlighting theme used by rendered code blocks.
///
/// Provide this context above [`MdxContent`](crate::MdxContent) (or any component that
/// renders [`DocCodeBlock`]) to control the code theme — for example to track a
/// site-wide light/dark toggle driven by the `data-theme` attribute.
///
/// When no override is provided, code blocks fall back to
/// `CodeTheme::system(Theme::GITHUB_LIGHT, Theme::TOKYO_NIGHT)`, which switches on the
/// reader's OS `prefers-color-scheme`.
///
/// Only available with the `highlight` feature (default), which pulls in `dioxus-code`.
#[cfg(feature = "highlight")]
#[derive(Clone, Copy)]
pub struct CodeThemeOverride(pub ReadSignal<CodeTheme>);

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
        // `<pre class="dxc"><code>`, painting the inline-code pill background as a box
        // behind every wrapped line. dioxus-code styles the block itself.
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

#[cfg(feature = "highlight")]
#[component]
fn HighlightedCode(props: HighlightedCodeProps) -> Element {
    // No grammar for this fence in this build - either the language is unknown
    // or its `lang-*` feature is off. Render plain text rather than coloring
    // the block with some unrelated grammar.
    let Some(language) = code_language(props.language.as_deref(), props.filename.as_deref()) else {
        return plain_code_block(&props.code);
    };
    let theme = match try_use_context::<CodeThemeOverride>() {
        Some(CodeThemeOverride(theme)) => theme(),
        None => CodeTheme::system(Theme::GITHUB_LIGHT, Theme::TOKYO_NIGHT),
    };

    rsx! {
        Code {
            src: SourceCode::new(language, props.code),
            theme,
        }
    }
}

/// Fallback for [`HighlightedCode`] when the `highlight` feature is disabled.
///
/// Renders the code as an escaped plain-text node inside the same
/// `<pre class="dxc"><code>` markup the highlighted path emits, so the outer
/// wrappers, copy buttons, and CodeGroup tabs keep working — only token coloring
/// is lost. See [`plain_code_block`] for why the base layout is inlined.
#[cfg(not(feature = "highlight"))]
#[component]
fn HighlightedCode(props: HighlightedCodeProps) -> Element {
    plain_code_block(&props.code)
}

/// Render a `<pre class="dxc"><code>` block containing the code as an escaped
/// plain-text node, used whenever no grammar is available: the `highlight`
/// feature is off, or the fence's `lang-*` feature is not enabled.
///
/// Without the `highlight` feature `dioxus-code`'s stylesheet is not linked, so its
/// base `.dxc` layout (padding, `overflow: auto`, monospace font) is inlined here to
/// keep scrolling and spacing intact.
pub(crate) fn plain_code_block(code: &str) -> Element {
    rsx! {
        pre {
            class: "dxc",
            margin: "0",
            padding: "1rem",
            overflow: "auto",
            tab_size: "4",
            font_family: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, \"Liberation Mono\", monospace",
            font_size: "0.9rem",
            line_height: "1.55",
            code { "{code}" }
        }
    }
}

/// Resolve a fence's language to a grammar compiled into this build.
///
/// Returns `None` when the language is unknown *or* when its `lang-*` feature is
/// disabled — [`Language`]'s variants are gated by the same features, so an
/// alias for a grammar that was not compiled in simply falls through to
/// [`Language::from_slug`], which also returns `None` for it.
#[cfg(feature = "highlight")]
pub(crate) fn code_language(language: Option<&str>, filename: Option<&str>) -> Option<Language> {
    language
        .and_then(language_from_alias)
        .or_else(|| filename.and_then(Language::detect))
        .or_else(|| language.and_then(Language::detect))
}

#[cfg(feature = "highlight")]
fn language_from_alias(language: &str) -> Option<Language> {
    let normalized = language.trim().to_ascii_lowercase();
    match normalized.as_str() {
        #[cfg(feature = "lang-bash")]
        "bash" | "sh" | "shell" | "zsh" | "console" | "terminal" => Some(Language::Bash),
        #[cfg(feature = "lang-cpp")]
        "c++" | "cc" | "cxx" | "hpp" => Some(Language::Cpp),
        #[cfg(feature = "lang-c-sharp")]
        "c#" | "cs" => Some(Language::CSharp),
        #[cfg(feature = "lang-dockerfile")]
        "docker" | "dockerfile" | "containerfile" => Some(Language::Dockerfile),
        #[cfg(feature = "lang-html")]
        "html" | "htm" => Some(Language::Html),
        #[cfg(feature = "lang-javascript")]
        "js" | "javascript" | "jsx" | "mjs" | "cjs" => Some(Language::JavaScript),
        #[cfg(feature = "lang-json")]
        "json" | "jsonc" => Some(Language::Json),
        #[cfg(feature = "lang-markdown")]
        "markdown" | "md" | "mdx" => Some(Language::Markdown),
        #[cfg(feature = "lang-python")]
        "py" | "python" => Some(Language::Python),
        // Rust needs no `lang-*` feature: `dioxus-code`'s `runtime` always
        // compiles it, so `Language::Rust` is never gated out.
        "rs" | "rust" => Some(Language::Rust),
        #[cfg(feature = "lang-typescript")]
        "ts" | "typescript" => Some(Language::TypeScript),
        #[cfg(feature = "lang-tsx")]
        "tsx" => Some(Language::Tsx),
        #[cfg(feature = "lang-toml")]
        "toml" => Some(Language::Toml),
        #[cfg(feature = "lang-yaml")]
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

#[cfg(all(test, feature = "highlight"))]
mod tests {
    use super::*;

    #[test]
    fn rust_needs_no_lang_feature() {
        assert_eq!(code_language(Some("rs"), None), Some(Language::Rust));
    }

    #[test]
    fn unknown_language_has_no_grammar() {
        assert_eq!(code_language(Some("brainfuck"), None), None);
    }

    /// A fence with no language and no filename has nothing to detect from, so
    /// `HighlightedCode` renders it as plain text instead of guessing a grammar.
    #[test]
    fn bare_fence_has_no_grammar() {
        assert_eq!(code_language(None, None), None);
    }

    /// Languages whose `lang-*` feature is off resolve to `None`, which routes
    /// the block through `plain_code_block` rather than panicking or coloring
    /// it with an unrelated grammar. C++ and C# are excluded from the default
    /// features because their grammars dominate the wasm bundle.
    #[test]
    #[cfg(not(feature = "lang-cpp"))]
    fn disabled_grammar_falls_back_to_plain_text() {
        assert_eq!(code_language(Some("c++"), None), None);
        assert_eq!(code_language(Some("cpp"), None), None);
    }

    #[test]
    #[cfg(not(feature = "lang-c-sharp"))]
    fn disabled_c_sharp_grammar_falls_back_to_plain_text() {
        assert_eq!(code_language(Some("c#"), None), None);
        assert_eq!(code_language(Some("cs"), None), None);
    }

    #[test]
    #[cfg(feature = "lang-python")]
    fn enabled_grammar_resolves() {
        assert_eq!(code_language(Some("py"), None), Some(Language::Python));
    }
}
