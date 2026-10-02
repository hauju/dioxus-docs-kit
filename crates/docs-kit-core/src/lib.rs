//! # docs-kit-core
//!
//! The UI-framework-agnostic half of `dioxus-docs-kit`: it loads the content
//! bundle written by `dioxus-docs-kit-build` and answers every question a docs
//! or blog shell asks of it — page lookup, navigation, search, OpenAPI
//! operations, `llms.txt`, sitemaps and feeds. It renders nothing, so any UI
//! layer can sit on top.
//!
//! ```rust,ignore
//! use docs_kit_core::{DocsConfig, DocsRegistry};
//! use std::sync::LazyLock;
//!
//! static DOCS: LazyLock<DocsRegistry> = LazyLock::new(|| {
//!     DocsConfig::new(docs_kit_core::docs_bundle!())
//!         .with_default_path("getting-started/introduction")
//!         .build()
//! });
//! ```

pub mod blog;
pub(crate) mod bundle;
pub mod config;
pub mod error;
pub mod registry;
pub mod search;
pub mod seo;

pub use blog::{
    Author, BlogCategory, BlogCategoryMetadata, BlogConfig, BlogFrontmatter, BlogPost,
    BlogRegistry, BlogSearchEntry,
};
pub use config::{DocsConfig, ThemeConfig};
pub use error::DocsKitError;
pub use registry::{ApiEndpointEntry, DocsHit, DocsRegistry, NavConfig, NavGroup, SearchEntry};

/// Embeds the docs bundle written by `dioxus-docs-kit-build` as a
/// `&'static str`.
///
/// ```rust,ignore
/// static DOCS: LazyLock<DocsRegistry> = LazyLock::new(|| {
///     DocsConfig::new(docs_kit_core::docs_bundle!()).build()
/// });
/// ```
///
/// Requires `dioxus-docs-kit-build` in `[build-dependencies]` and a `build.rs`
/// that calls `dioxus_docs_kit_build::DocsBuild::new("docs/_nav.json").generate()`.
#[macro_export]
macro_rules! docs_bundle {
    () => {
        include_str!(concat!(env!("OUT_DIR"), "/docs_bundle.json"))
    };
}

/// Embeds the blog bundle written by `dioxus-docs-kit-build` as a
/// `&'static str`.
///
/// ```rust,ignore
/// static BLOG: LazyLock<BlogRegistry> = LazyLock::new(|| {
///     BlogConfig::new(docs_kit_core::blog_bundle!()).build()
/// });
/// ```
///
/// Requires `dioxus-docs-kit-build` in `[build-dependencies]` and a `build.rs`
/// that calls `dioxus_docs_kit_build::BlogBuild::new("blog/_blog.json").generate()`.
#[macro_export]
macro_rules! blog_bundle {
    () => {
        include_str!(concat!(env!("OUT_DIR"), "/blog_bundle.json"))
    };
}
