//! The repo's docs, rendered by `topcoat-docs-kit`.
//!
//! ```sh
//! cargo run -p topcoat-docs-example   # http://127.0.0.1:3000/docs
//! ```

use std::sync::LazyLock;

use topcoat::{
    Result,
    context::Cx,
    router::{
        CatchAllSegments, HeaderValue, Router, error::redirect, header, page, path_param,
        query_params, response::IntoResponse, route,
    },
    view::{Child, View, component, view},
};
use topcoat_docs_kit::{
    BlogConfig, BlogRegistry, DocsConfig, DocsRegistry, STYLESHEET, ThemeConfig,
    blog_category_head, blog_category_page, blog_index_head, blog_index_page, blog_post_head,
    blog_post_page, blog_search_page, docs_head, docs_page, docs_search_head, docs_search_page,
    docs_theme_script,
};

static DOCS: LazyLock<DocsRegistry> = LazyLock::new(|| {
    DocsConfig::new(topcoat_docs_kit::docs_bundle!())
        .with_theme_toggle("light", "dark", "dark")
        .build()
});

static BLOG: LazyLock<BlogRegistry> = LazyLock::new(|| {
    BlogConfig::new(topcoat_docs_kit::blog_bundle!())
        .with_posts_per_page(9)
        .with_category_base_path("/blog/categories")
        .with_theme_toggle("light", "dark", "dark")
        .build()
});

#[tokio::main]
async fn main() {
    topcoat::start(router()).await.unwrap();
}

fn router() -> Router {
    Router::builder()
        .route(home)
        .route(stylesheet)
        .page(docs_index)
        .page(search)
        .page(docs)
        .page(blog_index)
        .page(blog_search)
        .page(blog_category)
        .page(blog_category_paged)
        .page(blog_post)
        .build()
}

#[route(GET "/")]
async fn home() -> Result<()> {
    Err(redirect("/docs").into())
}

#[route(GET "/docs-kit.css")]
async fn stylesheet() -> Result<impl IntoResponse> {
    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/css; charset=utf-8"),
        )],
        STYLESHEET,
    ))
}

#[page("/docs")]
async fn docs_index() -> Result<impl View> {
    Ok(view! { docs_document(path: "") })
}

#[query_params(error = bad_request)]
struct SearchQuery {
    q: Option<String>,
}

// Registered before the catch-all below, and more specific than it.
#[page("/docs/search")]
async fn search(cx: &Cx) -> Result<impl View> {
    let query = query_params::<SearchQuery>(cx)?
        .q
        .clone()
        .unwrap_or_default();
    Ok(view! { search_document(query: &query) })
}

#[component]
async fn search_document(query: &str) -> Result<impl View> {
    Ok(view! {
        document(
            theme: DOCS.theme.as_ref(),
            head: view! { docs_search_head(query: query) }.into(),
            docs_search_page(registry: &DOCS, base_path: "/docs", query: query)
        )
    })
}

path_param!(*doc_path);

#[page("/docs/{*doc_path}")]
async fn docs(cx: &Cx) -> Result<impl View> {
    let segments: CatchAllSegments<'_> = path_param::<DocPath>(cx);
    let path = segments.collect::<Vec<_>>().join("/");
    Ok(view! { docs_document(path: &path) })
}

#[component]
async fn docs_document(path: &str) -> Result<impl View> {
    Ok(view! {
        document(
            theme: DOCS.theme.as_ref(),
            head: view! { docs_head(registry: &DOCS, path: path) }.into(),
            docs_page(registry: &DOCS, base_path: "/docs", path: path)
        )
    })
}

// --- Blog --------------------------------------------------------------------

#[query_params(error = bad_request)]
struct BlogQuery {
    tag: Option<String>,
    page: Option<usize>,
}

#[page("/blog")]
async fn blog_index(cx: &Cx) -> Result<impl View> {
    let query = query_params::<BlogQuery>(cx)?;
    let tag = query.tag.clone();
    let page = query.page.unwrap_or(1);
    Ok(view! { blog_index_document(tag: tag.as_deref(), page: page) })
}

#[component]
async fn blog_index_document(tag: Option<&str>, page: usize) -> Result<impl View> {
    Ok(view! {
        document(
            theme: BLOG.theme.as_ref(),
            head: view! { blog_index_head(base_path: "/blog", tag: tag, page: page, site_url: None) }.into(),
            blog_index_page(registry: &BLOG, base_path: "/blog", tag: tag, page: page)
        )
    })
}

#[page("/blog/search")]
async fn blog_search(cx: &Cx) -> Result<impl View> {
    let query = query_params::<SearchQuery>(cx)?
        .q
        .clone()
        .unwrap_or_default();
    Ok(view! { blog_search_document(query: &query) })
}

#[component]
async fn blog_search_document(query: &str) -> Result<impl View> {
    Ok(view! {
        document(
            theme: BLOG.theme.as_ref(),
            head: view! { docs_search_head(query: query) }.into(),
            blog_search_page(registry: &BLOG, base_path: "/blog", query: query)
        )
    })
}

path_param!(category);
path_param!(category_page: usize, error = not_found);

#[page("/blog/categories/{category}")]
async fn blog_category(cx: &Cx) -> Result<impl View> {
    let slug = path_param::<Category>(cx).to_string();
    Ok(view! { blog_category_document(slug: &slug, page: 1) })
}

#[page("/blog/categories/{category}/page/{category_page}")]
async fn blog_category_paged(cx: &Cx) -> Result<impl View> {
    let slug = path_param::<Category>(cx).to_string();
    let page = *path_param::<CategoryPage>(cx)?;
    Ok(view! { blog_category_document(slug: &slug, page: page) })
}

#[component]
async fn blog_category_document(slug: &str, page: usize) -> Result<impl View> {
    Ok(view! {
        document(
            theme: BLOG.theme.as_ref(),
            head: view! { blog_category_head(registry: &BLOG, slug: slug, page: page, site_url: None) }.into(),
            blog_category_page(registry: &BLOG, base_path: "/blog", slug: slug, page: page)
        )
    })
}

path_param!(post_slug);

#[page("/blog/{post_slug}")]
async fn blog_post(cx: &Cx) -> Result<impl View> {
    let slug = path_param::<PostSlug>(cx).to_string();
    Ok(view! { blog_post_document(slug: &slug) })
}

#[component]
async fn blog_post_document(slug: &str) -> Result<impl View> {
    Ok(view! {
        document(
            theme: BLOG.theme.as_ref(),
            head: view! { blog_post_head(registry: &BLOG, base_path: "/blog", slug: slug, site_url: None) }.into(),
            blog_post_page(registry: &BLOG, base_path: "/blog", slug: slug)
        )
    })
}

// --- Document ----------------------------------------------------------------

#[component]
async fn document(
    theme: Option<&ThemeConfig>,
    head: Child<'_>,
    child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en" data-theme="dark">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                docs_theme_script(theme: theme)
                (head)
                <link rel="stylesheet" href="/docs-kit.css">
            </head>
            <body>(child)</body>
        </html>
    })
}
