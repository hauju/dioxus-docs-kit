use crate::components::managed_head::{HeadTag, ManagedPageHead};
use dioxus::prelude::*;

use crate::BlogContext;
use crate::blog::registry::BlogRegistry;
use docs_kit_core::seo::{article_jsonld, join_site_url};

/// Injects Open Graph / SEO meta tags and document title for a single blog post.
///
/// Reads `auto_meta` and `site_url` from [`BlogContext`]. When `auto_meta` is
/// off, emits nothing. Otherwise emits title/description, Open Graph, Twitter
/// Card, article metadata, and schema.org Article JSON-LD from frontmatter.
/// Canonical, `og:url`, and the JSON-LD `@id` are only emitted when `site_url`
/// is also set.
#[component]
pub fn BlogPostMeta(slug: String) -> Element {
    let registry = use_context::<&'static BlogRegistry>();
    let ctx = use_context::<BlogContext>();

    if !ctx.auto_meta {
        return rsx! {};
    }

    let post = match registry.get_post(&slug) {
        Some(p) => p,
        None => return rsx! {},
    };

    let title = &post.frontmatter.title;
    let description = post.frontmatter.description.as_deref().unwrap_or("");
    let canonical = ctx
        .site_url
        .as_deref()
        .map(|origin| join_site_url(origin, &ctx.base_path, &slug));
    // Root-relative `<base_path>/<slug>.md`; `join_site_url` with an empty origin
    // yields the path portion only.
    let markdown_href = ctx
        .markdown_alternate
        .then(|| format!("{}.md", join_site_url("", &ctx.base_path, &slug)));
    let date = &post.frontmatter.date;
    let author_name = registry
        .get_author(&post.frontmatter.author)
        .map(|a| a.name.as_str())
        .unwrap_or("");

    let json_ld = article_jsonld(
        title,
        description,
        canonical.as_deref(),
        date,
        author_name,
        post.frontmatter.cover_image.as_deref(),
    );

    let mut tags = listing_tags(
        title,
        description,
        canonical.as_deref(),
        post.frontmatter.cover_image.as_deref(),
    );
    tags.retain(|tag| tag != &HeadTag::meta("property", "og:type", "website"));
    tags.push(HeadTag::meta("property", "og:type", "article"));
    tags.push(HeadTag::meta("property", "article:published_time", date));
    if !author_name.is_empty() {
        tags.push(HeadTag::meta("property", "article:author", author_name));
    }
    for tag in &post.frontmatter.tags {
        tags.push(HeadTag::meta("property", "article:tag", tag));
    }
    if let Some(href) = markdown_href {
        tags.push(HeadTag::link("alternate", &href, Some("text/markdown")));
    }
    tags.push(HeadTag::jsonld(json_ld));
    rsx! { ManagedPageHead { title: title.clone(), tags } }
}

/// Injects basic SEO meta tags for the blog index/listing page.
///
/// Reads `auto_meta` and `site_url` from [`BlogContext`]. When `auto_meta` is
/// off, emits nothing. Canonical and `og:url` only emit when `site_url` is set.
#[component]
pub fn BlogIndexMeta(title: String, description: String) -> Element {
    let ctx = use_context::<BlogContext>();

    if !ctx.auto_meta {
        return rsx! {};
    }

    let canonical = ctx
        .site_url
        .as_deref()
        .map(|origin| join_site_url(origin, &ctx.base_path, ""));

    let tags = listing_tags(&title, &description, canonical.as_deref(), None);
    rsx! { ManagedPageHead { title, tags } }
}

pub(super) fn listing_tags(
    title: &str,
    description: &str,
    canonical: Option<&str>,
    image: Option<&str>,
) -> Vec<HeadTag> {
    let mut tags = vec![
        HeadTag::meta("name", "description", description),
        HeadTag::meta("property", "og:title", title),
        HeadTag::meta("property", "og:description", description),
        HeadTag::meta("property", "og:type", "website"),
        HeadTag::meta(
            "name",
            "twitter:card",
            if image.is_some() {
                "summary_large_image"
            } else {
                "summary"
            },
        ),
        HeadTag::meta("name", "twitter:title", title),
        HeadTag::meta("name", "twitter:description", description),
    ];
    if let Some(url) = canonical {
        tags.push(HeadTag::link("canonical", url, None));
        tags.push(HeadTag::meta("property", "og:url", url));
    }
    if let Some(image) = image {
        tags.push(HeadTag::meta("property", "og:image", image));
        tags.push(HeadTag::meta("name", "twitter:image", image));
    }
    tags
}
