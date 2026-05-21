//! # dioxus-docs-kit
//!
//! Reusable documentation site shell and blog engine for Dioxus applications.
//!
//! Provides a complete docs layout with sidebar navigation, search modal,
//! page navigation, OpenAPI API reference pages, and mobile drawer.
//! Also includes a full blog engine with post listing, tag filtering,
//! search, reading time, and MDX rendering.
//!
//! ## Quick Start — Docs
//!
//! ```rust,ignore
//! use dioxus::prelude::*;
//! use dioxus_docs_kit::{DocsConfig, DocsRegistry, DocsContext, DocsLayout, DocsPageContent};
//! use std::sync::LazyLock;
//!
//! static DOCS: LazyLock<DocsRegistry> = LazyLock::new(|| {
//!     DocsConfig::new(include_str!("../docs/_nav.json"), doc_content_map())
//!         .with_default_path("getting-started/introduction")
//!         .build()
//! });
//! ```
//!
//! ## Quick Start — Blog
//!
//! ```rust,ignore
//! use dioxus::prelude::*;
//! use dioxus_docs_kit::{BlogConfig, BlogRegistry, BlogContext, BlogLayout, BlogList, BlogPostView};
//! use std::sync::LazyLock;
//!
//! dioxus_docs_kit::blog_content_map!();
//!
//! static BLOG: LazyLock<BlogRegistry> = LazyLock::new(|| {
//!     BlogConfig::new(include_str!("../blog/_blog.json"), blog_content_map())
//!         .with_posts_per_page(9)
//!         .build()
//! });
//! ```

pub mod blog;
pub mod components;
pub mod config;
pub mod hooks;
pub mod registry;

use dioxus::prelude::*;

// ============================================================================
// Docs context
// ============================================================================

/// Navigation bridge that decouples library components from the consumer's Route enum.
///
/// The consumer creates this in their docs layout wrapper and provides it via `use_context_provider`.
#[derive(Clone)]
pub struct DocsContext {
    /// Current docs page path (e.g. "getting-started/introduction").
    pub current_path: ReadSignal<String>,
    /// Base URL path for docs (e.g. "/docs").
    pub base_path: String,
    /// Callback to navigate to a docs page by content path.
    pub navigate: Callback<String>,
}

// ============================================================================
// Blog context
// ============================================================================

/// Navigation bridge for blog pages, decoupled from the consumer's Route enum.
///
/// The consumer creates this in their blog layout wrapper and provides it via `use_context_provider`.
#[derive(Clone)]
pub struct BlogContext {
    /// Current blog post slug (empty on the list/index page).
    pub current_slug: ReadSignal<String>,
    /// Base URL path for the blog (e.g. "/blog").
    pub base_path: String,
    /// Callback to navigate to a blog post by slug (empty string = blog index).
    pub navigate: Callback<String>,
    /// Optional full site URL for SEO meta tags (e.g. "https://example.com").
    pub site_url: Option<String>,
}

// ============================================================================
// Docs re-exports
// ============================================================================

pub use config::{DocsConfig, ThemeConfig};
pub use registry::DocsRegistry;
pub use registry::{ApiEndpointEntry, NavConfig, NavGroup, SearchEntry};

pub use components::{
    CurrentTheme, DocsLayout, DocsPageContent, DocsPageNav, DocsSidebar, DocsVariant, DrawerOpen,
    LayoutOffsets, MobileDrawer, SearchButton, SearchModal, ThemeToggle,
};

pub use hooks::{DocsProviders, use_docs_providers};

pub use dioxus_mdx::{
    ApiOperation, ApiTag, DocContent, DocTableOfContents, EndpointPage, HttpMethod, OpenApiSpec,
    ParsedDoc, extract_headers, highlight_code,
};

#[cfg(feature = "mermaid")]
pub use dioxus_mdx::MermaidDiagram;

// ============================================================================
// Blog re-exports
// ============================================================================

pub use blog::types::{Author, BlogFrontmatter, BlogPost, BlogSearchEntry};
pub use blog::{BlogConfig, BlogProviders, BlogRegistry, use_blog_providers};

pub use components::{
    AuthorInfo, BlogCard, BlogIndexMeta, BlogLayout, BlogList, BlogMobileDrawer, BlogPostMeta,
    BlogPostNav, BlogPostView, BlogSearchButton, BlogSearchModal, BlogThemeToggle,
    ReadingProgressBar, ReadingTimeBadge, RelatedPosts, TagFilter,
};

// ============================================================================
// Macros
// ============================================================================

/// Generates a `doc_content_map()` function that returns a
/// `HashMap<&'static str, &'static str>` from the build-script output.
///
/// Place this at module level in your `main.rs`:
///
/// ```rust,ignore
/// dioxus_docs_kit::doc_content_map!();
/// ```
///
/// Requires `dioxus-docs-kit-build` in `[build-dependencies]` and a `build.rs`
/// that calls `dioxus_docs_kit_build::generate_content_map("docs/_nav.json")`.
#[macro_export]
macro_rules! doc_content_map {
    () => {
        fn doc_content_map() -> ::std::collections::HashMap<&'static str, &'static str> {
            include!(concat!(env!("OUT_DIR"), "/doc_content_generated.rs"))
        }
    };
}

/// Generates a `blog_content_map()` function that returns a
/// `HashMap<&'static str, &'static str>` from the build-script output.
///
/// Place this at module level in your `main.rs`:
///
/// ```rust,ignore
/// dioxus_docs_kit::blog_content_map!();
/// ```
///
/// Requires `dioxus-docs-kit-build` in `[build-dependencies]` and a `build.rs`
/// that calls `dioxus_docs_kit_build::generate_blog_content_map("blog/_blog.json")`.
#[macro_export]
macro_rules! blog_content_map {
    () => {
        fn blog_content_map() -> ::std::collections::HashMap<&'static str, &'static str> {
            include!(concat!(env!("OUT_DIR"), "/blog_content_generated.rs"))
        }
    };
}
