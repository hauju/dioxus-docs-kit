use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::ld_icons::*};
use dioxus_mdx::{DocContent, DocTableOfContents, EndpointPage, extract_headers};

use crate::DocsContext;
use crate::registry::DocsRegistry;

use super::docs_layout::LayoutOffsets;
use super::page_nav::DocsPageNav;

/// Documentation page content renderer.
///
/// Checks if the path is an API endpoint page or a regular MDX page and renders accordingly.
#[component]
pub fn DocsPageContent(path: String) -> Element {
    let registry = use_context::<&'static DocsRegistry>();
    let ctx = use_context::<DocsContext>();

    // Check if this is an API endpoint page
    if let Some(operation) = registry.get_api_operation(&path)
        && let Some(spec) = registry.get_first_api_spec()
    {
        return rsx! {
            div { class: "flex flex-col",
                EndpointPage { operation: operation.clone(), spec: spec.clone() }
                main { class: "px-8 lg:px-12 pb-12",
                    div { class: "max-w-2xl",
                        DocsPageNav { current_path: path.clone() }
                    }
                }
            }
        };
    }

    let offsets = try_use_context::<LayoutOffsets>().unwrap_or(LayoutOffsets {
        sticky_top: "top-20",
        scroll_mt: "scroll-mt-20",
        sidebar_height: "h-[calc(100vh-5rem)]",
    });

    let doc = match registry.get_parsed_doc(&path) {
        Some(d) => d,
        None => {
            let base = ctx.base_path.clone();
            return rsx! {
                div { class: "container mx-auto px-8 py-12 max-w-4xl",
                    div { class: "text-center",
                        h1 { class: "text-4xl font-bold mb-4", "404" }
                        p { class: "text-base-content/70 mb-8",
                            "Page not found: {path}"
                        }
                        Link {
                            to: NavigationTarget::Internal(base),
                            class: "btn btn-primary",
                            "Go to Documentation"
                        }
                    }
                }
            };
        }
    };

    let headers = extract_headers(&doc.raw_markdown);

    rsx! {
        div { class: "flex",
            // Main content
            main { class: "flex-1 min-w-0 px-8 py-12 lg:px-12",
                article { class: "max-w-3xl mx-auto",
                    // Page header
                    header { class: "mb-8 pb-8 border-b border-base-300",
                        div { class: "flex items-start justify-between gap-4",
                            h1 { class: "text-4xl font-bold tracking-tight mb-3",
                                "{doc.frontmatter.title}"
                            }
                            CopyMdxButton { content: doc.raw_markdown.clone() }
                        }
                        if let Some(desc) = &doc.frontmatter.description {
                            p { class: "text-lg text-base-content/70",
                                "{desc}"
                            }
                        }
                    }

                    // MDX content
                    div { class: "prose prose-base max-w-none
                        prose-headings:{offsets.scroll_mt}
                        prose-h2:text-2xl prose-h2:font-semibold prose-h2:mt-10 prose-h2:mb-4
                        prose-h3:text-xl prose-h3:font-medium prose-h3:mt-8 prose-h3:mb-3
                        prose-p:text-base-content/80 prose-p:leading-relaxed
                        prose-a:text-primary prose-a:no-underline hover:prose-a:underline
                        prose-code:bg-base-200 prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:text-sm
                        prose-pre:bg-base-200 prose-pre:border prose-pre:border-base-300",
                        DocContent { nodes: doc.content.clone() }
                    }

                    // Page navigation
                    DocsPageNav { current_path: path.clone() }
                }
            }

            // Table of Contents sidebar (right side)
            if !headers.is_empty() {
                aside { class: "w-56 shrink-0 hidden xl:block",
                    div { class: "sticky {offsets.sticky_top} p-6",
                        DocTableOfContents { headers }
                    }
                }
            }
        }
    }
}

/// Copy MDX source button for doc pages.
#[component]
fn CopyMdxButton(content: String) -> Element {
    #[allow(unused_mut)]
    let mut copied = use_signal(|| false);

    rsx! {
        button {
            class: "btn btn-ghost btn-sm gap-1.5 opacity-60 hover:opacity-100 transition-all duration-150 hover:bg-base-content/10 shrink-0",
            title: "Copy MDX source",
            onclick: move |_| {
                #[cfg(target_arch = "wasm32")]
                {
                    use dioxus::prelude::*;
                    let content = content.clone();
                    spawn(async move {
                        let js = format!(
                            "navigator.clipboard.writeText({}).catch(console.error)",
                            serde_json::to_string(&content).unwrap_or_default()
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
                span { class: "text-xs", "Copied!" }
            } else {
                Icon { class: "size-4", icon: LdCopy }
                span { class: "text-xs", "Copy page" }
            }
        }
    }
}
