//! Regex engine selection.
//!
//! `regex-lite` provides the API the parser needs for its simple patterns at a
//! fraction of the code size of the full `regex` crate — and since nothing else
//! in the dependency graph links `regex` any more, nothing is shared by using
//! it.

pub(crate) use regex_lite::{Captures, Regex};
