use dioxus::prelude::*;

use crate::BlogContext;
use crate::blog::registry::BlogRegistry;

/// Injects Open Graph / SEO meta tags and document title for a single blog post.
#[component]
pub fn BlogPostMeta(slug: String, site_url: String) -> Element {
    let registry = use_context::<&'static BlogRegistry>();
    let ctx = use_context::<BlogContext>();

    let post = match registry.get_post(&slug) {
        Some(p) => p,
        None => return rsx! {},
    };

    let title = &post.frontmatter.title;
    let description = post.frontmatter.description.as_deref().unwrap_or("");
    let url = format!("{}{}/{}", site_url, ctx.base_path, slug);
    let date = &post.frontmatter.date;
    let author_name = registry
        .get_author(&post.frontmatter.author)
        .map(|a| a.name.as_str())
        .unwrap_or("");

    rsx! {
        document::Title { "{title}" }
        document::Meta { name: "description", content: "{description}" }

        // Open Graph
        document::Meta { property: "og:title", content: "{title}" }
        document::Meta { property: "og:description", content: "{description}" }
        document::Meta { property: "og:type", content: "article" }
        document::Meta { property: "og:url", content: "{url}" }
        if let Some(ref cover) = post.frontmatter.cover_image {
            document::Meta { property: "og:image", content: "{cover}" }
        }

        // Twitter Card
        document::Meta { name: "twitter:card", content: "summary_large_image" }
        document::Meta { name: "twitter:title", content: "{title}" }
        document::Meta { name: "twitter:description", content: "{description}" }
        if let Some(ref cover) = post.frontmatter.cover_image {
            document::Meta { name: "twitter:image", content: "{cover}" }
        }

        // Article metadata
        document::Meta { property: "article:published_time", content: "{date}" }
        if !author_name.is_empty() {
            document::Meta { property: "article:author", content: "{author_name}" }
        }
        for tag in post.frontmatter.tags.iter() {
            document::Meta { property: "article:tag", content: "{tag}" }
        }
    }
}

/// Injects basic SEO meta tags for the blog index/listing page.
#[component]
pub fn BlogIndexMeta(title: String, description: String, site_url: String) -> Element {
    let ctx = use_context::<BlogContext>();
    let url = format!("{}{}", site_url, ctx.base_path);

    rsx! {
        document::Title { "{title}" }
        document::Meta { name: "description", content: "{description}" }
        document::Meta { property: "og:title", content: "{title}" }
        document::Meta { property: "og:description", content: "{description}" }
        document::Meta { property: "og:type", content: "website" }
        document::Meta { property: "og:url", content: "{url}" }
        document::Meta { name: "twitter:card", content: "summary" }
        document::Meta { name: "twitter:title", content: "{title}" }
        document::Meta { name: "twitter:description", content: "{description}" }
    }
}
