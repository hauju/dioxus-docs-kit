//! Regex engine selection.
//!
//! With `highlight` on, `arborium-tree-sitter` already links the full `regex`
//! crate, so using it here costs nothing extra. Without it, `regex-lite`
//! provides the same API for the simple patterns this crate uses at a fraction
//! of the code size.

#[cfg(feature = "highlight")]
pub(crate) use regex::{Captures, Regex};
#[cfg(not(feature = "highlight"))]
pub(crate) use regex_lite::{Captures, Regex};
