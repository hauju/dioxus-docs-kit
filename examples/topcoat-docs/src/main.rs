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
    DocsConfig, DocsRegistry, STYLESHEET, docs_head, docs_page, docs_search_head, docs_search_page,
    docs_theme_script,
};

static DOCS: LazyLock<DocsRegistry> = LazyLock::new(|| {
    DocsConfig::new(topcoat_docs_kit::docs_bundle!())
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
            head: view! { docs_head(registry: &DOCS, path: path) }.into(),
            docs_page(registry: &DOCS, base_path: "/docs", path: path)
        )
    })
}

#[component]
async fn document(head: Child<'_>, child: Child<'_>) -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en" data-theme="dark">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                docs_theme_script(registry: &DOCS)
                (head)
                <link rel="stylesheet" href="/docs-kit.css">
            </head>
            <body>(child)</body>
        </html>
    })
}
