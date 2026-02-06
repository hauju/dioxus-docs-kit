//! # dioxus-docs-kit
//!
//! Reusable documentation site shell for Dioxus applications.
//!
//! Provides a complete docs layout with sidebar navigation, search modal,
//! page navigation, OpenAPI API reference pages, and mobile drawer.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use dioxus::prelude::*;
//! use dioxus_docs_kit::{DocsConfig, DocsRegistry, DocsContext, DocsLayout, DocsPageContent};
//! use std::sync::LazyLock;
//!
//! static DOCS: LazyLock<DocsRegistry> = LazyLock::new(|| {
//!     DocsConfig::new(include_str!("../docs/_nav.json"), doc_content_map())
//!         .with_openapi("api-reference", include_str!("../docs/api-reference/spec.yaml"))
//!         .build()
//! });
//!
//! #[component]
//! fn MyDocsLayout() -> Element {
//!     let nav = use_navigator();
//!     let route = use_route::<Route>();
//!     let current_path = use_memo(move || /* extract path from route */);
//!     let docs_ctx = DocsContext {
//!         current_path: current_path.into(),
//!         base_path: "/docs".into(),
//!         navigate: Callback::new(move |path: String| { /* push route */ }),
//!     };
//!     use_context_provider(|| &*DOCS as &'static DocsRegistry);
//!     use_context_provider(|| docs_ctx);
//!     rsx! { DocsLayout {} }
//! }
//! ```

pub mod components;
pub mod config;
pub mod registry;

use dioxus::prelude::*;

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

// Re-export config and registry
pub use config::{DocsConfig, ThemeConfig};
pub use registry::DocsRegistry;

// Re-export types consumers need
pub use registry::{ApiEndpointEntry, NavConfig, NavGroup, SearchEntry};

// Re-export UI components
pub use components::{
    DocsLayout, DocsPageContent, DocsPageNav, DocsSidebar, DrawerOpen, LayoutOffsets,
    MobileDrawer, SearchButton, SearchModal, ThemeToggle,
};

// Re-export key dioxus-mdx types that consumers commonly need
pub use dioxus_mdx::{
    ApiOperation, ApiTag, DocContent, DocTableOfContents, EndpointPage, HttpMethod, OpenApiSpec,
    ParsedDoc, extract_headers, highlight_code,
};
