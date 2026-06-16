use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::LdClock;

use crate::BlogContext;
use crate::blog::registry::BlogRegistry;
use crate::blog::types::BlogPost;

/// Individual blog post card for the listing page.
#[component]
pub fn BlogCard(post: BlogPost) -> Element {
    let ctx = use_context::<BlogContext>();
    let registry = use_context::<&'static BlogRegistry>();
    let slug = post.slug.clone();
    let href = format!("{}/{}", ctx.base_path, slug);
    let author = registry.get_author(&post.frontmatter.author);
    let date_display = registry.format_date(&post.frontmatter.date);

    rsx! {
        Link {
            to: NavigationTarget::Internal(href),
            class: "group flex flex-col rounded-xl border border-base-300 bg-base-200/30 hover:border-primary/30 hover:shadow-lg transition-all duration-200 overflow-hidden",

            if let Some(ref cover) = post.frontmatter.cover_image {
                div { class: "aspect-video overflow-hidden bg-base-300",
                    img {
                        src: "{cover}",
                        alt: "{post.frontmatter.title}",
                        class: "w-full h-full object-cover group-hover:scale-105 transition-transform duration-300",
                    }
                }
            }

            div { class: "flex flex-col flex-1 p-5 gap-3",
                if !post.frontmatter.tags.is_empty() {
                    div { class: "flex flex-wrap gap-1.5",
                        for tag in post.frontmatter.tags.iter().take(3) {
                            span { class: "badge badge-sm badge-outline", "{tag}" }
                        }
                        if post.frontmatter.tags.len() > 3 {
                            span { class: "badge badge-sm badge-ghost",
                                "+{post.frontmatter.tags.len() - 3}"
                            }
                        }
                    }
                }

                h2 { class: "text-lg font-semibold leading-snug group-hover:text-primary transition-colors line-clamp-2",
                    "{post.frontmatter.title}"
                }

                if let Some(ref desc) = post.frontmatter.description {
                    p { class: "text-sm text-base-content/60 leading-relaxed line-clamp-3",
                        "{desc}"
                    }
                }

                div { class: "mt-auto pt-3 border-t border-base-300/60 flex items-center gap-3 text-xs text-base-content/50",
                    if let Some(author) = author {
                        div { class: "flex items-center gap-1.5",
                            if let Some(ref avatar) = author.avatar {
                                img {
                                    src: "{avatar}",
                                    alt: "{author.name}",
                                    class: "size-5 rounded-full",
                                }
                            }
                            span { "{author.name}" }
                        }
                    }
                    span { "{date_display}" }
                    div { class: "flex items-center gap-1 ml-auto",
                        Icon { class: "size-3", icon: LdClock }
                        span { "{post.reading_time_minutes} min read" }
                    }
                }
            }
        }
    }
}
