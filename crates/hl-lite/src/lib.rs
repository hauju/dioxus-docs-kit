//! A tiny, dependency-free syntax highlighter.
//!
//! `hl-lite` turns a source string into a flat list of [`Span`]s, each tagged
//! with a [`Kind`] that maps to a CSS class (`hl-keyword`, `hl-string`, …).
//! It is built for documentation sites: twelve hand-written byte-level lexers,
//! no regex engine, no C, no dependencies, and a few tens of kilobytes of
//! WebAssembly instead of the megabytes a tree-sitter grammar set costs.
//!
//! The lexers are *lexical*, not scope-aware — the same level of fidelity
//! [highlight.js](https://highlightjs.org) or Prism give you, not a parse
//! tree. See the crate README for the per-language table of known gaps.
//!
//! # Example
//!
//! ```
//! use hl_lite::{highlight, Kind, Lang};
//!
//! let lang = Lang::from_slug("rs").unwrap();
//! let spans = highlight(lang, "let x = 1;");
//!
//! assert_eq!(spans[0].kind, Kind::Keyword);
//! assert_eq!(spans[0].text, "let");
//! assert_eq!(spans[0].kind.class(), "keyword");
//!
//! // The spans always reconstruct the input exactly.
//! let joined: String = spans.iter().map(|s| s.text).collect();
//! assert_eq!(joined, "let x = 1;");
//! ```
//!
//! # Guarantees
//!
//! * Concatenating every [`Span::text`] reproduces the input byte for byte.
//! * Spans never split a `char`, so non-ASCII input cannot panic.
//! * Unterminated strings, comments, heredocs and templates run to the end of
//!   the input instead of panicking or looping.
//! * Every lexer is a single forward pass: highlighting is O(input length).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod bash;
mod css;
mod dockerfile;
mod emit;
mod html;
mod js;
mod json;
mod markdown;
mod python;
mod rust;
mod toml;
mod util;
mod yaml;

/// A language `hl-lite` can highlight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    /// Rust.
    Rust,
    /// Bash / sh / shell sessions.
    Bash,
    /// CSS.
    Css,
    /// Dockerfile.
    Dockerfile,
    /// HTML (also used for XML and SVG).
    Html,
    /// JavaScript.
    JavaScript,
    /// JSON (tolerates JSONC comments).
    Json,
    /// Markdown.
    Markdown,
    /// Python.
    Python,
    /// TOML.
    Toml,
    /// TypeScript.
    TypeScript,
    /// YAML.
    Yaml,
}

/// Fence slug (and alias) to language. Matched case-insensitively, in order.
const SLUGS: &[(&str, Lang)] = &[
    ("rust", Lang::Rust),
    ("rs", Lang::Rust),
    ("bash", Lang::Bash),
    ("sh", Lang::Bash),
    ("shell", Lang::Bash),
    ("zsh", Lang::Bash),
    ("console", Lang::Bash),
    ("terminal", Lang::Bash),
    ("css", Lang::Css),
    ("dockerfile", Lang::Dockerfile),
    ("docker", Lang::Dockerfile),
    ("containerfile", Lang::Dockerfile),
    ("html", Lang::Html),
    ("htm", Lang::Html),
    ("xml", Lang::Html),
    ("svg", Lang::Html),
    ("javascript", Lang::JavaScript),
    ("js", Lang::JavaScript),
    ("jsx", Lang::JavaScript),
    ("mjs", Lang::JavaScript),
    ("cjs", Lang::JavaScript),
    ("json", Lang::Json),
    ("jsonc", Lang::Json),
    ("json5", Lang::Json),
    ("markdown", Lang::Markdown),
    ("md", Lang::Markdown),
    ("mdx", Lang::Markdown),
    ("python", Lang::Python),
    ("py", Lang::Python),
    ("py3", Lang::Python),
    ("toml", Lang::Toml),
    ("typescript", Lang::TypeScript),
    ("ts", Lang::TypeScript),
    ("tsx", Lang::TypeScript),
    ("yaml", Lang::Yaml),
    ("yml", Lang::Yaml),
];

impl Lang {
    /// Resolve a fence slug or common alias, case-insensitively.
    ///
    /// Returns `None` for anything unknown, so callers can fall back to
    /// rendering the block as plain text.
    ///
    /// ```
    /// # use hl_lite::Lang;
    /// assert_eq!(Lang::from_slug("TSX"), Some(Lang::TypeScript));
    /// assert_eq!(Lang::from_slug("brainfuck"), None);
    /// ```
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Lang> {
        SLUGS
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(slug))
            .map(|&(_, lang)| lang)
    }

    /// Guess the language of a file name or path.
    ///
    /// Uses the extension, falling back to the base name for the files that
    /// carry their language there rather than in a suffix.
    ///
    /// ```
    /// # use hl_lite::Lang;
    /// assert_eq!(Lang::from_path("src/main.rs"), Some(Lang::Rust));
    /// assert_eq!(Lang::from_path("Dockerfile"), Some(Lang::Dockerfile));
    /// assert_eq!(Lang::from_path("LICENSE"), None);
    /// ```
    #[must_use]
    pub fn from_path(path: &str) -> Option<Lang> {
        let name = path
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(path)
            .trim_end_matches(['.', ' ']);

        if let Some((_, ext)) = name.rsplit_once('.')
            && let Some(lang) = Lang::from_slug(ext)
        {
            return Some(lang);
        }
        // `Dockerfile`, `Containerfile`, and variants like `Dockerfile.ci`.
        let stem = name.split('.').next().unwrap_or(name);
        (stem.eq_ignore_ascii_case("dockerfile") || stem.eq_ignore_ascii_case("containerfile"))
            .then_some(Lang::Dockerfile)
    }

    /// The canonical slug for this language.
    #[must_use]
    pub fn slug(self) -> &'static str {
        match self {
            Lang::Rust => "rust",
            Lang::Bash => "bash",
            Lang::Css => "css",
            Lang::Dockerfile => "dockerfile",
            Lang::Html => "html",
            Lang::JavaScript => "javascript",
            Lang::Json => "json",
            Lang::Markdown => "markdown",
            Lang::Python => "python",
            Lang::Toml => "toml",
            Lang::TypeScript => "typescript",
            Lang::Yaml => "yaml",
        }
    }
}

/// What a [`Span`] of source is.
///
/// The variants are deliberately generic: they are render targets, not a
/// grammar. Each language maps its own constructs onto the closest one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// Anything unclassified: whitespace, plain identifiers, prose.
    Plain,
    /// Language keyword (`fn`, `if`, `@media`, `FROM`).
    Keyword,
    /// String, character and regex literal; also fenced code in Markdown.
    String,
    /// Comment.
    Comment,
    /// Numeric literal (and TOML/YAML dates).
    Number,
    /// Type name.
    Type,
    /// Function or macro name at a call or definition site.
    Function,
    /// Attribute / annotation / decorator (`#[derive]`, `@app.route`, HTML
    /// attribute names, shell flags).
    Attribute,
    /// Key of a key/value pair (CSS property, JSON/TOML/YAML key).
    Property,
    /// Markup tag name, TOML table header, CSS selector.
    Tag,
    /// Operator.
    Operator,
    /// Bracket, comma, separator, list marker.
    Punctuation,
    /// Literal constant (`true`, `null`, `#fff`, Rust lifetimes).
    Constant,
    /// Variable reference (`$PATH`, `var(--x)`, YAML anchors).
    Variable,
}

impl Kind {
    /// The CSS class suffix for this kind, e.g. `Keyword` → `"keyword"`.
    ///
    /// Renderers use it as `hl-{class}`.
    #[must_use]
    pub fn class(self) -> &'static str {
        match self {
            Kind::Plain => "plain",
            Kind::Keyword => "keyword",
            Kind::String => "string",
            Kind::Comment => "comment",
            Kind::Number => "number",
            Kind::Type => "type",
            Kind::Function => "function",
            Kind::Attribute => "attribute",
            Kind::Property => "property",
            Kind::Tag => "tag",
            Kind::Operator => "operator",
            Kind::Punctuation => "punctuation",
            Kind::Constant => "constant",
            Kind::Variable => "variable",
        }
    }
}

/// A classified slice of the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span<'a> {
    /// What the slice is.
    pub kind: Kind,
    /// The slice itself, borrowed from the input.
    pub text: &'a str,
}

/// Highlight `src` as `lang`.
///
/// Adjacent spans of the same kind are merged, and the concatenation of every
/// [`Span::text`] is exactly `src`.
#[must_use]
pub fn highlight(lang: Lang, src: &str) -> Vec<Span<'_>> {
    let mut out = emit::Emit::new(src);
    match lang {
        Lang::Rust => rust::lex(&mut out),
        Lang::Bash => bash::lex(&mut out),
        Lang::Css => css::lex(&mut out),
        Lang::Dockerfile => dockerfile::lex(&mut out),
        Lang::Html => html::lex(&mut out),
        Lang::JavaScript => js::lex(&mut out, false),
        Lang::TypeScript => js::lex(&mut out, true),
        Lang::Json => json::lex(&mut out),
        Lang::Markdown => markdown::lex(&mut out),
        Lang::Python => python::lex(&mut out),
        Lang::Toml => toml::lex(&mut out),
        Lang::Yaml => yaml::lex(&mut out),
    }
    out.finish()
}
