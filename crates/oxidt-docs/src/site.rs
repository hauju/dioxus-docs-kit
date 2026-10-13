//! Content loading and the routes that render it, shared by `build` and `dev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use serde::Deserialize;
use topcoat::{
    Result,
    context::Cx,
    router::{
        CatchAllSegments, HeaderValue, Router, error::redirect, header, page, path_param,
        query_params, response::IntoResponse, route,
    },
    view::{Child, Unescaped, View, component, view},
};
use topcoat_docs_kit::{
    DocsConfig, DocsRegistry, STYLESHEET, docs_head, docs_page, docs_search_head, docs_search_page,
    docs_theme_script,
};

#[derive(Clone)]
pub struct Options {
    /// Project root; content is read from `<root>/docs`.
    pub root: PathBuf,
    /// `(url prefix, spec path relative to root)` pairs.
    pub openapi: Vec<(String, String)>,
    /// Public origin, e.g. `https://docs.example.com`; enables sitemap.xml + robots.txt.
    pub site_url: Option<String>,
}

#[derive(Deserialize)]
struct Nav {
    groups: Vec<NavGroup>,
}

#[derive(Deserialize)]
struct NavGroup {
    pages: Vec<String>,
}

/// The registry routes render. Topcoat views need `&'static`, so each load
/// leaks its registry; `dev` reloads leak the previous one.
static REGISTRY: RwLock<Option<&'static DocsRegistry>> = RwLock::new(None);

/// Set by `dev`: pages poll [`VERSION`] and reload when content changes.
pub static LIVE_RELOAD: AtomicBool = AtomicBool::new(false);
pub static VERSION: AtomicU64 = AtomicU64::new(0);

pub fn registry() -> &'static DocsRegistry {
    REGISTRY
        .read()
        .unwrap()
        .expect("content is loaded before serving")
}

/// Parse `<root>/docs` into a registry and make it the one routes render.
/// Returns the content bundle JSON the registry was built from.
pub fn load(opts: &Options) -> Result<&'static str, String> {
    let docs_dir = opts.root.join("docs");
    let nav_json = read(&docs_dir.join("_nav.json"))?;
    let nav: Nav = serde_json::from_str(&nav_json).map_err(|e| format!("docs/_nav.json: {e}"))?;

    let mut pages = Vec::new();
    for page in nav.groups.iter().flat_map(|g| &g.pages) {
        pages.push((page.as_str(), read(&docs_dir.join(format!("{page}.mdx")))?));
    }
    let mut specs = Vec::new();
    for (prefix, spec) in &opts.openapi {
        specs.push((prefix.as_str(), read(&opts.root.join(spec))?));
    }

    let bundle =
        dioxus_docs_kit_build::docs_bundle_json(&nav_json, &pairs(&pages), &pairs(&specs))?;
    let bundle: &'static str = Box::leak(bundle.into_boxed_str());
    let registry = DocsConfig::new(bundle)
        .with_theme_toggle("light", "dark", "dark")
        .try_build()
        .map_err(|e| e.to_string())?;
    *REGISTRY.write().unwrap() = Some(Box::leak(Box::new(registry)));
    VERSION.fetch_add(1, Ordering::Relaxed);
    Ok(bundle)
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn pairs<'a>(owned: &'a [(&'a str, String)]) -> Vec<(&'a str, &'a str)> {
    owned.iter().map(|(k, v)| (*k, v.as_str())).collect()
}

pub fn router() -> Router {
    Router::builder()
        .route(home)
        .route(stylesheet)
        .route(content_version)
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

#[route(GET "/__oxidt/version")]
async fn content_version() -> Result<impl IntoResponse> {
    Ok(VERSION.load(Ordering::Relaxed).to_string())
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
    let registry = registry();
    Ok(view! {
        document(
            head: view! { docs_search_head(query: query) }.into(),
            docs_search_page(registry: registry, base_path: "/docs", query: query)
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
    let registry = registry();
    Ok(view! {
        document(
            head: view! { docs_head(registry: registry, path: path) }.into(),
            docs_page(registry: registry, base_path: "/docs", path: path)
        )
    })
}

/// Polls the dev server and reloads once content was rebuilt since this page
/// rendered; `data-version` carries the version the page was rendered from.
const LIVE_RELOAD_SCRIPT: &str = "(()=>{const v=document.currentScript.dataset.version;setInterval(async()=>{try{if(await(await fetch('/__oxidt/version')).text()!==v)location.reload()}catch{}},1000)})()";

#[component]
async fn document(head: Child<'_>, child: Child<'_>) -> Result<impl View> {
    let live_reload = LIVE_RELOAD.load(Ordering::Relaxed);
    let version = VERSION.load(Ordering::Relaxed).to_string();
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en" data-theme="dark">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                docs_theme_script(theme: registry().theme.as_ref())
                (head)
                <link rel="stylesheet" href="/docs-kit.css">
                if live_reload {
                    <script data-version=(version.as_str())>(Unescaped::new_unchecked(LIVE_RELOAD_SCRIPT))</script>
                }
            </head>
            <body>(child)</body>
        </html>
    })
}
