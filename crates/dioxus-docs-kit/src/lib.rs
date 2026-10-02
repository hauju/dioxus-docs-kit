//! # dioxus-docs-kit
//!
//! Reusable documentation site shell and blog engine for Dioxus applications.
//!
//! Provides a complete docs layout with sidebar navigation, search modal,
//! page navigation, OpenAPI API reference pages, and mobile drawer.
//! Also includes a full blog engine with post listing, tag filtering,
//! search, reading time, and MDX rendering.
//!
//! Content is parsed into a JSON bundle by `dioxus-docs-kit-build` in your
//! `build.rs`; the app embeds that bundle and deserializes it once, so the wasm
//! client never links an MDX, Markdown or YAML parser.
//!
//! ## Quick Start — Docs
//!
//! ```rust,ignore
//! // build.rs
//! dioxus_docs_kit_build::DocsBuild::new("docs/_nav.json")
//!     .with_openapi("api-reference", "docs/api-reference/openapi.yaml")
//!     .generate();
//! ```
//!
//! ```rust,ignore
//! use dioxus::prelude::*;
//! use dioxus_docs_kit::{DocsConfig, DocsRegistry, DocsContext, DocsLayout, DocsPageContent};
//! use std::sync::LazyLock;
//!
//! static DOCS: LazyLock<DocsRegistry> = LazyLock::new(|| {
//!     DocsConfig::new(dioxus_docs_kit::docs_bundle!())
//!         .with_default_path("getting-started/introduction")
//!         .build()
//! });
//! ```
//!
//! ## Quick Start — Blog
//!
//! ```rust,ignore
//! // build.rs
//! dioxus_docs_kit_build::BlogBuild::new("blog/_blog.json").generate();
//! ```
//!
//! ```rust,ignore
//! use dioxus::prelude::*;
//! use dioxus_docs_kit::{BlogConfig, BlogRegistry, BlogContext, BlogLayout, BlogList, BlogPostView};
//! use std::sync::LazyLock;
//!
//! static BLOG: LazyLock<BlogRegistry> = LazyLock::new(|| {
//!     BlogConfig::new(dioxus_docs_kit::blog_bundle!())
//!         .with_posts_per_page(9)
//!         .build()
//! });
//! ```

pub mod blog;
pub mod components;
pub mod hooks;
#[cfg(feature = "server")]
pub mod server;
#[cfg(feature = "webmcp")]
pub mod webmcp;

use dioxus::prelude::*;

/// Precompiled stylesheet covering every class the kit's components emit
/// (Tailwind utilities, DaisyUI dark/light themes, typography prose, and the
/// `--dk-*` token surface from `theme.css`).
///
/// Link it to run the kit without any Tailwind/Bun setup of your own:
///
/// ```rust,ignore
/// rsx! {
///     document::Stylesheet { href: dioxus_docs_kit::DOCS_KIT_CSS }
/// }
/// ```
///
/// If your app already runs its own Tailwind build, skip this and use the
/// `safelist.html` approach from the README instead — the precompiled sheet
/// only contains the kit's classes, not yours.
pub const DOCS_KIT_CSS: Asset = asset!("/assets/docs-kit.css");

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
    /// Optional full site URL (e.g. `https://example.com`). Used as the canonical
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
    /// When true, [`DocsPageMeta`] emits a
    /// `<link rel="alternate" type="text/markdown">` pointing at the page's raw
    /// Markdown source (`<base_path>/<path>.md`), a discoverability hint for AI
    /// crawlers and "view as Markdown" tooling. Enable this only if your server
    /// actually serves those `.md` URLs (see `server::SeoRouter` behind the
    /// `server` feature). Emitted only for MDX pages (OpenAPI endpoint pages
    /// have no Markdown source), and only when [`auto_meta`](Self::auto_meta)
    /// is also on.
    pub markdown_alternate: bool,
    /// When true, [`DocsPageMeta`] emits
    /// `<link rel="alternate" type="text/plain" href="/llms.txt">` on every docs
    /// page so an agent that lands on the human-facing docs finds the
    /// machine-readable index. Enable it only if your server serves `/llms.txt`
    /// (`server::SeoRouter::with_docs`, or your own route). Emitted regardless
    /// of [`auto_meta`](Self::auto_meta): it is a site-level pointer you opted
    /// into, not per-page metadata your own `<head>` component would duplicate.
    pub llms_txt: bool,
    /// When set, [`DocsPageMeta`] emits `<link rel="help" href="...">` on every
    /// docs page — the standard HTML link type for "where to get help", e.g. a
    /// guide that tells agents and humans how to report problems with the docs
    /// or API. Emitted regardless of [`auto_meta`](Self::auto_meta), like
    /// [`llms_txt`](Self::llms_txt).
    pub help_url: Option<String>,
}

impl DocsContext {
    /// Create a context from the three required fields.
    ///
    /// The meta fields default to `site_url: None`, `auto_meta: true`,
    /// `markdown_alternate: false`, `llms_txt: false`, `help_url: None`;
    /// override them with the `with_*` setters. Prefer this over a struct
    /// literal — new fields get sensible defaults here instead of breaking your
    /// build.
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
            llms_txt: false,
            help_url: None,
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

    /// Emit `<link rel="alternate" type="text/plain" href="/llms.txt">` on every
    /// docs page (default: off).
    pub fn with_llms_txt(mut self, llms_txt: bool) -> Self {
        self.llms_txt = llms_txt;
        self
    }

    /// Emit `<link rel="help" href="...">` on every docs page, pointing at a
    /// guide on how to report problems (default: none).
    pub fn with_help_link(mut self, url: impl Into<String>) -> Self {
        self.help_url = Some(url.into());
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
    /// Current category slug, when category routes are enabled.
    pub current_category: Option<ReadSignal<String>>,
    /// Base URL path for the blog (e.g. "/blog").
    pub base_path: String,
    /// Callback to navigate to a blog post by slug (empty string = blog index).
    pub navigate: Callback<String>,
    /// Optional full site URL (e.g. `https://example.com`). Used as the canonical
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
    /// When true, [`BlogPostMeta`] emits a
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
            current_category: None,
            base_path: base_path.into(),
            navigate,
            site_url: None,
            auto_meta: true,
            markdown_alternate: false,
        }
    }

    /// Track category routes for active navigation and mobile drawer dismissal.
    pub fn with_current_category(mut self, slug: impl Into<ReadSignal<String>>) -> Self {
        self.current_category = Some(slug.into());
        self
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

/// The framework-agnostic registry, config and error modules, re-exported at
/// their pre-0.11 paths.
pub use docs_kit_core::{config, error, registry};

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

#[cfg(feature = "webmcp")]
pub use webmcp::DocsWebMcp;

pub use dioxus_mdx::{
    ApiOperation, ApiTag, CodeBlockNode, DocCodeBlock, DocContent, DocTableOfContents,
    EndpointPage, HttpMethod, OpenApiSpec, ParsedDoc, extract_headers,
};

/// The vendored Lucide icons the shell renders, re-exported so consumers can
/// use the same `Icon` in their own headers and pages.
pub use dioxus_mdx::lucide;

/// The syntax highlighter behind rendered code blocks.
///
/// Re-exported so a consumer can lex a snippet, or map a fence slug to a
/// language, without depending on `hl-lite` directly. Token colors are CSS:
/// see the `--dk-hl-*` tokens in `theme.css`.
pub use dioxus_mdx::hl;

#[cfg(feature = "mermaid")]
pub use dioxus_mdx::MermaidDiagram;

// ============================================================================
// Blog re-exports
// ============================================================================

pub use blog::hooks::{ActiveTag, CurrentPage};
pub use blog::types::{
    Author, BlogCategory, BlogCategoryMetadata, BlogFrontmatter, BlogPost, BlogSearchEntry,
};
pub use blog::{BlogConfig, BlogProviders, BlogRegistry, use_blog_providers};

pub use components::{
    AuthorInfo, BlogCard, BlogCategoryPage, BlogIndexMeta, BlogLayout, BlogList, BlogMobileDrawer,
    BlogPostMeta, BlogPostNav, BlogPostView, BlogSearchButton, BlogSearchModal, BlogThemeToggle,
    ReadingProgressBar, ReadingTimeBadge, RelatedPosts, TagFilter,
};

// ============================================================================
// Macros
// ============================================================================

pub use docs_kit_core::{blog_bundle, docs_bundle};
