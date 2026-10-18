//! The browser-side diagram loader, shared by every renderer of
//! `<pre class="mermaid">` blocks.

/// Renders Mermaid source with Pele, loading its pinned ESM API and diagram
/// types from each block's `data-pele-url` when it comes within 200px of the viewport.
///
/// Safe to run on every Dioxus route change: one observer tracks new blocks,
/// and stale asynchronous renders cannot overwrite a replacement page.
/// Colors follow CSS variables, so theme changes need no re-render.
/// Without JavaScript the server-rendered source remains readable.
pub const MERMAID_JS: &str = include_str!("mermaid.js");
