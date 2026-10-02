//! # topcoat-docs-kit
//!
//! Documentation site shell for [Topcoat](https://github.com/tokio-rs/topcoat)
//! apps, on the same build-time content pipeline as `dioxus-docs-kit`: write
//! MDX + `_nav.json`, let `dioxus-docs-kit-build` bundle it in `build.rs`, and
//! render it here with server-side components. No runtime script is required —
//! tabs, accordions and copy buttons work in plain HTML.
//!
//! ```rust,ignore
//! // build.rs
//! dioxus_docs_kit_build::DocsBuild::new("docs/_nav.json").generate();
//! ```
//!
//! ```rust,ignore
//! use std::sync::LazyLock;
//! use topcoat_docs_kit::{DocsConfig, DocsRegistry, docs_head, docs_page};
//!
//! static DOCS: LazyLock<DocsRegistry> =
//!     LazyLock::new(|| DocsConfig::new(topcoat_docs_kit::docs_bundle!()).build());
//!
//! #[page]
//! async fn docs(cx: &Cx) -> Result<impl View> {
//!     let path = path_param::<Rest>(cx)?.join("/");
//!     Ok(view! {
//!         <!DOCTYPE html>
//!         <html data-theme="dark">
//!             <head>
//!                 docs_head(registry: &DOCS, path: &path)
//!                 <link rel="stylesheet" href="/docs-kit.css">
//!             </head>
//!             <body>docs_page(registry: &DOCS, base_path: "/docs", path: &path)</body>
//!         </html>
//!     })
//! }
//! ```
//!
//! Serve [`STYLESHEET`] at the `href` you link (see the `topcoat-docs` example).
//!
//! Not ported yet: OpenAPI endpoint pages, search, the blog, and the theme
//! toggle.

mod content;
mod icon;
mod shell;

pub use content::doc_content;
pub use shell::{docs_head, docs_page};

pub use docs_kit_core::{DocsConfig, DocsRegistry, docs_bundle};

/// Precompiled stylesheet covering every class these components emit
/// (Tailwind utilities, DaisyUI dark/light themes, typography prose and the
/// `--dk-*` tokens). Serve it as `text/css`.
pub const STYLESHEET: &str = include_str!("../assets/docs-kit.css");
