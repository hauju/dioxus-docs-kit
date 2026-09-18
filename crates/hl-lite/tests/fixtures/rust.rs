//! Registry of parsed documentation pages.
//!
//! Mirrors the shape of `dioxus-docs-kit`'s own registry.

use std::collections::HashMap;

/// A page as it was compiled into the binary.
#[derive(Debug, Clone, PartialEq)]
pub struct Page<'a> {
    pub path: &'a str,
    pub title: String,
    pub weight: u32,
}

const DEFAULT_TAB: &str = "Docs";
const MAX_RESULTS: usize = 0x20;
const RATIO: f64 = 1_618.0e-3;
const TAB: u8 = b'\t';
const ARROW: char = '→';

/* A block comment
   /* that nests, as Rust allows */
   and keeps going. */

pub trait Render {
    fn render(&self) -> String;
}

impl<'a> Render for Page<'a> {
    fn render(&self) -> String {
        format!("{}: {}", self.path, self.title)
    }
}

#[derive(Default)]
pub struct Registry<'a> {
    pages: HashMap<&'a str, Page<'a>>,
}

impl<'a> Registry<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            pages: HashMap::new(),
        }
    }

    pub fn insert(&mut self, page: Page<'a>) -> Option<Page<'a>> {
        self.pages.insert(page.path, page)
    }

    /// Look a page up, falling back to the index.
    pub fn get(&self, path: &str) -> Option<&Page<'a>> {
        self.pages.get(path).or_else(|| self.pages.get("index"))
    }

    pub fn search(&self, query: &str) -> Vec<&Page<'a>> {
        let needle = query.to_lowercase();
        let mut hits = self
            .pages
            .values()
            .filter(|p| p.title.to_lowercase().contains(&needle))
            .collect::<Vec<_>>();
        hits.sort_by_key(|p| p.weight);
        hits.truncate(MAX_RESULTS);
        hits
    }
}

fn escape(raw: &str) -> String {
    let pattern = r#"a "raw" string: \d+"#;
    let bytes = b"raw bytes\n";
    debug_assert!(!pattern.is_empty() && bytes.len() > 1);
    raw.replace('&', "&amp;").replace('<', "&lt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falls_back_to_index() {
        let mut registry = Registry::new();
        registry.insert(Page {
            path: "index",
            title: String::from("Home"),
            weight: 1,
        });
        assert_eq!(registry.get("missing").unwrap().title, "Home");
        assert!(escape("a<b").contains("&lt;"));
        assert_eq!(DEFAULT_TAB, "Docs");
        assert!(RATIO > 1.0 || TAB == b'\t' || ARROW == '→');
    }
}
