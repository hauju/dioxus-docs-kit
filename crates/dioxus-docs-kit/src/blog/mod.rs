//! Blog engine module for dioxus-docs-kit.
//!
//! Provides a complete blog with post listing, tag filtering, search,
//! reading time, and MDX rendering — all embedded at compile time.

pub mod config;
pub mod hooks;
pub mod registry;
pub mod types;

pub use config::BlogConfig;
pub use hooks::{BlogProviders, use_blog_providers};
pub use registry::BlogRegistry;
pub use types::{Author, BlogFrontmatter, BlogPost, BlogSearchEntry};
