use dioxus::prelude::*;

use super::blog_meta::listing_tags;
use super::managed_head::{HeadTag, ManagedBlogHead};
use super::{BlogCard, TagFilter};
use crate::components::seo::join_site_url;
use crate::{BlogCategory, BlogContext, BlogRegistry};

/// A shareable topic page showing published posts with the category's tag.
///
/// Enable with `BlogConfig::with_category_base_path` and mount on
/// `<base>/:slug` and `<base>/:slug/page/:page`. Page numbers start at one.
/// Unknown categories and invalid pages render a noindex not-found page and
/// return HTTP 404 when the `server` feature is enabled.
#[component]
pub fn BlogCategoryPage(slug: String, #[props(default = 1)] page: usize) -> Element {
    let registry = use_context::<&'static BlogRegistry>();
    let ctx = use_context::<BlogContext>();
    let Some((category, posts)) = registry
        .get_category(&slug)
        .zip(registry.category_posts_page(&slug, page))
    else {
        #[cfg(feature = "server")]
        if let Some(mut context) = dioxus_fullstack_core::FullstackContext::current() {
            context.set_current_http_status(dioxus_fullstack_core::HttpError::new(
                dioxus::server::http::StatusCode::NOT_FOUND,
                "Category page not found",
            ));
        }
        return rsx! {
            ManagedBlogHead { title: "Category page not found", tags: vec![HeadTag::meta("name", "robots", "noindex")] }
            main { class: "max-w-6xl mx-auto px-4 py-12",
                h1 { class: "text-4xl font-bold mb-4", "Category page not found" }
                p { class: "text-base-content/70 mb-8", "This topic or page doesn't exist." }
                Link { to: NavigationTarget::Internal(ctx.base_path), class: "btn btn-primary", "Back to Blog" }
            }
        };
    };
    let total_pages = registry.total_pages_for_tag(&category.tag);
    let count = registry.tag_count(&category.tag);
    rsx! {
        CategoryMeta { category: category.clone(), page }
        main { class: "dk-blog-category max-w-6xl mx-auto px-4 py-12",
            header { class: "max-w-3xl mb-8",
                Link { to: NavigationTarget::Internal(ctx.base_path), class: "text-sm text-base-content/60 hover:text-primary", "Blog" }
                h1 { class: "text-4xl font-bold tracking-tight mt-4 mb-3", "{category.title}" }
                p { class: "text-lg text-base-content/60 leading-relaxed", "{category.description}" }
                p { class: "text-sm text-base-content/50 mt-4", "{count} articles" }
                if let Some(image) = &category.image {
                    img { src: "{image}", alt: "", class: "w-full rounded-xl mt-6", loading: "lazy" }
                }
            }
            div { class: "mb-8", TagFilter {} }
            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                for post in posts {
                    BlogCard { key: "{post.slug}", post: post.clone() }
                }
            }
            if total_pages > 1 {
                nav { class: "flex flex-wrap items-center justify-center gap-2 mt-12", aria_label: "Category pagination",
                    if page > 1 {
                        Link { to: NavigationTarget::Internal(registry.category_url(&slug, page - 1).unwrap()), class: "btn btn-ghost btn-sm", "Previous" }
                    }
                    span { class: "text-sm text-base-content/60", "Page {page} of {total_pages}" }
                    if page < total_pages {
                        Link { to: NavigationTarget::Internal(registry.category_url(&slug, page + 1).unwrap()), class: "btn btn-ghost btn-sm", "Next" }
                    }
                }
            }
        }
    }
}

// Category-specific canonical URL and social metadata.
#[component]
fn CategoryMeta(category: BlogCategory, page: usize) -> Element {
    let ctx = use_context::<BlogContext>();
    let registry = use_context::<&'static BlogRegistry>();
    if !ctx.auto_meta {
        return rsx! {};
    }
    let title = if page == 1 {
        category.title.clone()
    } else {
        format!("{} — Page {page}", category.title)
    };
    let description = category.description;
    let path = registry.category_url(&category.slug, page).unwrap();
    let canonical = ctx
        .site_url
        .as_deref()
        .map(|site| join_site_url(site, &path, ""));
    let image = category.image.map(|image| {
        if image.starts_with('/') && !image.starts_with("//") {
            ctx.site_url
                .as_deref()
                .map(|site| join_site_url(site, &image, ""))
                .unwrap_or(image)
        } else {
            image
        }
    });
    let tags = listing_tags(&title, &description, canonical.as_deref(), image.as_deref());
    rsx! { ManagedBlogHead { title, tags } }
}
