//! Blog engine: post registry, categories, search and feeds — all embedded
//! at compile time.

mod categories;
pub mod config;
pub mod registry;
pub mod types;

pub use config::BlogConfig;
pub use registry::BlogRegistry;
pub use types::{
    Author, BlogCategory, BlogCategoryMetadata, BlogFrontmatter, BlogPost, BlogSearchEntry,
};
