//! Build-time content pipeline for [`dioxus-docs-kit`].
//!
//! Call [`DocsBuild`] (and [`BlogBuild`]) from your `build.rs`. Each one reads
//! its manifest, parses every `.mdx` page and OpenAPI spec, renders the prose to
//! HTML, precomputes the search index, and writes a single JSON bundle into
//! `OUT_DIR`. The app embeds that bundle with `dioxus_docs_kit::docs_bundle!()`,
//! so the wasm client never links the MDX parser, markdown-rs, `openapiv3` or
//! `serde_yaml`, and never re-parses a page on navigation.
//!
//! ```rust,ignore
//! fn main() {
//!     dioxus_docs_kit_build::DocsBuild::new("docs/_nav.json")
//!         .with_openapi("api-reference", "docs/api-reference/petstore.yaml")
//!         .with_validation(dioxus_docs_kit_build::ValidationMode::Strict)
//!         .generate();
//!
//!     dioxus_docs_kit_build::BlogBuild::new("blog/_blog.json").generate();
//! }
//! ```
//!
//! [`dioxus-docs-kit`]: https://docs.rs/dioxus-docs-kit

mod bundle;

pub use bundle::{blog_bundle_json, docs_bundle_json};

use bundle::{BUNDLE_VERSION, BlogBundle, BlogFrontmatter, BlogPost};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::Path;

/// How content problems are handled by the build helper.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ValidationMode {
    /// Report problems as Cargo warnings (the backwards-compatible default).
    #[default]
    Warn,
    /// Report all problems, then fail the build instead of shipping broken pages.
    Strict,
}

fn report_diagnostics(diagnostics: &[String], mode: ValidationMode) {
    for message in diagnostics {
        println!("cargo:warning={message}");
    }
    assert!(
        mode != ValidationMode::Strict || diagnostics.is_empty(),
        "dioxus-docs-kit: strict content validation failed with {} problem(s):\n{}",
        diagnostics.len(),
        diagnostics.join("\n")
    );
}

#[derive(Deserialize)]
struct NavConfig {
    groups: Vec<NavGroup>,
}

#[derive(Deserialize)]
struct NavGroup {
    pages: Vec<String>,
}

/// Builds the absolute path used inside the generated `include_str!()`.
///
/// Backslashes are normalized to forward slashes so the generated string
/// literal is valid on Windows (`C:\Users\...` would otherwise contain
/// invalid escape sequences).
fn include_path(manifest_dir: &str, relative: &str) -> String {
    format!("{manifest_dir}/{relative}").replace('\\', "/")
}

/// Read a content file, registering it with cargo so the build re-runs when it
/// changes (and when a missing file is later created).
///
/// `None` means the file does not exist; the caller turns that into a
/// diagnostic naming the manifest entry that pointed at it.
fn read_content(manifest_dir: &str, relative: &str) -> Option<String> {
    println!("cargo:rerun-if-changed={relative}");
    fs::read_to_string(include_path(manifest_dir, relative)).ok()
}

fn missing_file_diagnostic(manifest_dir: &str, key: &str, relative: &str) -> String {
    let full_path = include_path(manifest_dir, relative);
    format!(
        "\"{key}\" is listed in the nav/manifest but {full_path} does not exist — the page will 404. Create the file or remove the entry."
    )
}

fn write_bundle(file_name: &str, json: String) {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join(file_name);
    fs::write(&dest, json).expect("Failed to write generated bundle");
}

// ============================================================================
// Docs bundle generation
// ============================================================================

/// Builds the documentation bundle a `dioxus-docs-kit` app embeds.
///
/// Reads `_nav.json`, parses every listed `.mdx` page (prose rendered to HTML)
/// and every registered OpenAPI spec, precomputes the search index, and writes
/// `$OUT_DIR/docs_bundle.json`.
///
/// ```rust,ignore
/// dioxus_docs_kit_build::DocsBuild::new("docs/_nav.json")
///     .with_openapi("api-reference", "docs/api-reference/petstore.yaml")
///     .generate();
/// ```
pub struct DocsBuild {
    nav_json_path: String,
    openapi: Vec<(String, String)>,
    mode: ValidationMode,
}

impl DocsBuild {
    /// Start a docs bundle from a `_nav.json` path, relative to the crate root.
    ///
    /// The docs directory is inferred from the parent of `nav_json_path`
    /// (e.g. `"docs/_nav.json"` → `"docs"`).
    pub fn new(nav_json_path: impl Into<String>) -> Self {
        Self {
            nav_json_path: nav_json_path.into(),
            openapi: Vec::new(),
            mode: ValidationMode::Warn,
        }
    }

    /// Register an OpenAPI spec (YAML or JSON) served under `prefix`.
    ///
    /// The spec is parsed here, so `openapiv3` and `serde_yaml` stay in the
    /// build graph. `prefix` must match the nav group named by
    /// `DocsConfig::with_api_group_name` (default `"API Reference"`); do **not**
    /// list individual operation paths in `_nav.json`.
    pub fn with_openapi(mut self, prefix: impl Into<String>, spec_path: impl Into<String>) -> Self {
        self.openapi.push((prefix.into(), spec_path.into()));
        self
    }

    /// Choose how content problems are reported (default [`ValidationMode::Warn`]).
    ///
    /// [`ValidationMode::Strict`] fails on missing files, duplicate nav entries,
    /// unsupported frontmatter, and broken links detected by the link validator.
    /// External/app routes and inferred dynamic OpenAPI routes are skipped; this
    /// does not validate runtime registry configuration.
    pub fn with_validation(mut self, mode: ValidationMode) -> Self {
        self.mode = mode;
        self
    }

    /// Parse everything and write `$OUT_DIR/docs_bundle.json`.
    ///
    /// # Panics
    ///
    /// Panics if `_nav.json` or an OpenAPI spec cannot be read or parsed — a
    /// build script has no better way to report that.
    pub fn generate(self) {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let nav_json_path = self.nav_json_path.as_str();
        let mut diagnostics = Vec::new();

        println!("cargo:rerun-if-changed={nav_json_path}");

        let nav_json = fs::read_to_string(nav_json_path)
            .unwrap_or_else(|e| panic!("Failed to read {nav_json_path}: {e}"));
        let nav: NavConfig = serde_json::from_str(&nav_json)
            .unwrap_or_else(|e| panic!("Failed to parse {nav_json_path}: {e}"));

        // Infer docs directory from nav path parent (e.g. "docs/_nav.json" → "docs")
        let docs_dir = Path::new(nav_json_path)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("docs");

        let mut sources: Vec<(String, String)> = Vec::new();
        let mut seen = HashSet::new();
        for group in &nav.groups {
            for page in &group.pages {
                if !seen.insert(page) {
                    diagnostics.push(format!("{nav_json_path}: duplicate page \"{page}\""));
                    continue;
                }
                let mdx_path = format!("{docs_dir}/{page}.mdx");
                match read_content(&manifest_dir, &mdx_path) {
                    Some(content) => sources.push((page.clone(), content)),
                    None => {
                        diagnostics.push(missing_file_diagnostic(&manifest_dir, page, &mdx_path));
                    }
                }
            }
        }

        let specs: Vec<(String, String)> = self
            .openapi
            .iter()
            .map(|(prefix, spec_path)| {
                println!("cargo:rerun-if-changed={spec_path}");
                let raw = fs::read_to_string(include_path(&manifest_dir, spec_path))
                    .unwrap_or_else(|e| panic!("Failed to read {spec_path}: {e}"));
                (prefix.clone(), raw)
            })
            .collect();

        // Gather all diagnostics before failing so authors can fix them together.
        let pages: Vec<String> = nav
            .groups
            .iter()
            .flat_map(|g| g.pages.iter().cloned())
            .collect();
        diagnostics.extend(validate_docs(&manifest_dir, docs_dir, &pages));
        report_diagnostics(&diagnostics, self.mode);

        let json = docs_bundle_json(&nav_json, &as_pairs(&sources), &as_pairs(&specs))
            .unwrap_or_else(|e| panic!("dioxus-docs-kit-build: {e}"));
        write_bundle("docs_bundle.json", json);
    }
}

fn as_pairs(owned: &[(String, String)]) -> Vec<(&str, &str)> {
    owned
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect()
}

// ============================================================================
// Blog bundle generation
// ============================================================================

#[derive(Deserialize)]
struct BlogManifest {
    posts: Vec<String>,
    #[serde(default)]
    authors: HashMap<String, serde_json::Value>,
    #[serde(default)]
    categories: HashMap<String, serde_json::Value>,
}

/// Builds the blog bundle a `dioxus-docs-kit` app embeds.
///
/// Reads `_blog.json`, parses every listed post (frontmatter, prose rendered to
/// HTML, reading time), drops drafts, sorts newest first, precomputes the search
/// index, and writes `$OUT_DIR/blog_bundle.json`.
pub struct BlogBuild {
    manifest_path: String,
    mode: ValidationMode,
}

impl BlogBuild {
    /// Start a blog bundle from a `_blog.json` path, relative to the crate root.
    ///
    /// The blog directory is inferred from the parent of `manifest_path`
    /// (e.g. `"blog/_blog.json"` → `"blog"`).
    pub fn new(manifest_path: impl Into<String>) -> Self {
        Self {
            manifest_path: manifest_path.into(),
            mode: ValidationMode::Warn,
        }
    }

    /// Choose how content problems are reported (default [`ValidationMode::Warn`]).
    ///
    /// Invalid blog frontmatter is a build error in both modes: the post would
    /// otherwise silently vanish from the site. Blog links are not checked.
    pub fn with_validation(mut self, mode: ValidationMode) -> Self {
        self.mode = mode;
        self
    }

    /// Parse everything and write `$OUT_DIR/blog_bundle.json`.
    ///
    /// # Panics
    ///
    /// Panics if `_blog.json` cannot be read or parsed, or if any listed post
    /// has malformed frontmatter.
    pub fn generate(self) {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let manifest_path = self.manifest_path.as_str();
        let mut diagnostics = Vec::new();

        println!("cargo:rerun-if-changed={manifest_path}");

        let json = fs::read_to_string(manifest_path)
            .unwrap_or_else(|e| panic!("Failed to read {manifest_path}: {e}"));
        let manifest: BlogManifest = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("Failed to parse {manifest_path}: {e}"));

        // Infer blog directory from manifest path parent
        let blog_dir = Path::new(manifest_path)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("blog");

        let mut posts: Vec<BlogPost> = Vec::new();
        let mut seen = HashSet::new();
        let mut malformed = Vec::new();
        let mut tags = HashSet::new();
        for slug in &manifest.posts {
            if !seen.insert(slug) {
                diagnostics.push(format!("{manifest_path}: duplicate post \"{slug}\""));
                continue;
            }
            let mdx_path = format!("{blog_dir}/{slug}.mdx");
            let Some(content) = read_content(&manifest_dir, &mdx_path) else {
                diagnostics.push(missing_file_diagnostic(&manifest_dir, slug, &mdx_path));
                continue;
            };

            // Malformed frontmatter fails the build instead of letting the post
            // silently vanish from the site at runtime — after every other
            // diagnostic has been reported.
            let (frontmatter, body) = match parse_blog_frontmatter(&mdx_path, &content) {
                Ok(parsed) => parsed,
                Err(message) => {
                    malformed.push(message);
                    continue;
                }
            };
            tags.extend(frontmatter.tags.iter().cloned());
            if frontmatter.draft {
                continue;
            }
            posts.push(bundle::build_post(slug, frontmatter, body));
        }

        bundle::sort_posts(&mut posts);

        for key in unknown_category_keys(&manifest.categories, &tags) {
            diagnostics.push(format!(
                "{manifest_path}: category {key:?} matches no post tag (tags are compared exactly), so its title, description and slug are never used"
            ));
        }
        diagnostics.extend(malformed.iter().cloned());
        report_diagnostics(&diagnostics, self.mode);
        assert!(
            malformed.is_empty(),
            "dioxus-docs-kit: malformed blog frontmatter:\n{}",
            malformed.join("\n")
        );

        let search = bundle::build_blog_search_index(&posts);
        let bundle = BlogBundle {
            version: BUNDLE_VERSION,
            authors: serde_json::to_value(&manifest.authors).expect("authors are JSON"),
            categories: serde_json::to_value(&manifest.categories).expect("categories are JSON"),
            posts,
            search,
        };
        write_bundle(
            "blog_bundle.json",
            serde_json::to_string(&bundle).expect("serialize blog bundle"),
        );
    }
}

// ============================================================================
// Build-time validation: internal links + frontmatter
// ============================================================================

// Anchor ids come straight from `dioxus_mdx` (a normal dependency of this
// crate now that its parser compiles without `dioxus`), so build-time anchor
// checks resolve to exactly the ids the renderer emits.
use dioxus_mdx::slugify;

/// Remove fenced code blocks (``` or ~~~) so markdown-looking text inside code
/// samples is not mistaken for links or headings.
fn strip_code_fences(content: &str) -> String {
    let mut out = String::new();
    let mut fence: Option<char> = None;
    for line in content.lines() {
        let trimmed = line.trim_start();
        let marker = if trimmed.starts_with("```") {
            Some('`')
        } else if trimmed.starts_with("~~~") {
            Some('~')
        } else {
            None
        };
        match (fence, marker) {
            (None, Some(m)) => fence = Some(m), // opening fence
            (Some(open), Some(m)) if open == m => fence = None, // closing fence
            (None, None) => {
                out.push_str(line);
                out.push('\n');
            }
            _ => {} // inside a fence (or a mismatched fence marker within one)
        }
    }
    out
}

/// Extract anchor slugs for level 2-4 ATX headings, matching the ids the
/// renderer injects. H1 is excluded (it is not linkable and is stripped as the
/// duplicate page title).
fn extract_heading_slugs(content: &str) -> Vec<String> {
    dioxus_mdx::extract_headers(content)
        .into_iter()
        .map(|(id, ..)| id)
        .collect()
}

/// Extract non-image markdown link targets (`[text](target)`), stripping any
/// `"title"` suffix and `<>` wrappers. Image links (`![...](...)`) are skipped.
fn extract_links(content: &str) -> Vec<String> {
    let bytes = content.as_bytes();
    let mut links = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            let is_image = i > 0 && bytes[i - 1] == b'!';
            if let Some(close) = (i + 1..bytes.len()).find(|&j| bytes[j] == b']') {
                if bytes.get(close + 1) == Some(&b'(')
                    && let Some(pclose) = (close + 2..bytes.len()).find(|&j| bytes[j] == b')')
                {
                    if !is_image
                        && let Some(tok) = content[close + 2..pclose].split_whitespace().next()
                    {
                        let tok = tok.trim_start_matches('<').trim_end_matches('>');
                        if !tok.is_empty() {
                            links.push(tok.to_string());
                        }
                    }
                    i = pclose + 1;
                    continue;
                }
                i = close + 1;
                continue;
            }
        }
        i += 1;
    }
    links
}

/// Returns true if `target` begins with a URL scheme (`https:`, `mailto:`, …).
fn has_scheme(target: &str) -> bool {
    let mut chars = target.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    for c in chars {
        if c == ':' {
            return true;
        }
        if !(c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.') {
            return false;
        }
    }
    false
}

/// Strip a trailing slash and any `.mdx`/`.md` extension so a link target lines
/// up with the extension-less nav page keys.
fn normalize_page_key(s: &str) -> String {
    let s = s.trim_end_matches('/');
    let s = s
        .strip_suffix(".mdx")
        .or_else(|| s.strip_suffix(".md"))
        .unwrap_or(s);
    s.to_string()
}

/// Resolve a relative link target against the directory of `current_page`.
/// Returns `None` if the path escapes the docs root.
fn resolve_relative(current_page: &str, path: &str) -> Option<String> {
    let mut base: Vec<&str> = current_page.split('/').collect();
    base.pop(); // drop the current file component, keeping its directory
    for seg in path.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                base.pop()?;
            }
            s => base.push(s),
        }
    }
    Some(base.join("/"))
}

/// Outcome of resolving an internal link target to a docs page.
enum LinkResolution {
    /// Resolved to a known page (carries its key for anchor validation).
    Valid(String),
    /// Clearly targets a docs page, but no page matches.
    Broken,
    /// Not validatable (external route, runtime-generated section, …).
    Skip,
}

/// A group directory is "validatable" only if it holds at least two static nav
/// pages. Single-page groups (e.g. an `api-reference` group whose remaining
/// pages are generated at runtime from an OpenAPI spec) are skipped, since the
/// build crate cannot know those slugs and would false-positive on them.
fn group_is_validatable(page: &str, group_counts: &HashMap<&str, usize>) -> bool {
    page.split_once('/')
        .map(|(g, _)| group_counts.get(g).copied().unwrap_or(0) >= 2)
        .unwrap_or(false)
}

/// Classify a root-absolute link (e.g. `/docs/guides/foo`). The consumer's base
/// path (e.g. `/docs`) is unknown, so both the full path and the path with its
/// first segment stripped are tried against the known pages.
fn classify_root_absolute(
    rest: &str,
    page_set: &HashSet<&str>,
    group_counts: &HashMap<&str, usize>,
) -> LinkResolution {
    let full = normalize_page_key(rest);
    let stripped = rest.split_once('/').map(|(_, s)| normalize_page_key(s));

    if page_set.contains(full.as_str()) {
        return LinkResolution::Valid(full);
    }
    if let Some(s) = &stripped
        && page_set.contains(s.as_str())
    {
        return LinkResolution::Valid(s.clone());
    }

    let clearly_docs = group_is_validatable(&full, group_counts)
        || stripped
            .as_ref()
            .is_some_and(|s| group_is_validatable(s, group_counts));
    if clearly_docs {
        LinkResolution::Broken
    } else {
        LinkResolution::Skip
    }
}

/// Classify a relative link (e.g. `../guides/foo`) resolved against the current
/// file's directory.
fn classify_relative(
    current_page: &str,
    path: &str,
    page_set: &HashSet<&str>,
    group_counts: &HashMap<&str, usize>,
) -> LinkResolution {
    let Some(resolved) = resolve_relative(current_page, path) else {
        return LinkResolution::Skip;
    };
    let resolved = normalize_page_key(&resolved);
    if resolved.is_empty() {
        return LinkResolution::Skip;
    }
    if page_set.contains(resolved.as_str()) {
        return LinkResolution::Valid(resolved);
    }
    if group_is_validatable(&resolved, group_counts) {
        LinkResolution::Broken
    } else {
        LinkResolution::Skip
    }
}

/// Return a diagnostic for a broken page target or missing anchor.
fn check_link(
    src: &str,
    current_page: &str,
    target: &str,
    page_set: &HashSet<&str>,
    group_counts: &HashMap<&str, usize>,
    headings: &HashMap<&str, HashSet<String>>,
) -> Option<String> {
    let (path_part, fragment) = match target.split_once('#') {
        Some((p, f)) => (p, Some(f)),
        None => (target, None),
    };

    // Same-page anchor (`#heading`).
    if path_part.is_empty() {
        return fragment.and_then(|frag| check_anchor(src, current_page, target, frag, headings));
    }

    // Skip external links (`https:`, `mailto:`, protocol-relative `//host`).
    if path_part.starts_with("//") || has_scheme(path_part) {
        return None;
    }

    let resolution = if let Some(rest) = path_part.strip_prefix('/') {
        classify_root_absolute(rest, page_set, group_counts)
    } else {
        classify_relative(current_page, path_part, page_set, group_counts)
    };

    match resolution {
        LinkResolution::Valid(page) => {
            fragment.and_then(|frag| check_anchor(src, &page, target, frag, headings))
        }
        LinkResolution::Broken => Some(format!(
            "{src}: internal link target \"{target}\" does not match any known docs page"
        )),
        LinkResolution::Skip => None,
    }
}

/// Validate that `fragment` matches a heading anchor in `page`. Skipped when the
/// target page's headings are unknown (its file was not read).
fn check_anchor(
    src: &str,
    page: &str,
    target: &str,
    fragment: &str,
    headings: &HashMap<&str, HashSet<String>>,
) -> Option<String> {
    if fragment.is_empty() {
        return None;
    }
    if let Some(anchors) = headings.get(page)
        && !anchors.contains(&slugify(fragment))
    {
        return Some(format!(
            "{src}: link \"{target}\" points to \"#{fragment}\" but no heading with that anchor exists in {page}"
        ));
    }
    None
}

/// Collect frontmatter and internal-link diagnostics for existing nav pages.
fn validate_docs(manifest_dir: &str, docs_dir: &str, pages: &[String]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    // Read each existing page once.
    let mut contents: Vec<(String, String)> = Vec::new();
    for page in pages {
        let mdx_path = format!("{docs_dir}/{page}.mdx");
        let full_path = include_path(manifest_dir, &mdx_path);
        if let Ok(raw) = fs::read_to_string(&full_path) {
            diagnostics.extend(validate_docs_frontmatter(&mdx_path, &raw));
            contents.push((page.clone(), raw));
        }
    }

    let page_set: HashSet<&str> = pages.iter().map(String::as_str).collect();

    let mut group_counts: HashMap<&str, usize> = HashMap::new();
    for page in pages {
        if let Some((group, _)) = page.split_once('/') {
            *group_counts.entry(group).or_insert(0) += 1;
        }
    }

    // Strip fenced code once and reuse for both headings and link scanning.
    let stripped: Vec<(String, String)> = contents
        .iter()
        .map(|(page, raw)| (page.clone(), strip_code_fences(raw)))
        .collect();

    let mut headings: HashMap<&str, HashSet<String>> = HashMap::new();
    for (page, body) in &stripped {
        headings.insert(
            page.as_str(),
            extract_heading_slugs(body).into_iter().collect(),
        );
    }

    for (page, body) in &stripped {
        let src = format!("{docs_dir}/{page}.mdx");
        for target in extract_links(body) {
            diagnostics.extend(check_link(
                &src,
                page,
                &target,
                &page_set,
                &group_counts,
                &headings,
            ));
        }
    }
    diagnostics
}

/// A docs page's frontmatter block (if present) must parse as a YAML mapping.
/// No particular fields are required for docs pages.
fn validate_docs_frontmatter(path: &str, content: &str) -> Vec<String> {
    let content = content.trim();
    if !content.starts_with("---") {
        return Vec::new();
    }
    let after = &content[3..];
    // No closing delimiter → not a frontmatter block (matches runtime behavior).
    let Some(end) = after.find("\n---") else {
        return Vec::new();
    };
    let yaml = after[..end].trim();
    if yaml.is_empty() {
        return Vec::new(); // an empty frontmatter block is valid
    }
    match serde_yaml::from_str::<serde_yaml::Value>(yaml) {
        Ok(serde_yaml::Value::Mapping(map)) => unsupported_frontmatter_shapes(&map)
            .into_iter()
            .map(|warning| format!("{path}: {warning}"))
            .collect(),
        // The runtime treats an unparseable leading block as page content and
        // still renders the page (a `---`-fenced paragraph is legal markdown),
        // so a hard build failure here would reject pages that work. Warn only.
        Ok(_) => vec![format!(
            "{path}: leading --- block is not a YAML mapping and will render as page content, not frontmatter"
        )],
        Err(e) => vec![format!(
            "{path}: leading --- block is not valid YAML ({e}) and will render as page content, not frontmatter"
        )],
    }
}

/// Report frontmatter shapes that are valid YAML but outside the subset the
/// runtime parser (`dioxus_mdx::parse_yaml_lite`) understands: a flat mapping
/// of scalars and scalar sequences.
///
/// Without this, a page can build cleanly and then silently lose its whole
/// frontmatter block at runtime (no title, no sidebar entry).
fn unsupported_frontmatter_shapes(map: &serde_yaml::Mapping) -> Vec<String> {
    let mut warnings = Vec::new();
    for (key, value) in map {
        let key = key.as_str().unwrap_or("<non-string key>");
        let problem = match value {
            serde_yaml::Value::Mapping(_) => "is a nested mapping",
            serde_yaml::Value::Sequence(items)
                if items.iter().any(|item| {
                    matches!(
                        item,
                        serde_yaml::Value::Mapping(_) | serde_yaml::Value::Sequence(_)
                    )
                }) =>
            {
                "has a non-scalar sequence item"
            }
            serde_yaml::Value::String(s) if s.contains('\n') => "spans multiple lines",
            _ => continue,
        };
        warnings.push(format!(
            "frontmatter key \"{key}\" {problem}, which the runtime parser rejects - \
             the whole frontmatter block will be dropped at runtime"
        ));
    }
    warnings
}

/// Parse a blog post's frontmatter block, returning it with the remaining body.
///
/// A post must have a valid block carrying the required fields, or the build
/// fails: it would otherwise silently vanish from the site. Parsing goes
/// through `dioxus_mdx::parse_yaml_lite`, the same YAML subset the runtime used
/// before 0.9.0, so nothing that used to build starts failing here.
fn parse_blog_frontmatter<'a>(
    path: &str,
    content: &'a str,
) -> Result<(BlogFrontmatter, &'a str), String> {
    let content = content.trim();
    if !content.starts_with("---") {
        return Err(format!(
            "{path}: missing frontmatter block (expected leading ---)"
        ));
    }
    let after = &content[3..];
    let Some(end) = after.find("\n---") else {
        return Err(format!(
            "{path}: unclosed frontmatter block (missing closing ---)"
        ));
    };
    let yaml = after[..end].trim();
    let body = after[end + 4..].trim_start();

    let malformed = |e| format!("{path}: malformed frontmatter: {e}");
    let map = dioxus_mdx::parse_yaml_lite(yaml).map_err(malformed)?;
    Ok((
        BlogFrontmatter {
            title: map.require_str("title").map_err(malformed)?,
            description: map.optional_str("description").map_err(malformed)?,
            date: map.require_str("date").map_err(malformed)?,
            author: map.require_str("author").map_err(malformed)?,
            tags: map.optional_str_seq("tags").map_err(malformed)?,
            cover_image: map.optional_str("coverImage").map_err(malformed)?,
            draft: map.optional_bool("draft").map_err(malformed)?,
            featured: map.optional_bool("featured").map_err(malformed)?,
        },
        body,
    ))
}

/// `_blog.json` category keys that no listed post is tagged with. Their
/// metadata is silently ignored at runtime, which usually means a typo.
fn unknown_category_keys(
    categories: &HashMap<String, serde_json::Value>,
    tags: &HashSet<String>,
) -> Vec<String> {
    let mut unknown: Vec<String> = categories
        .keys()
        .filter(|key| !tags.contains(*key))
        .cloned()
        .collect();
    unknown.sort();
    unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_keys_without_a_matching_tag_are_reported() {
        let categories: HashMap<String, serde_json::Value> =
            serde_json::from_str(r#"{"rust": {}, "Rust": {}, "wasm": {}}"#).unwrap();
        let tags = HashSet::from(["rust".to_string(), "wasm".to_string()]);
        assert_eq!(
            unknown_category_keys(&categories, &tags),
            vec!["Rust".to_string()]
        );
    }

    #[test]
    fn missing_page_is_a_diagnostic_and_is_not_bundled() {
        assert!(read_content("/nonexistent-docs-kit-test", "missing.mdx").is_none());
        let diagnostics = vec![missing_file_diagnostic(
            "/nonexistent-docs-kit-test",
            "missing",
            "missing.mdx",
        )];
        assert!(diagnostics[0].contains("missing.mdx"));
        report_diagnostics(&diagnostics, ValidationMode::Warn);
        assert!(
            std::panic::catch_unwind(|| report_diagnostics(&diagnostics, ValidationMode::Strict))
                .is_err()
        );
    }

    #[test]
    fn strict_validation_accepts_clean_content() {
        report_diagnostics(&[], ValidationMode::Strict);
        assert!(
            validate_docs_frontmatter("intro.mdx", "---\ntitle: Intro\n---\n## Setup").is_empty()
        );
    }

    #[test]
    fn link_diagnostics_include_source_target_and_anchor() {
        let pages = HashSet::from(["guides/intro", "guides/setup"]);
        let groups = HashMap::from([("guides", 2)]);
        let headings = HashMap::from([("guides/setup", HashSet::from(["install".to_string()]))]);
        let check = |target| {
            check_link(
                "docs/guides/intro.mdx",
                "guides/intro",
                target,
                &pages,
                &groups,
                &headings,
            )
        };
        assert!(check("setup#install").is_none());
        assert!(check("https://example.com/missing").is_none());
        let missing = check("missing").unwrap();
        assert!(missing.contains("docs/guides/intro.mdx"));
        assert!(missing.contains("\"missing\""));
        let anchor = check("setup#nope").unwrap();
        assert!(anchor.contains("#nope"));
        assert!(anchor.contains("guides/setup"));
    }

    #[test]
    fn unsupported_frontmatter_is_available_to_strict_validation() {
        let diagnostics = validate_docs_frontmatter(
            "intro.mdx",
            "---\ntitle: Intro\nauthor:\n  name: Jane\n---\nBody",
        );
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].contains("intro.mdx"));
        assert!(diagnostics[0].contains("nested mapping"));
    }

    #[test]
    fn include_path_joins_with_forward_slash() {
        assert_eq!(
            include_path("/home/me/project", "docs/intro.mdx"),
            "/home/me/project/docs/intro.mdx"
        );
    }

    #[test]
    fn include_path_normalizes_windows_backslashes() {
        assert_eq!(
            include_path("C:\\Users\\me\\project", "docs\\intro.mdx"),
            "C:/Users/me/project/docs/intro.mdx"
        );
    }

    // ---- link validation ---------------------------------------------------

    #[test]
    fn extract_links_skips_images() {
        let md = "see ![alt](/img/logo.png) and [Quickstart](/docs/getting-started/quickstart)";
        assert_eq!(
            extract_links(md),
            vec!["/docs/getting-started/quickstart".to_string()]
        );
    }

    #[test]
    fn extract_links_strips_title_and_angle_brackets() {
        let md = "[a](/docs/x \"the title\") and [b](</docs/y>)";
        assert_eq!(
            extract_links(md),
            vec!["/docs/x".to_string(), "/docs/y".to_string()]
        );
    }

    #[test]
    fn strip_code_fences_removes_fenced_links() {
        let md = "before\n```\n[not a link](/docs/nope)\n```\nafter [real](/docs/real)";
        let body = strip_code_fences(md);
        assert!(!body.contains("nope"));
        assert_eq!(extract_links(&body), vec!["/docs/real".to_string()]);
    }

    #[test]
    fn has_scheme_detects_external() {
        assert!(has_scheme("https://example.com"));
        assert!(has_scheme("mailto:me@example.com"));
        assert!(!has_scheme("/docs/guides/x"));
        assert!(!has_scheme("guides/x"));
        assert!(!has_scheme("../guides/x"));
    }

    #[test]
    fn resolve_relative_resolves_against_dir() {
        assert_eq!(
            resolve_relative("guides/blog", "customization").as_deref(),
            Some("guides/customization")
        );
        assert_eq!(
            resolve_relative("guides/blog", "../guides/customization").as_deref(),
            Some("guides/customization")
        );
        assert_eq!(
            resolve_relative("getting-started/introduction", "../guides/basic-usage").as_deref(),
            Some("guides/basic-usage")
        );
        // Escapes the docs root.
        assert_eq!(resolve_relative("changelog", "../../x"), None);
    }

    #[test]
    fn extract_heading_slugs_covers_h2_to_h4_only() {
        let md = "# Title\n## Section One\n### Sub Section\n##### Too Deep\ntext\n";
        assert_eq!(
            extract_heading_slugs(md),
            vec!["section-one".to_string(), "sub-section".to_string()]
        );
    }

    fn sample_page_data() -> (Vec<&'static str>, HashMap<&'static str, usize>) {
        let pages = vec![
            "getting-started/introduction",
            "getting-started/quickstart",
            "guides/basic-usage",
            "guides/customization",
            "guides/integration",
            "guides/blog",
            "api-reference/overview",
            "changelog",
        ];
        let mut group_counts: HashMap<&str, usize> = HashMap::new();
        for p in &pages {
            if let Some((g, _)) = p.split_once('/') {
                *group_counts.entry(g).or_insert(0) += 1;
            }
        }
        (pages, group_counts)
    }

    #[test]
    fn root_absolute_heuristic() {
        let (pages, group_counts) = sample_page_data();
        let page_set: HashSet<&str> = pages.iter().copied().collect();

        // Valid under a `/docs` base path.
        assert!(matches!(
            classify_root_absolute("docs/guides/basic-usage", &page_set, &group_counts),
            LinkResolution::Valid(_)
        ));
        // Valid under an empty (`/`) base path.
        assert!(matches!(
            classify_root_absolute("getting-started/introduction", &page_set, &group_counts),
            LinkResolution::Valid(_)
        ));
        // Broken: a dense group, but no such page.
        assert!(matches!(
            classify_root_absolute("docs/guides/nope", &page_set, &group_counts),
            LinkResolution::Broken
        ));
        // Skip: api-reference has a single static page; the rest are runtime
        // OpenAPI operations the build crate cannot see.
        assert!(matches!(
            classify_root_absolute("docs/api-reference/getUser", &page_set, &group_counts),
            LinkResolution::Skip
        ));
        // Skip: non-docs routes.
        assert!(matches!(
            classify_root_absolute("blog/hello", &page_set, &group_counts),
            LinkResolution::Skip
        ));
    }

    #[test]
    fn relative_heuristic() {
        let (pages, group_counts) = sample_page_data();
        let page_set: HashSet<&str> = pages.iter().copied().collect();

        assert!(matches!(
            classify_relative(
                "guides/basic-usage",
                "customization",
                &page_set,
                &group_counts
            ),
            LinkResolution::Valid(_)
        ));
        assert!(matches!(
            classify_relative("guides/basic-usage", "nope", &page_set, &group_counts),
            LinkResolution::Broken
        ));
        // Relative link into the single-page (OpenAPI) group is skipped.
        assert!(matches!(
            classify_relative(
                "api-reference/overview",
                "get-user",
                &page_set,
                &group_counts
            ),
            LinkResolution::Skip
        ));
    }

    // ---- frontmatter validation --------------------------------------------

    #[test]
    fn docs_frontmatter_valid_and_empty_ok() {
        validate_docs_frontmatter("x.mdx", "---\ntitle: Hi\n---\nbody");
        validate_docs_frontmatter("x.mdx", "---\n---\nbody");
        validate_docs_frontmatter("x.mdx", "no frontmatter here");
    }

    #[test]
    fn docs_frontmatter_unparseable_block_warns_but_does_not_panic() {
        // The runtime renders these pages (the block is treated as content),
        // so the build must not reject them — it only emits cargo:warning.
        validate_docs_frontmatter("x.mdx", "---\ntitle: [unclosed\n---\nbody");
        validate_docs_frontmatter("x.mdx", "---\n- a\n- b\n---\nbody");
        validate_docs_frontmatter("x.mdx", "---\nJust a fenced paragraph.\n---\nbody");
    }

    fn shape_warnings(yaml: &str) -> Vec<String> {
        let map: serde_yaml::Mapping = serde_yaml::from_str(yaml).expect("valid YAML mapping");
        unsupported_frontmatter_shapes(&map)
    }

    #[test]
    fn runtime_supported_frontmatter_shapes_warn_about_nothing() {
        assert!(
            shape_warnings(
                "title: Hi\ndescription: ~\ntags: [a, b]\nlist:\n  - one\ndraft: true\ncount: 3"
            )
            .is_empty()
        );
    }

    #[test]
    fn frontmatter_shapes_the_runtime_rejects_warn() {
        // Nested mapping.
        let warnings = shape_warnings("title: Hi\nauthor:\n  name: Jane");
        assert_eq!(warnings.len(), 1, "got: {warnings:?}");
        assert!(warnings[0].contains("\"author\""), "got: {warnings:?}");
        assert!(warnings[0].contains("nested mapping"), "got: {warnings:?}");

        // Non-scalar sequence item.
        let warnings = shape_warnings("tags:\n  - name: rust");
        assert_eq!(warnings.len(), 1, "got: {warnings:?}");
        assert!(
            warnings[0].contains("non-scalar sequence item"),
            "got: {warnings:?}"
        );

        // Multi-line (block) scalar.
        let warnings = shape_warnings("description: |\n  line one\n  line two");
        assert_eq!(warnings.len(), 1, "got: {warnings:?}");
        assert!(warnings[0].contains("multiple lines"), "got: {warnings:?}");
    }

    #[test]
    fn blog_frontmatter_valid_ok() {
        let (fm, body) = parse_blog_frontmatter(
            "p.mdx",
            "---\ntitle: Hi\ndate: \"2026-01-01\"\nauthor: jane\n---\nbody",
        )
        .unwrap();
        assert_eq!(fm.title, "Hi");
        assert_eq!(fm.date, "2026-01-01");
        assert_eq!(body, "body");
    }

    #[test]
    fn blog_frontmatter_bad_yaml_is_an_error() {
        let err = parse_blog_frontmatter(
            "p.mdx",
            "---\ntitle: Hi\ndate: \"2026\"\nauthor:\n  name: jane\n---\nbody",
        )
        .unwrap_err();
        assert!(err.contains("malformed frontmatter"), "got: {err}");
    }

    #[test]
    fn blog_frontmatter_missing_field_is_an_error() {
        // No `date` field.
        let err =
            parse_blog_frontmatter("p.mdx", "---\ntitle: Hi\nauthor: jane\n---\nbody").unwrap_err();
        assert!(err.contains("date"), "got: {err}");
    }

    #[test]
    fn blog_frontmatter_no_block_is_an_error() {
        let err = parse_blog_frontmatter("p.mdx", "just body, no frontmatter").unwrap_err();
        assert!(err.contains("missing frontmatter"), "got: {err}");
    }

    #[test]
    fn blog_frontmatter_wrong_typed_optional_field_is_an_error() {
        // `tags` must be a sequence; a scalar used to silently drop the post at
        // runtime, so it must fail the build.
        let err = parse_blog_frontmatter(
            "p.mdx",
            "---\ntitle: Hi\ndate: \"2026-01-01\"\nauthor: jane\ntags: rust\n---\nbody",
        )
        .unwrap_err();
        assert!(err.contains("tags"), "got: {err}");
    }

    #[test]
    fn blog_frontmatter_optional_fields_ok() {
        let (fm, _) = parse_blog_frontmatter(
            "p.mdx",
            "---\ntitle: Hi\ndate: \"2026-01-01\"\nauthor: jane\ntags: [rust, web]\ndraft: true\ncoverImage: cover.png\n---\nbody",
        )
        .unwrap();
        assert_eq!(fm.tags, vec!["rust".to_string(), "web".to_string()]);
        assert!(fm.draft);
        assert_eq!(fm.cover_image.as_deref(), Some("cover.png"));
    }
}
