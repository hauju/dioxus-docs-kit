use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::LdClock;
use dioxus_mdx::{DocContent, DocTableOfContents, extract_headers};

use crate::BlogContext;
use crate::blog::registry::BlogRegistry;

use super::author_info::AuthorInfo;
use super::post_nav::BlogPostNav;

/// Single blog post view.
#[component]
pub fn BlogPostView(slug: String) -> Element {
    let registry = use_context::<&'static BlogRegistry>();
    let ctx = use_context::<BlogContext>();

    let post = match registry.get_post(&slug) {
        Some(p) => p,
        None => {
            let base = ctx.base_path.clone();
            return rsx! {
                div { class: "max-w-4xl mx-auto px-4 py-12",
                    div { class: "text-center",
                        h1 { class: "text-4xl font-bold mb-4", "404" }
                        p { class: "text-base-content/70 mb-8",
                            "Post not found: {slug}"
                        }
                        Link {
                            to: NavigationTarget::Internal(base),
                            class: "btn btn-primary",
                            "Back to Blog"
                        }
                    }
                }
            };
        }
    };

    let date_display = registry.format_date(&post.frontmatter.date);
    let headers = extract_headers(&post.raw_markdown);

    rsx! {
        div { class: "flex max-w-6xl mx-auto",
            main { class: "flex-1 min-w-0 px-4 py-12 lg:px-12",
                article { class: "max-w-3xl mx-auto",
                    if let Some(ref cover) = post.frontmatter.cover_image {
                        div { class: "mb-8 rounded-xl overflow-hidden",
                            img {
                                src: "{cover}",
                                alt: "{post.frontmatter.title}",
                                class: "w-full",
                            }
                        }
                    }

                    header { class: "mb-8 pb-8 border-b border-base-300",
                        if !post.frontmatter.tags.is_empty() {
                            div { class: "flex flex-wrap gap-1.5 mb-4",
                                for tag in post.frontmatter.tags.iter() {
                                    span { class: "badge badge-sm badge-outline badge-primary font-medium",
                                        "{tag}"
                                    }
                                }
                            }
                        }

                        h1 { class: "text-4xl font-bold tracking-tight mb-4",
                            "{post.frontmatter.title}"
                        }

                        if let Some(ref desc) = post.frontmatter.description {
                            p { class: "text-lg text-base-content/60 mb-6",
                                "{desc}"
                            }
                        }

                        div { class: "flex items-center gap-4 text-sm text-base-content/60",
                            AuthorInfo { author_id: post.frontmatter.author.clone() }
                            span { class: "text-base-content/30", "|" }
                            span { "{date_display}" }
                            span { class: "text-base-content/30", "|" }
                            div { class: "flex items-center gap-1",
                                Icon { class: "size-3.5", icon: LdClock }
                                span { "{post.reading_time_minutes} min read" }
                            }
                        }
                    }

                    div { class: "prose prose-base max-w-none
                        prose-headings:scroll-mt-20
                        prose-h2:text-2xl prose-h2:font-semibold prose-h2:mt-10 prose-h2:mb-4
                        prose-h3:text-xl prose-h3:font-medium prose-h3:mt-8 prose-h3:mb-3
                        prose-p:text-base-content/80 prose-p:leading-relaxed
                        prose-a:text-primary prose-a:no-underline hover:prose-a:underline
                        prose-code:bg-base-200 prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:text-sm
                        prose-pre:bg-base-200 prose-pre:border prose-pre:border-base-300",
                        DocContent { nodes: post.content.clone() }
                    }

                    BlogPostNav { current_slug: slug.clone() }
                }
            }

            if !headers.is_empty() {
                aside { class: "w-56 shrink-0 hidden xl:block",
                    div { class: "sticky top-20 p-6",
                        DocTableOfContents { headers }
                    }
                }
            }
        }
    }
}
