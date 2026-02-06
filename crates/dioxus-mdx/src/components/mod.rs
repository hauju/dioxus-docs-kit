//! Dioxus components for rendering MDX documentation.
//!
//! This module provides pre-built components for rendering parsed MDX content
//! including callouts, cards, tabs, steps, code blocks, and more.
//!
//! # Styling
//!
//! Components use Tailwind CSS with DaisyUI classes. Ensure your project has
//! Tailwind and DaisyUI configured for proper styling.
//!
//! # Example
//!
//! ```rust,ignore
//! use dioxus::prelude::*;
//! use dioxus_mdx::{parse_mdx, MdxContent};
//!
//! #[component]
//! fn DocsPage(content: String) -> Element {
//!     rsx! {
//!         MdxContent { content }
//!     }
//! }
//! ```

mod accordion;
mod api_examples;
mod callout;
mod card;
mod code;
mod icons;
pub mod openapi;
mod param_field;
mod renderer;
mod response_field;
mod steps;
mod tabs;
mod toc;
mod update;

pub use accordion::*;
pub use api_examples::*;
pub use callout::*;
pub use card::*;
pub use code::*;
pub use icons::*;
pub use openapi::*;
pub use param_field::*;
pub use renderer::*;
pub use response_field::*;
pub use steps::*;
pub use tabs::*;
pub use toc::*;
pub use update::*;
