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
        response::IntoResponse, route,
    },
    view::{View, component, view},
};
use topcoat_docs_kit::{DocsConfig, DocsRegistry, STYLESHEET, docs_head, docs_page};

static DOCS: LazyLock<DocsRegistry> =
    LazyLock::new(|| DocsConfig::new(topcoat_docs_kit::docs_bundle!()).build());

#[tokio::main]
async fn main() {
    topcoat::start(router()).await.unwrap();
}

fn router() -> Router {
    Router::builder()
        .route(home)
        .route(stylesheet)
        .page(docs_index)
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
    Ok(view! { document(path: "") })
}

path_param!(*doc_path);

#[page("/docs/{*doc_path}")]
async fn docs(cx: &Cx) -> Result<impl View> {
    let segments: CatchAllSegments<'_> = path_param::<DocPath>(cx);
    let path = segments.collect::<Vec<_>>().join("/");
    Ok(view! { document(path: &path) })
}

#[component]
async fn document(path: &str) -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en" data-theme="dark">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                docs_head(registry: &DOCS, path: path)
                <link rel="stylesheet" href="/docs-kit.css">
            </head>
            <body>
                docs_page(registry: &DOCS, base_path: "/docs", path: path)
            </body>
        </html>
    })
}
