//! The build-time content bundle: what `dioxus-docs-kit-build` writes and
//! `dioxus-docs-kit` deserializes.
//!
//! Every document is parsed, every OpenAPI spec is resolved and the whole
//! search index is scored-ready *here*, so the browser only runs `serde_json`
//! over the result. The structs below are serialize-only mirrors of the
//! runtime types in `dioxus-docs-kit` (`DocsBundle`, `SearchEntry`, `BlogPost`,
//! …); `tests/bundle_contract.rs` round-trips a real bundle through those
//! types so the two definitions cannot drift apart silently.

use dioxus_mdx::{DocNode, HttpMethod, OpenApiSpec, ParsedDoc, slugify};
use serde::Serialize;
use serde_json::Value;

/// Bumped when the bundle layout changes incompatibly; the runtime refuses a
/// bundle it does not understand instead of failing field by field.
pub(crate) const BUNDLE_VERSION: u32 = 1;

fn is_empty_str(s: &str) -> bool {
    s.is_empty()
}

// ============================================================================
// Docs bundle
// ============================================================================

#[derive(Serialize)]
pub(crate) struct DocsBundle {
    pub version: u32,
    /// `_nav.json` verbatim — the runtime owns its own `NavConfig` shape.
    pub nav: Value,
    /// Parsed pages, in `_nav.json` order.
    pub docs: Vec<(String, ParsedDoc)>,
    /// Parsed OpenAPI specs keyed by URL prefix.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub openapi: Vec<(String, OpenApiSpec)>,
    /// Section-level search index, lowercase fields included.
    pub search: Vec<SearchEntry>,
}

/// Build the docs bundle JSON from content you have already loaded.
///
/// [`DocsBuild::generate`](crate::DocsBuild::generate) reads the files and calls
/// this; use it directly when your pages come from somewhere other than the
/// filesystem, or in tests.
///
/// - `nav_json`: the `_nav.json` document.
/// - `pages`: content path (e.g. `"guides/install"`) → raw MDX.
/// - `specs`: URL prefix (e.g. `"api-reference"`) → raw OpenAPI YAML or JSON.
///
/// The returned string is what [`DocsConfig::new`] expects.
///
/// [`DocsConfig::new`]: https://docs.rs/dioxus-docs-kit/latest/dioxus_docs_kit/config/struct.DocsConfig.html#method.new
pub fn docs_bundle_json(
    nav_json: &str,
    pages: &[(&str, &str)],
    specs: &[(&str, &str)],
) -> Result<String, String> {
    let nav: Value =
        serde_json::from_str(nav_json).map_err(|e| format!("failed to parse _nav.json: {e}"))?;
    let nav_groups = nav_groups_of(&nav)?;

    let docs: Vec<(String, ParsedDoc)> = pages
        .iter()
        .map(|(path, mdx)| ((*path).to_string(), dioxus_mdx::parse_document(mdx)))
        .collect();

    let openapi: Vec<(String, OpenApiSpec)> = specs
        .iter()
        .map(|(prefix, raw)| {
            dioxus_mdx::parse_openapi(raw)
                .map(|spec| ((*prefix).to_string(), spec))
                .map_err(|e| format!("failed to parse the OpenAPI spec for \"{prefix}\": {e}"))
        })
        .collect::<Result<_, _>>()?;

    let search = build_search_index(&nav_groups, &docs, &openapi);
    serde_json::to_string(&DocsBundle {
        version: BUNDLE_VERSION,
        nav,
        docs,
        openapi,
        search,
    })
    .map_err(|e| format!("failed to serialize the docs bundle: {e}"))
}

/// `(group name, pages)` pairs, in `_nav.json` order.
fn nav_groups_of(nav: &Value) -> Result<Vec<(String, Vec<String>)>, String> {
    #[derive(serde::Deserialize)]
    struct Nav {
        groups: Vec<Group>,
    }
    #[derive(serde::Deserialize)]
    struct Group {
        #[serde(default)]
        group: String,
        pages: Vec<String>,
    }
    let nav: Nav = serde_json::from_value(nav.clone())
        .map_err(|e| format!("failed to parse _nav.json: {e}"))?;
    Ok(nav.groups.into_iter().map(|g| (g.group, g.pages)).collect())
}

/// One searchable section. Mirrors `dioxus_docs_kit::SearchEntry`.
#[derive(Serialize)]
pub(crate) struct SearchEntry {
    pub path: String,
    #[serde(skip_serializing_if = "is_empty_str")]
    pub anchor: String,
    pub title: String,
    #[serde(skip_serializing_if = "is_empty_str")]
    pub heading: String,
    #[serde(skip_serializing_if = "is_empty_str")]
    pub description: String,
    #[serde(skip_serializing_if = "is_empty_str")]
    pub body: String,
    #[serde(skip_serializing_if = "is_empty_str")]
    pub breadcrumb: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_method: Option<HttpMethod>,
    pub title_lower: String,
    #[serde(skip_serializing_if = "is_empty_str")]
    pub heading_lower: String,
    #[serde(skip_serializing_if = "is_empty_str")]
    pub description_lower: String,
    #[serde(skip_serializing_if = "is_empty_str")]
    pub body_lower: String,
}

impl SearchEntry {
    #[allow(clippy::too_many_arguments)]
    fn new(
        path: String,
        anchor: String,
        title: String,
        heading: String,
        description: String,
        body: String,
        breadcrumb: String,
        api_method: Option<HttpMethod>,
    ) -> Self {
        Self {
            title_lower: search_lower(&title),
            heading_lower: search_lower(&heading),
            description_lower: search_lower(&description),
            body_lower: search_lower(&body),
            path,
            anchor,
            title,
            heading,
            description,
            body,
            breadcrumb,
            api_method,
        }
    }
}

/// Build the docs search index: one entry per document *section* (split on
/// h2–h4 headings) plus one page-level entry per OpenAPI operation.
///
/// API entries carry the bare tag as their breadcrumb; the registry prefixes it
/// with the configured API group name at load time, so renaming that group stays
/// a single runtime setting.
fn build_search_index(
    nav_groups: &[(String, Vec<String>)],
    docs: &[(String, ParsedDoc)],
    openapi: &[(String, OpenApiSpec)],
) -> Vec<SearchEntry> {
    let mut entries = Vec::new();

    for (group, pages) in nav_groups {
        for page in pages {
            let Some((_, doc)) = docs.iter().find(|(path, _)| path == page) else {
                continue;
            };
            let title = if doc.frontmatter.title.is_empty() {
                page.split('/')
                    .next_back()
                    .unwrap_or(page)
                    .replace('-', " ")
            } else {
                doc.frontmatter.title.clone()
            };
            let description = doc.frontmatter.description.clone().unwrap_or_default();

            let sections = split_into_sections(&doc.raw_markdown);
            let has_headings = sections.iter().any(|s| !s.heading.is_empty());
            for section in sections {
                let body = clean_markdown(&section.body);
                // Drop an empty intro once real sections exist; keep it for a
                // heading-less page so it stays findable by title.
                if section.heading.is_empty() && body.is_empty() && has_headings {
                    continue;
                }
                entries.push(SearchEntry::new(
                    page.clone(),
                    section.anchor,
                    title.clone(),
                    section.heading,
                    description.clone(),
                    body,
                    group.clone(),
                    None,
                ));
            }
        }
    }

    for (prefix, spec) in openapi {
        for op in &spec.operations {
            let title = op
                .summary
                .clone()
                .unwrap_or_else(|| op.slug().replace('-', " "));
            let description = op.description.clone().unwrap_or_default();
            let tag = op
                .tags
                .first()
                .cloned()
                .unwrap_or_else(|| "Other".to_string());

            entries.push(SearchEntry::new(
                format!("{prefix}/{}", op.slug()),
                String::new(),
                title,
                String::new(),
                description.clone(),
                clean_markdown(&description),
                tag,
                Some(op.method),
            ));
        }
    }

    entries
}

// ============================================================================
// Blog bundle
// ============================================================================

#[derive(Serialize)]
pub(crate) struct BlogBundle {
    pub version: u32,
    /// `_blog.json`'s `authors` map verbatim.
    pub authors: Value,
    /// `_blog.json`'s `categories` map verbatim.
    pub categories: Value,
    /// Published posts (drafts dropped), newest first.
    pub posts: Vec<BlogPost>,
    pub search: Vec<BlogSearchEntry>,
}

/// Mirrors `dioxus_docs_kit::BlogPost`.
#[derive(Serialize)]
pub(crate) struct BlogPost {
    pub slug: String,
    pub frontmatter: BlogFrontmatter,
    pub content: Vec<DocNode>,
    pub raw_markdown: String,
    pub reading_time_minutes: u32,
}

/// Mirrors `dioxus_docs_kit::BlogFrontmatter`.
#[derive(Debug, Serialize)]
pub(crate) struct BlogFrontmatter {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub date: String,
    pub author: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_image: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub draft: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub featured: bool,
}

/// Mirrors `dioxus_docs_kit::BlogSearchEntry`.
#[derive(Serialize)]
pub(crate) struct BlogSearchEntry {
    pub slug: String,
    pub title: String,
    #[serde(skip_serializing_if = "is_empty_str")]
    pub description: String,
    #[serde(skip_serializing_if = "is_empty_str")]
    pub body: String,
    pub date: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    pub title_lower: String,
    #[serde(skip_serializing_if = "is_empty_str")]
    pub description_lower: String,
    #[serde(skip_serializing_if = "is_empty_str")]
    pub body_lower: String,
}

/// Build the blog bundle JSON from content you have already loaded.
///
/// [`BlogBuild::generate`](crate::BlogBuild::generate) reads the files and calls
/// this; use it directly when your posts come from somewhere other than the
/// filesystem, or in tests.
///
/// - `manifest_json`: the `_blog.json` document (`authors` and `categories` are
///   carried through verbatim; its `posts` list is not consulted here).
/// - `posts`: slug → raw MDX, including frontmatter.
///
/// Drafts are dropped and the remaining posts sorted newest first, exactly as
/// the runtime used to do. Returns the first malformed post's error.
pub fn blog_bundle_json(manifest_json: &str, posts: &[(&str, &str)]) -> Result<String, String> {
    #[derive(serde::Deserialize)]
    struct Manifest {
        #[serde(default)]
        authors: Value,
        #[serde(default)]
        categories: Value,
    }
    let manifest: Manifest = serde_json::from_str(manifest_json)
        .map_err(|e| format!("failed to parse _blog.json: {e}"))?;

    let mut built = Vec::new();
    for (slug, mdx) in posts {
        let (frontmatter, body) = crate::parse_blog_frontmatter(slug, mdx)?;
        if frontmatter.draft {
            continue;
        }
        built.push(build_post(slug, frontmatter, body));
    }
    sort_posts(&mut built);

    let search = build_blog_search_index(&built);
    serde_json::to_string(&BlogBundle {
        version: BUNDLE_VERSION,
        authors: normalize_map(manifest.authors),
        categories: normalize_map(manifest.categories),
        posts: built,
        search,
    })
    .map_err(|e| format!("failed to serialize the blog bundle: {e}"))
}

/// A missing `authors`/`categories` key deserializes to `null`, which the
/// runtime cannot read as a map; emit an empty object instead.
fn normalize_map(value: Value) -> Value {
    if value.is_object() {
        value
    } else {
        Value::Object(serde_json::Map::new())
    }
}

/// Parse a post body into the bundle's node tree, reading time included.
pub(crate) fn build_post(slug: &str, frontmatter: BlogFrontmatter, body: &str) -> BlogPost {
    // Blog post views render the frontmatter title in their own <h1>; strip a
    // duplicate body H1 so each page emits exactly one.
    let (content, raw_markdown) = dioxus_mdx::parse_body(dioxus_mdx::strip_leading_h1(body));
    BlogPost {
        slug: slug.to_string(),
        reading_time_minutes: reading_time_minutes(&raw_markdown),
        frontmatter,
        content,
        raw_markdown,
    }
}

/// Newest first, slug-ascending within a date.
pub(crate) fn sort_posts(posts: &mut [BlogPost]) {
    posts.sort_by(|a, b| {
        b.frontmatter
            .date
            .cmp(&a.frontmatter.date)
            .then_with(|| a.slug.cmp(&b.slug))
    });
}

pub(crate) fn build_blog_search_index(posts: &[BlogPost]) -> Vec<BlogSearchEntry> {
    posts
        .iter()
        .map(|post| {
            let title = post.frontmatter.title.clone();
            let description = post.frontmatter.description.clone().unwrap_or_default();
            let body = clean_markdown(&post.raw_markdown);
            BlogSearchEntry {
                slug: post.slug.clone(),
                title_lower: search_lower(&title),
                description_lower: search_lower(&description),
                body_lower: search_lower(&body),
                title,
                description,
                body,
                date: post.frontmatter.date.clone(),
                tags: post.frontmatter.tags.clone(),
            }
        })
        .collect()
}

/// Calculate reading time from raw text (words / 200 WPM, minimum 1 minute).
pub(crate) fn reading_time_minutes(text: &str) -> u32 {
    let word_count = text.split_whitespace().count();
    ((word_count as f64 / 200.0).ceil() as u32).max(1)
}

// ============================================================================
// Section splitting and text normalisation
// ============================================================================

/// A single documentation section produced by [`split_into_sections`].
struct Section {
    /// Heading text, empty for the leading intro section.
    heading: String,
    /// Anchor id (`slugify(heading)`), empty for the intro section.
    anchor: String,
    /// Raw markdown body for this section (before [`clean_markdown`]).
    body: String,
}

/// Split reconstructed markdown into sections on its h2–h4 ATX headings.
///
/// The slice before the first heading is always returned first as the intro
/// section (empty `heading`/`anchor`). Fenced code blocks are skipped so a `##`
/// inside code is not mistaken for a heading. Anchors use the same
/// `dioxus_mdx::slugify` the renderer applies to heading ids.
fn split_into_sections(raw: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut heading = String::new();
    let mut anchor = String::new();
    let mut body = String::new();
    let mut fence: Option<char> = None;

    for line in raw.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            let marker = if trimmed.starts_with("```") { '`' } else { '~' };
            match fence {
                None => fence = Some(marker),
                Some(open) if open == marker => fence = None,
                Some(_) => {} // the other marker inside a fence is literal content
            }
            body.push_str(line);
            body.push('\n');
            continue;
        }
        if fence.is_none()
            && let Some((_, text)) = dioxus_mdx::parse_atx_heading(trimmed)
        {
            sections.push(Section {
                heading: std::mem::take(&mut heading),
                anchor: std::mem::take(&mut anchor),
                body: std::mem::take(&mut body),
            });
            anchor = slugify(text);
            heading = text.to_string();
            continue;
        }
        body.push_str(line);
        body.push('\n');
    }
    sections.push(Section {
        heading,
        anchor,
        body,
    });
    sections
}

/// One char in, one char out, so char offsets stay aligned with the original
/// text (the runtime's snippet highlight ranges rely on this).
fn lower_char(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

/// Lowercase a string for case-insensitive matching.
///
/// Must stay byte-for-byte identical to `dioxus_docs_kit`'s own `search_lower`,
/// which folds the *query* the same way at runtime.
fn search_lower(s: &str) -> String {
    s.chars().map(lower_char).collect()
}

/// Collapse a section's raw markdown into plain-ish text for matching and
/// snippet display: fenced code blocks are dropped, `[text](url)` links reduce
/// to their text, the noisiest inline markers are stripped, and whitespace is
/// collapsed to single spaces. Intentionally lightweight — not a full markdown
/// renderer.
fn clean_markdown(md: &str) -> String {
    // Drop fenced code blocks wholesale (noise, and their `##` lines are not
    // real headings).
    let mut no_code = String::with_capacity(md.len());
    let mut fence: Option<char> = None;
    for line in md.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            let marker = if trimmed.starts_with("```") { '`' } else { '~' };
            match fence {
                None => fence = Some(marker),
                Some(open) if open == marker => fence = None,
                Some(_) => {} // the other marker inside a fence is literal content
            }
            continue;
        }
        if fence.is_some() {
            continue;
        }
        no_code.push_str(line);
        no_code.push('\n');
    }

    // Collapse `[text](url)` to `text`, strip the noisiest inline markers, and
    // squeeze runs of whitespace to a single space.
    let mut out = String::with_capacity(no_code.len());
    let mut prev_space = false;
    let mut chars = no_code.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '[' {
            let mut text = String::new();
            for tc in chars.by_ref() {
                if tc == ']' {
                    break;
                }
                text.push(tc);
            }
            if chars.peek() == Some(&'(') {
                chars.next();
                for uc in chars.by_ref() {
                    if uc == ')' {
                        break;
                    }
                }
            }
            for tc in text.chars() {
                push_clean(&mut out, tc, &mut prev_space);
            }
        } else {
            push_clean(&mut out, c, &mut prev_space);
        }
    }

    out.trim().to_string()
}

fn push_clean(out: &mut String, c: char, prev_space: &mut bool) {
    if matches!(c, '#' | '*' | '_' | '`' | '>' | '|' | '\\') {
        return;
    }
    if c.is_whitespace() {
        if !*prev_space {
            out.push(' ');
            *prev_space = true;
        }
    } else {
        out.push(c);
        *prev_space = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sections_split_on_headings_with_intro_and_slugified_anchors() {
        let sections = split_into_sections(
            "Intro text.\n\n## Installation Steps\n\nRun it.\n\n### Advanced Setup\n\nTweak it.\n",
        );
        assert_eq!(sections.len(), 3);

        assert_eq!(sections[0].heading, "");
        assert_eq!(sections[0].anchor, "");
        assert!(sections[0].body.contains("Intro text."));

        assert_eq!(sections[1].heading, "Installation Steps");
        assert_eq!(sections[1].anchor, "installation-steps");
        assert!(sections[1].body.contains("Run it."));

        assert_eq!(sections[2].heading, "Advanced Setup");
        assert_eq!(sections[2].anchor, "advanced-setup");
    }

    #[test]
    fn sections_skip_headings_inside_code_fences() {
        let sections = split_into_sections(
            "Intro.\n\n```md\n## Not A Heading\n```\n\n## Real Heading\n\nBody.\n",
        );
        let headings: Vec<&str> = sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, vec!["", "Real Heading"]);
    }

    #[test]
    fn empty_leading_section_kept_only_when_page_has_no_headings() {
        let sections = split_into_sections("## First\n\nbody\n");
        assert_eq!(sections[0].heading, "");
        assert!(sections[0].body.trim().is_empty());
    }

    #[test]
    fn clean_markdown_drops_code_fences_and_strips_markers() {
        let md =
            "Intro **bold** text\n\n```rust\nlet x = 1;\n```\n\nSee [the docs](https://x.y) now";
        let cleaned = clean_markdown(md);
        assert!(!cleaned.contains("let x"), "code fence body dropped");
        assert!(!cleaned.contains('*'), "emphasis markers stripped");
        assert!(cleaned.contains("the docs"), "link text kept");
        assert!(!cleaned.contains("https://"), "link target dropped");
        assert!(!cleaned.contains('\n'), "whitespace collapsed");
    }

    #[test]
    fn search_lower_folds_case_one_char_per_char() {
        assert_eq!(search_lower("HELLO"), "hello");
        assert_eq!(search_lower("CafÉ"), "café");
        assert_eq!(search_lower("AbC").chars().count(), 3);
    }

    #[test]
    fn reading_time_rounds_up_with_minimum() {
        assert_eq!(reading_time_minutes("a few words"), 1);
        assert_eq!(reading_time_minutes(&"word ".repeat(400)), 2);
    }
}
