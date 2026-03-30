use dioxus::prelude::*;

use crate::blog::registry::BlogRegistry;

use super::blog_card::BlogCard;

/// Displays related posts based on tag overlap.
///
/// Renders nothing if no related posts are found.
#[component]
pub fn RelatedPosts(slug: String, #[props(default = 3)] max: usize) -> Element {
    let registry = use_context::<&'static BlogRegistry>();
    let related = registry.related_posts(&slug, max);

    if related.is_empty() {
        return rsx! {};
    }

    rsx! {
        section { class: "mt-16 pt-8 border-t border-base-300",
            h2 { class: "text-xl font-semibold mb-6", "Related Posts" }
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                for post in related {
                    BlogCard { key: "{post.slug}", post: post.clone() }
                }
            }
        }
    }
}
