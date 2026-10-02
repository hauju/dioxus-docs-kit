//! Blog engine module for dioxus-docs-kit.
//!
//! Provides a complete blog with post listing, tag filtering, search,
//! reading time, and MDX rendering — all embedded at compile time.
//! The registry and types live in [`docs_kit_core::blog`].

pub use docs_kit_core::blog::{config, registry, types};

pub mod hooks;

pub use config::BlogConfig;
pub use hooks::{BlogProviders, use_blog_providers};
pub use registry::BlogRegistry;
pub use types::{
    Author, BlogCategory, BlogCategoryMetadata, BlogFrontmatter, BlogPost, BlogSearchEntry,
};
