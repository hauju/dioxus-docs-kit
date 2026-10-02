//! The blog: index (featured posts, tag or category filter, pagination), post
//! pages, category pages and search, plus their `<head>` tags. Markup mirrors
//! `dioxus-docs-kit`'s blog; filters and pagination are plain links, so no
//! runtime script is needed.
//!
//! Page numbers are one-based everywhere in this module, as they are in URLs.

use dioxus_mdx::extract_headers;
use dioxus_mdx::lucide::{LdChevronLeft, LdChevronRight, LdClock, LdSearch};
use docs_kit_core::search::{MAX_RESULTS, SNIPPET_WINDOW, build_snippet, split_terms};
use docs_kit_core::seo::{article_jsonld, join_site_url};
use docs_kit_core::{BlogPost, BlogRegistry};
use topcoat::{
    Result,
    router::StatusCode,
    view::{Child, Unescaped, View, component, view},
};

use crate::content::doc_content;
use crate::icon::icon;
use crate::shell::toc;
use crate::theme::theme_toggle;

// ============================================================================
// Pages
// ============================================================================

/// The blog index at `base_path`: featured posts, the tag (or category) filter,
/// and one page of posts. `tag` and `page` come from the `?tag=` and `?page=`
/// query parameters the filter and pagination links use. A page past the end
/// renders "No posts found" with a 404 status.
#[component]
pub async fn blog_index_page(
    registry: &BlogRegistry,
    base_path: &str,
    tag: Option<&str>,
    page: usize,
) -> Result<impl View> {
    let page = page.max(1);
    let (posts, total_pages) = match tag {
        Some(tag) => (
            registry.posts_page_by_tag(tag, page - 1),
            registry.total_pages_for_tag(tag),
        ),
        None => (
            registry.non_featured_posts_page(page - 1),
            registry.non_featured_total_pages(),
        ),
    };
    let show_featured = tag.is_none() && page == 1 && registry.has_featured();
    let heading = match tag {
        Some(tag) => format!("Blog: {tag}"),
        None => "Blog".to_string(),
    };

    Ok(view! {
        blog_frame(registry: registry, base_path: base_path, query: "",
            <div class="max-w-6xl mx-auto px-4 py-12">
                <h1 class="text-4xl font-bold tracking-tight mb-8">(heading)</h1>
                if !registry.all_tags().is_empty() {
                    <div class="mb-8">tag_filter(registry: registry, base_path: base_path, active_tag: tag, active_category: None)</div>
                }
                if show_featured {
                    <div class="mb-10">
                        <div class="flex items-center gap-3 mb-4">
                            <span class="badge badge-primary badge-sm">"Featured"</span>
                            <div class="h-px flex-1 bg-base-300"></div>
                        </div>
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                            for post in registry.featured_posts() {
                                blog_card(registry: registry, base_path: base_path, post: post)
                            }
                        </div>
                    </div>
                }
                if posts.is_empty() {
                    if page > 1 {
                        (StatusCode::NOT_FOUND)
                    }
                    <div class="text-center py-16 text-base-content/50"><p class="text-lg">"No posts found."</p></div>
                } else {
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                        for post in &posts {
                            blog_card(registry: registry, base_path: base_path, post: post)
                        }
                    </div>
                }
                if total_pages > 1 {
                    <nav class="flex flex-wrap items-center justify-center gap-2 mt-12" aria-label="Blog pagination">
                        if page > 1 {
                            <a href=(index_url(base_path, tag, page - 1)) class="btn btn-ghost btn-sm">icon(glyph: LdChevronLeft, class: "size-4") "Prev"</a>
                        }
                        for n in 1..=total_pages {
                            <a
                                href=(index_url(base_path, tag, n))
                                aria-current=((n == page).then_some("page"))
                                class=(if n == page { "btn btn-sm btn-primary" } else { "btn btn-sm btn-ghost" })
                            >
                                (n)
                            </a>
                        }
                        if page < total_pages {
                            <a href=(index_url(base_path, tag, page + 1)) class="btn btn-ghost btn-sm">"Next" icon(glyph: LdChevronRight, class: "size-4")</a>
                        }
                    </nav>
                }
            </div>
        )
    })
}

/// One post: cover, tags, title, author, date, reading time, content, table
/// of contents, related posts and older/newer links. Unknown slugs render a
/// 404 page and set the response status.
#[component]
pub async fn blog_post_page(
    registry: &BlogRegistry,
    base_path: &str,
    slug: &str,
) -> Result<impl View> {
    let post = registry.get_post(slug);
    let headers = post
        .map(|p| extract_headers(&p.raw_markdown))
        .unwrap_or_default();
    let related = registry.related_posts(slug, 3);
    let prev = registry.prev_post(slug);
    let next = registry.next_post(slug);

    Ok(view! {
        blog_frame(registry: registry, base_path: base_path, query: "",
            match post {
                None => {
                    (StatusCode::NOT_FOUND)
                    <div class="max-w-4xl mx-auto px-4 py-12">
                        <div class="text-center">
                            <h1 class="text-4xl font-bold mb-4">"404"</h1>
                            <p class="text-base-content/70 mb-8">"Post not found: " (slug)</p>
                            <a href=(base_path) class="btn btn-primary">"Back to Blog"</a>
                        </div>
                    </div>
                },
                Some(post) => {
                    // Scroll-driven, so no script; hidden where unsupported.
                    <div class="dk-reading-progress-track fixed top-0 left-0 w-full z-[100] pointer-events-none">
                        <div class="dk-reading-progress h-0.5 bg-primary"></div>
                    </div>
                    <div class="flex max-w-6xl mx-auto">
                        <main class="flex-1 min-w-0 px-4 py-12 lg:px-12">
                            <article class="max-w-3xl mx-auto">
                                if let Some(cover) = &post.frontmatter.cover_image {
                                    <div class="mb-8 rounded-xl overflow-hidden">
                                        <img src=(cover.as_str()) alt=(post.frontmatter.title.as_str()) class="w-full">
                                    </div>
                                }
                                <header class="mb-8 pb-8 border-b border-base-300">
                                    if !post.frontmatter.tags.is_empty() {
                                        <div class="flex flex-wrap gap-1.5 mb-4">
                                            for tag in &post.frontmatter.tags {
                                                <a href=(tag_url(registry, base_path, tag)) class="badge badge-sm badge-outline badge-primary font-medium">(tag.as_str())</a>
                                            }
                                        </div>
                                    }
                                    <h1 class="text-4xl font-bold tracking-tight mb-4">(post.frontmatter.title.as_str())</h1>
                                    if let Some(desc) = &post.frontmatter.description {
                                        <p class="text-lg text-base-content/60 mb-6">(desc.as_str())</p>
                                    }
                                    <div class="flex flex-wrap items-center gap-4 text-sm text-base-content/60">
                                        author_info(registry: registry, author_id: &post.frontmatter.author)
                                        <span class="text-base-content/30">"|"</span>
                                        <span>(registry.format_date(&post.frontmatter.date))</span>
                                        <span class="text-base-content/30">"|"</span>
                                        <div class="flex items-center gap-1">
                                            icon(glyph: LdClock, class: "size-3.5")
                                            <span>(post.reading_time_minutes) " min read"</span>
                                        </div>
                                    </div>
                                </header>
                                <div class="prose prose-base max-w-none prose-headings:scroll-mt-20 prose-h2:text-2xl prose-h2:font-semibold prose-h2:mt-10 prose-h2:mb-4 prose-h3:text-xl prose-h3:font-medium prose-h3:mt-8 prose-h3:mb-3 prose-p:text-base-content/80 prose-p:leading-relaxed prose-a:text-primary prose-a:no-underline hover:prose-a:underline prose-code:bg-base-200 prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:text-sm">
                                    doc_content(nodes: &post.content, base_path: base_path)
                                </div>
                                if !related.is_empty() {
                                    <section class="mt-16 pt-8 border-t border-base-300">
                                        <h2 class="text-xl font-semibold mb-6">"Related Posts"</h2>
                                        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                                            for post in &related {
                                                blog_card(registry: registry, base_path: base_path, post: post)
                                            }
                                        </div>
                                    </section>
                                }
                                <nav class="mt-16 pt-8 border-t border-base-300 flex justify-between gap-4">
                                    <div class="flex-1">
                                        if let Some(prev) = prev {
                                            <a href=(format!("{base_path}/{}", prev.slug)) class="group flex flex-col p-4 rounded-lg border border-base-300 hover:border-primary/50 hover:bg-base-200/50 transition-all">
                                                <span class="text-xs text-base-content/50 mb-1">"Older"</span>
                                                <span class="font-medium group-hover:text-primary transition-colors">(prev.frontmatter.title.as_str())</span>
                                            </a>
                                        }
                                    </div>
                                    <div class="flex-1 text-right">
                                        if let Some(next) = next {
                                            <a href=(format!("{base_path}/{}", next.slug)) class="group flex flex-col p-4 rounded-lg border border-base-300 hover:border-primary/50 hover:bg-base-200/50 transition-all items-end">
                                                <span class="text-xs text-base-content/50 mb-1">"Newer"</span>
                                                <span class="font-medium group-hover:text-primary transition-colors">(next.frontmatter.title.as_str())</span>
                                            </a>
                                        }
                                    </div>
                                </nav>
                            </article>
                        </main>
                        toc(headers: &headers)
                    </div>
                },
            }
        )
    })
}

/// A category page (`BlogConfig::with_category_base_path`): title,
/// description, the category filter and one page of the category's posts.
/// Unknown categories and out-of-range pages render a 404.
#[component]
pub async fn blog_category_page(
    registry: &BlogRegistry,
    base_path: &str,
    slug: &str,
    page: usize,
) -> Result<impl View> {
    let found = registry
        .get_category(slug)
        .zip(registry.category_posts_page(slug, page));
    let total_pages = found
        .as_ref()
        .map(|(c, _)| registry.total_pages_for_tag(&c.tag))
        .unwrap_or(0);

    Ok(view! {
        blog_frame(registry: registry, base_path: base_path, query: "",
            match found {
                None => {
                    (StatusCode::NOT_FOUND)
                    <main class="max-w-6xl mx-auto px-4 py-12">
                        <h1 class="text-4xl font-bold mb-4">"Category page not found"</h1>
                        <p class="text-base-content/70 mb-8">"This topic or page doesn't exist."</p>
                        <a href=(base_path) class="btn btn-primary">"Back to Blog"</a>
                    </main>
                },
                Some((category, posts)) => {
                    <main class="dk-blog-category max-w-6xl mx-auto px-4 py-12">
                        <header class="max-w-3xl mb-8">
                            <a href=(base_path) class="text-sm text-base-content/60 hover:text-primary">"Blog"</a>
                            <h1 class="text-4xl font-bold tracking-tight mt-4 mb-3">(category.title.as_str())</h1>
                            <p class="text-lg text-base-content/60 leading-relaxed">(category.description.as_str())</p>
                            <p class="text-sm text-base-content/50 mt-4">(registry.tag_count(&category.tag)) " articles"</p>
                            if let Some(image) = &category.image {
                                <img src=(image.as_str()) alt="" class="w-full rounded-xl mt-6" loading="lazy">
                            }
                        </header>
                        <div class="mb-8">tag_filter(registry: registry, base_path: base_path, active_tag: None, active_category: Some(slug))</div>
                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                            for post in &posts {
                                blog_card(registry: registry, base_path: base_path, post: post)
                            }
                        </div>
                        if total_pages > 1 {
                            <nav class="flex flex-wrap items-center justify-center gap-2 mt-12" aria-label="Category pagination">
                                if let Some(url) = registry.category_url(slug, page.saturating_sub(1)) {
                                    <a href=(url) class="btn btn-ghost btn-sm">"Previous"</a>
                                }
                                <span class="text-sm text-base-content/60">"Page " (page) " of " (total_pages)</span>
                                if let Some(url) = registry.category_url(slug, page + 1) {
                                    <a href=(url) class="btn btn-ghost btn-sm">"Next"</a>
                                }
                            </nav>
                        }
                    </main>
                },
            }
        )
    })
}

/// Blog search results for `query` (the header search box submits to
/// `<base_path>/search?q=…`).
#[component]
pub async fn blog_search_page(
    registry: &BlogRegistry,
    base_path: &str,
    query: &str,
) -> Result<impl View> {
    let terms = split_terms(query);
    let hits: Vec<_> = registry
        .search_posts(query)
        .into_iter()
        .take(MAX_RESULTS)
        .collect();
    let blank = query.trim().is_empty();
    let summary = match hits.len() {
        1 => "1 result for".to_string(),
        n => format!("{n} results for"),
    };

    Ok(view! {
        blog_frame(registry: registry, base_path: base_path, query: query,
            <main class="max-w-3xl mx-auto px-4 py-12">
                <h1 class="text-4xl font-bold tracking-tight mb-3">"Search the blog"</h1>
                if blank {
                    <p class="text-base-content/70">"Type a word or phrase into the search box above."</p>
                } else {
                    <p class="text-base-content/70 mb-8">(summary) " \u{201c}" (query) "\u{201d}"</p>
                    if hits.is_empty() {
                        <p class="text-base-content/50">"No posts match. Try fewer or different words."</p>
                    }
                    <ul class="dk-search-results divide-y divide-base-300">
                        for hit in &hits {
                            let source = if hit.body.is_empty() { &hit.description } else { &hit.body };
                            let snippet = build_snippet(source, &terms, SNIPPET_WINDOW);
                            <li>
                                <a href=(format!("{base_path}/{}", hit.slug)) class="dk-search-result block py-4 group">
                                    <span class="font-medium group-hover:text-primary transition-colors">(hit.title.as_str())</span>
                                    <div class="flex flex-wrap items-center gap-2 mt-0.5 text-xs text-base-content/50">
                                        <span>(registry.format_date(&hit.date))</span>
                                        for tag in &hit.tags {
                                            <span class="badge badge-xs badge-outline badge-primary">(tag.as_str())</span>
                                        }
                                    </div>
                                    if !snippet.is_empty() {
                                        <p class="dk-search-snippet text-sm text-base-content/70 mt-1.5">
                                            for seg in &snippet {
                                                if seg.highlight {
                                                    <mark class="dk-search-mark bg-primary/20 text-primary rounded-sm px-0.5">(seg.text.as_str())</mark>
                                                } else {
                                                    (seg.text.as_str())
                                                }
                                            }
                                        </p>
                                    }
                                </a>
                            </li>
                        }
                    </ul>
                }
            </main>
        )
    })
}

// ============================================================================
// <head>
// ============================================================================

/// `<title>`, description, Open Graph and Twitter tags for the blog index.
/// Pass `site_url` (e.g. `"https://example.com"`) for canonical and `og:url`.
#[component]
pub async fn blog_index_head(
    base_path: &str,
    tag: Option<&str>,
    page: usize,
    site_url: Option<&str>,
) -> Result<impl View> {
    let (mut title, description) = match tag {
        Some(tag) => (
            format!("Blog: {tag}"),
            format!("Browse blog posts tagged {tag}."),
        ),
        None => (
            "Blog".to_string(),
            "Latest blog posts and updates.".to_string(),
        ),
    };
    if page > 1 {
        title = format!("{title} — Page {page}");
    }
    let canonical = site_url.map(|site| join_site_url(site, &index_url(base_path, tag, page), ""));
    Ok(
        view! { listing_meta(title: &title, description: &description, og_type: "website", canonical: canonical.as_deref(), image: None) },
    )
}

/// Head tags for a post: the listing tags plus `og:type=article`,
/// `article:*` metadata and Article JSON-LD. Nothing for an unknown slug.
#[component]
pub async fn blog_post_head(
    registry: &BlogRegistry,
    base_path: &str,
    slug: &str,
    site_url: Option<&str>,
) -> Result<impl View> {
    let post = registry.get_post(slug);
    let canonical = site_url.map(|site| join_site_url(site, base_path, slug));
    Ok(view! {
        match post {
            None => <title>"Post not found"</title>,
            Some(post) => {
                let description = post.frontmatter.description.as_deref().unwrap_or("");
                let author = registry.get_author(&post.frontmatter.author).map(|a| a.name.as_str()).unwrap_or("");
                let cover = post.frontmatter.cover_image.as_deref();
                listing_meta(title: &post.frontmatter.title, description: description, og_type: "article", canonical: canonical.as_deref(), image: cover)
                <meta property="article:published_time" content=(post.frontmatter.date.as_str())>
                if !author.is_empty() {
                    <meta property="article:author" content=(author)>
                }
                for tag in &post.frontmatter.tags {
                    <meta property="article:tag" content=(tag.as_str())>
                }
                <script type="application/ld+json">
                    // `article_jsonld` escapes `</`, so the payload cannot close the tag.
                    (Unescaped::new_unchecked(article_jsonld(&post.frontmatter.title, description, canonical.as_deref(), &post.frontmatter.date, author, cover)))
                </script>
            },
        }
    })
}

/// Head tags for a category page.
#[component]
pub async fn blog_category_head(
    registry: &BlogRegistry,
    slug: &str,
    page: usize,
    site_url: Option<&str>,
) -> Result<impl View> {
    let category = registry.get_category(slug);
    Ok(view! {
        match category {
            None => <title>"Category page not found"</title>,
            Some(category) => {
                let title = if page > 1 { format!("{} — Page {page}", category.title) } else { category.title.clone() };
                let canonical = site_url.zip(registry.category_url(slug, page)).map(|(site, path)| join_site_url(site, &path, ""));
                let image = category.image.as_deref().map(|image| match site_url {
                    Some(site) if image.starts_with('/') && !image.starts_with("//") => join_site_url(site, image, ""),
                    _ => image.to_string(),
                });
                listing_meta(title: &title, description: &category.description, og_type: "website", canonical: canonical.as_deref(), image: image.as_deref())
            },
        }
    })
}

/// The shared title/description/Open Graph/Twitter block; `og_type` is
/// `website` for listings and `article` for posts.
#[component]
async fn listing_meta(
    title: &str,
    description: &str,
    og_type: &str,
    canonical: Option<&str>,
    image: Option<&str>,
) -> Result<impl View> {
    Ok(view! {
        <title>(title)</title>
        <meta name="description" content=(description)>
        <meta property="og:title" content=(title)>
        <meta property="og:description" content=(description)>
        <meta property="og:type" content=(og_type)>
        <meta name="twitter:card" content=(if image.is_some() { "summary_large_image" } else { "summary" })>
        <meta name="twitter:title" content=(title)>
        <meta name="twitter:description" content=(description)>
        if let Some(url) = canonical {
            <link rel="canonical" href=(url)>
            <meta property="og:url" content=(url)>
        }
        if let Some(image) = image {
            <meta property="og:image" content=(image)>
            <meta name="twitter:image" content=(image)>
        }
    })
}

// ============================================================================
// Pieces
// ============================================================================

/// Header (blog home link, search box, theme toggle) around `child`.
#[component]
async fn blog_frame(
    registry: &BlogRegistry,
    base_path: &str,
    query: &str,
    child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <div class="dk-root dk-blog-root min-h-screen bg-base-100">
            <div class="dk-header sticky top-0 z-50 flex items-center gap-4 bg-base-200/80 backdrop-blur border-b border-base-300 px-4 lg:px-8">
                <a href=(base_path) class="flex-1 min-w-0 py-2.5 text-sm font-semibold">"Blog"</a>
                <form class="dk-search-form shrink-0 py-1.5" action=(format!("{base_path}/search")) method="get" role="search">
                    <label class="input input-sm w-36 sm:w-56">
                        icon(glyph: LdSearch, class: "size-4 opacity-50")
                        <input type="search" name="q" value=(query) placeholder="Search posts" aria-label="Search the blog">
                    </label>
                </form>
                theme_toggle(theme: registry.theme.as_ref())
            </div>
            <div class="flex-1 min-w-0">(child)</div>
        </div>
    })
}

#[component]
async fn blog_card(registry: &BlogRegistry, base_path: &str, post: &BlogPost) -> Result<impl View> {
    let href = format!("{base_path}/{}", post.slug);
    let author = registry.get_author(&post.frontmatter.author);
    let tags = &post.frontmatter.tags;
    Ok(view! {
        <article class="group flex flex-col rounded-xl border border-base-300 bg-base-200/30 hover:border-primary/30 hover:shadow-lg transition-all duration-200 overflow-hidden">
            if let Some(cover) = &post.frontmatter.cover_image {
                <a href=(href.as_str()) class="aspect-video overflow-hidden bg-base-300">
                    <img src=(cover.as_str()) alt=(post.frontmatter.title.as_str()) class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300">
                </a>
            }
            <div class="flex flex-col flex-1 p-5 gap-3">
                if !tags.is_empty() {
                    <div class="flex flex-wrap gap-1.5">
                        for tag in tags.iter().take(3) {
                            <a href=(tag_url(registry, base_path, tag)) class="badge badge-sm badge-outline hover:badge-primary">(tag.as_str())</a>
                        }
                        if tags.len() > 3 {
                            <span class="badge badge-sm badge-ghost">"+" (tags.len() - 3)</span>
                        }
                    </div>
                }
                <h2 class="text-lg font-semibold leading-snug group-hover:text-primary transition-colors line-clamp-2">
                    <a href=(href.as_str())>(post.frontmatter.title.as_str())</a>
                </h2>
                if let Some(desc) = &post.frontmatter.description {
                    <p class="text-sm text-base-content/60 leading-relaxed line-clamp-3">(desc.as_str())</p>
                }
                <div class="mt-auto pt-3 border-t border-base-300/60 flex items-center gap-3 text-xs text-base-content/50">
                    if let Some(author) = author {
                        <div class="flex items-center gap-1.5">
                            if let Some(avatar) = &author.avatar {
                                <img src=(avatar.as_str()) alt=(author.name.as_str()) class="size-5 rounded-full">
                            }
                            <span>(author.name.as_str())</span>
                        </div>
                    }
                    <span>(registry.format_date(&post.frontmatter.date))</span>
                    <div class="flex items-center gap-1 ml-auto">
                        icon(glyph: LdClock, class: "size-3")
                        <span>(post.reading_time_minutes) " min read"</span>
                    </div>
                </div>
            </div>
        </article>
    })
}

/// Category links when the registry has categories, otherwise tag links
/// (`?tag=`). Each shows its post count.
#[component]
async fn tag_filter(
    registry: &BlogRegistry,
    base_path: &str,
    active_tag: Option<&str>,
    active_category: Option<&str>,
) -> Result<impl View> {
    let pill = |active: bool| {
        if active {
            "btn btn-sm btn-primary"
        } else {
            "btn btn-sm btn-ghost"
        }
    };
    Ok(view! {
        if registry.category_base_path().is_some() {
            <nav class="flex flex-wrap gap-2" aria-label="Blog categories">
                <a href=(base_path) class=(pill(active_category.is_none() && active_tag.is_none()))>"All"</a>
                for category in registry.categories() {
                    if let Some(href) = registry.category_url(&category.slug, 1) {
                        let active = active_category == Some(category.slug.as_str());
                        <a href=(href) class=(pill(active)) aria-current=(active.then_some("page"))>
                            (category.title.as_str())
                            <span class="ml-1 text-xs opacity-50">(registry.tag_count(&category.tag))</span>
                        </a>
                    }
                }
            </nav>
        } else {
            <nav class="flex flex-wrap gap-2" aria-label="Blog tags">
                <a href=(base_path) class=(pill(active_tag.is_none()))>"All"</a>
                for tag in registry.all_tags() {
                    let active = active_tag == Some(tag.as_str());
                    // Clicking the active tag clears the filter.
                    <a href=(if active { base_path.to_string() } else { index_url(base_path, Some(tag), 1) }) class=(pill(active)) aria-current=(active.then_some("page"))>
                        (tag.as_str())
                        <span class="ml-1 text-xs opacity-50">(registry.tag_count(tag))</span>
                    </a>
                }
            </nav>
        }
    })
}

#[component]
async fn author_info(registry: &BlogRegistry, author_id: &str) -> Result<impl View> {
    let author = registry.get_author(author_id);
    Ok(view! {
        match author {
            None => <span class="text-sm text-base-content/50">(author_id)</span>,
            Some(author) => {
                let inner = view! {
                    <div class="flex items-center gap-2">
                        if let Some(avatar) = &author.avatar {
                            <img src=(avatar.as_str()) alt=(author.name.as_str()) class="size-6 rounded-full">
                        }
                        <span class="font-medium">(author.name.as_str())</span>
                    </div>
                };
                match &author.url {
                    Some(url) => <a href=(url.as_str()) target="_blank" rel="noopener noreferrer" class="hover:text-primary transition-colors">(inner)</a>,
                    None => (inner),
                }
            },
        }
    })
}

// ============================================================================
// URLs
// ============================================================================

/// A post tag's link: its category page when one exists, else the index
/// filtered by the tag.
fn tag_url(registry: &BlogRegistry, base_path: &str, tag: &str) -> String {
    registry
        .category_url_for_tag(tag)
        .unwrap_or_else(|| index_url(base_path, Some(tag), 1))
}

/// `<base_path>[?tag=…][&page=…]`; page one has no `page` parameter.
fn index_url(base_path: &str, tag: Option<&str>, page: usize) -> String {
    let mut params = Vec::new();
    if let Some(tag) = tag {
        params.push(format!("tag={}", encode_query_value(tag)));
    }
    if page > 1 {
        params.push(format!("page={page}"));
    }
    if params.is_empty() {
        base_path.to_string()
    } else {
        format!("{base_path}?{}", params.join("&"))
    }
}

/// Percent-encode everything but RFC 3986 unreserved characters.
fn encode_query_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{encode_query_value, index_url};

    #[test]
    fn index_urls_drop_defaults_and_encode_tags() {
        assert_eq!(index_url("/blog", None, 1), "/blog");
        assert_eq!(index_url("/blog", None, 2), "/blog?page=2");
        assert_eq!(
            index_url("/blog", Some("web dev"), 3),
            "/blog?tag=web%20dev&page=3"
        );
        assert_eq!(encode_query_value("c++&ü"), "c%2B%2B%26%C3%BC");
    }
}
