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

// The tree-sitter C grammars pulled in via `dioxus-code` (for syntax
// highlighting) reference the libc global `stderr`. Its definition lives in
// `arborium-sysroot`'s C shim, but that shim links as a plain static archive
// whose `stdio.o` member only gets pulled in when `stderr` is undefined as the
// linker reaches it — and on some host toolchains (notably Homebrew LLVM on
// macOS) it isn't, so the wasm link fails with `undefined symbol: stderr`.
//
// We define `stderr` here in the *library* so every consumer that links
// `dioxus-docs-kit` gets the symbol — defining it only in the docs-kit binary
// (`src/main.rs`) leaves library consumers (their own app binaries) to hit the
// same link error. A strong definition preempts the sysroot's lazy archive
// member, so where the shim already links cleanly its `stdio.o` simply isn't
// pulled and there is never a duplicate symbol. `fprintf` is a no-op macro in
// the shim's headers, so `stderr` is referenced but never dereferenced at
// runtime. `#[used]` keeps the symbol from being dropped from the rlib before
// it can satisfy the cross-crate reference at final link.
//
// This is only needed when the `highlight` feature is enabled, since that is what
// links the tree-sitter C code (via `dioxus-code`) that references `stderr`.
#[cfg(all(target_arch = "wasm32", feature = "highlight"))]
mod wasm_sysroot_stderr {
    use core::ffi::c_void;
    static mut DUMMY_FILE: u8 = 0;
    #[used]
    #[unsafe(no_mangle)]
    static mut stderr: *mut c_void = &raw mut DUMMY_FILE as *mut c_void;
}

pub mod blog;
pub mod components;
pub mod config;
pub mod error;
pub mod hooks;
pub mod registry;
pub(crate) mod search;
#[cfg(feature = "server")]
pub mod server;

use dioxus::prelude::*;

// ============================================================================
// Docs context
// ============================================================================

/// Navigation bridge that decouples library components from the consumer's Route enum.
///
/// The consumer creates this in their docs layout wrapper and provides it via `use_context_provider`.
#[derive(Clone)]
#[non_exhaustive]
pub struct DocsContext {
    /// Current docs page path (e.g. "getting-started/introduction").
    pub current_path: ReadSignal<String>,
    /// Base URL path for docs (e.g. "/docs").
    pub base_path: String,
    /// Callback to navigate to a docs page by content path.
    pub navigate: Callback<String>,
    /// Optional full site URL (e.g. "https://example.com"). Used as the canonical
    /// host for emitted `<link rel="canonical">` and `og:url` tags. Independent
    /// of [`auto_meta`](Self::auto_meta) — set it whenever you want kit helpers
    /// (sitemap generation, canonical URLs) to know the public origin, even if
    /// you suppress automatic meta emission.
    pub site_url: Option<String>,
    /// When true, the kit emits per-page `<title>`, `<meta name="description">`,
    /// Open Graph and Twitter Card tags from frontmatter. Set to `false` if your
    /// app manages its own `<head>` (e.g. brand-specific OG images, structured
    /// data) and the kit's emissions would conflict. Title and description tags
    /// always emit when this is on; canonical and `og:url` only emit when
    /// [`site_url`](Self::site_url) is also set.
    pub auto_meta: bool,
    /// When true, [`DocsPageMeta`](crate::DocsPageMeta) emits a
    /// `<link rel="alternate" type="text/markdown">` pointing at the page's raw
    /// Markdown source (`<base_path>/<path>.md`), a discoverability hint for AI
    /// crawlers and "view as Markdown" tooling. Enable this only if your server
    /// actually serves those `.md` URLs (see `server::SeoRouter` behind the
    /// `server` feature). Emitted only for MDX pages (OpenAPI endpoint pages
    /// have no Markdown source), and only when [`auto_meta`](Self::auto_meta)
    /// is also on.
    pub markdown_alternate: bool,
}

impl DocsContext {
    /// Create a context from the three required fields.
    ///
    /// The meta fields default to `site_url: None`, `auto_meta: true`,
    /// `markdown_alternate: false`; override them with the `with_*` setters.
    /// Prefer this over a struct literal — new fields get sensible defaults
    /// here instead of breaking your build.
    pub fn new(
        current_path: impl Into<ReadSignal<String>>,
        base_path: impl Into<String>,
        navigate: Callback<String>,
    ) -> Self {
        Self {
            current_path: current_path.into(),
            base_path: base_path.into(),
            navigate,
            site_url: None,
            auto_meta: true,
            markdown_alternate: false,
        }
    }

    /// Set the public site origin (e.g. `"https://example.com"`).
    pub fn with_site_url(mut self, site_url: impl Into<String>) -> Self {
        self.site_url = Some(site_url.into());
        self
    }

    /// Enable or disable automatic per-page meta emission (default: on).
    pub fn with_auto_meta(mut self, auto_meta: bool) -> Self {
        self.auto_meta = auto_meta;
        self
    }

    /// Emit `<link rel="alternate" type="text/markdown">` tags (default: off).
    pub fn with_markdown_alternate(mut self, markdown_alternate: bool) -> Self {
        self.markdown_alternate = markdown_alternate;
        self
    }
}

// ============================================================================
// Blog context
// ============================================================================

/// Navigation bridge for blog pages, decoupled from the consumer's Route enum.
///
/// The consumer creates this in their blog layout wrapper and provides it via `use_context_provider`.
#[derive(Clone)]
#[non_exhaustive]
pub struct BlogContext {
    /// Current blog post slug (empty on the list/index page).
    pub current_slug: ReadSignal<String>,
    /// Base URL path for the blog (e.g. "/blog").
    pub base_path: String,
    /// Callback to navigate to a blog post by slug (empty string = blog index).
    pub navigate: Callback<String>,
    /// Optional full site URL (e.g. "https://example.com"). Used as the canonical
    /// host for emitted `<link rel="canonical">`, `og:url`, and JSON-LD URLs.
    /// Independent of [`auto_meta`](Self::auto_meta) — set it whenever you want
    /// kit helpers (sitemap/RSS, canonical URLs) to know the public origin, even
    /// if you suppress automatic meta emission.
    pub site_url: Option<String>,
    /// When true, the kit emits per-page `<title>`, `<meta name="description">`,
    /// Open Graph, Twitter Card, and Article JSON-LD tags from frontmatter. Set
    /// to `false` if your app manages its own `<head>` (e.g. brand-specific OG
    /// images, structured data) and the kit's emissions would conflict. Title
    /// and description tags always emit when this is on; canonical, `og:url`,
    /// and JSON-LD `@id` only emit when [`site_url`](Self::site_url) is also set.
    pub auto_meta: bool,
    /// When true, [`BlogPostMeta`](crate::BlogPostMeta) emits a
    /// `<link rel="alternate" type="text/markdown">` pointing at the post's raw
    /// Markdown source (`<base_path>/<slug>.md`), a discoverability hint for AI
    /// crawlers and "view as Markdown" tooling. Enable this only if your server
    /// actually serves those `.md` URLs (see `server::SeoRouter` behind the
    /// `server` feature). Emitted only when [`auto_meta`](Self::auto_meta) is
    /// also on.
    pub markdown_alternate: bool,
}

impl BlogContext {
    /// Create a context from the three required fields.
    ///
    /// The meta fields default to `site_url: None`, `auto_meta: true`,
    /// `markdown_alternate: false`; override them with the `with_*` setters.
    /// Prefer this over a struct literal — new fields get sensible defaults
    /// here instead of breaking your build.
    pub fn new(
        current_slug: impl Into<ReadSignal<String>>,
        base_path: impl Into<String>,
        navigate: Callback<String>,
    ) -> Self {
        Self {
            current_slug: current_slug.into(),
            base_path: base_path.into(),
            navigate,
            site_url: None,
            auto_meta: true,
            markdown_alternate: false,
        }
    }

    /// Set the public site origin (e.g. `"https://example.com"`).
    pub fn with_site_url(mut self, site_url: impl Into<String>) -> Self {
        self.site_url = Some(site_url.into());
        self
    }

    /// Enable or disable automatic per-page meta emission (default: on).
    pub fn with_auto_meta(mut self, auto_meta: bool) -> Self {
        self.auto_meta = auto_meta;
        self
    }

    /// Emit `<link rel="alternate" type="text/markdown">` tags (default: off).
    pub fn with_markdown_alternate(mut self, markdown_alternate: bool) -> Self {
        self.markdown_alternate = markdown_alternate;
        self
    }
}

// ============================================================================
// Docs re-exports
// ============================================================================

#[cfg(feature = "highlight")]
pub use config::CodeThemeConfig;
pub use config::{DocsConfig, ThemeConfig};
pub use error::DocsKitError;
pub use registry::DocsRegistry;
pub use registry::{ApiEndpointEntry, NavConfig, NavGroup, SearchEntry};

pub use components::{
    ActiveTab, CopyPageButton, CurrentTheme, DocsLayout, DocsPageContent, DocsPageMeta,
    DocsPageNav, DocsSidebar, DocsVariant, DrawerOpen, LayoutOffsets, MobileDrawer, SearchButton,
    SearchModal, SearchOpen, ThemeToggle, use_theme_provider,
};

pub use hooks::{DocsProviders, use_docs_context, use_docs_providers};

pub use dioxus_mdx::{
    ApiOperation, ApiTag, DocContent, DocTableOfContents, EndpointPage, HttpMethod, OpenApiSpec,
    ParsedDoc, extract_headers,
};

#[cfg(feature = "highlight")]
pub use dioxus_mdx::CodeThemeOverride;

#[cfg(feature = "highlight")]
pub use dioxus_code::{Code, CodeTheme, Language, SourceCode, Theme};

#[cfg(feature = "mermaid")]
pub use dioxus_mdx::MermaidDiagram;

// ============================================================================
// Blog re-exports
// ============================================================================

pub use blog::hooks::{ActiveTag, CurrentPage};
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
