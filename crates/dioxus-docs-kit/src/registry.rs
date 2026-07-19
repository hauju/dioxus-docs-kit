//! Documentation content registry.
//!
//! Holds parsed docs, nav config, search index, and OpenAPI specs.

#[cfg(feature = "highlight")]
use crate::config::CodeThemeConfig;
use crate::config::{DocsConfig, ThemeConfig};
use crate::error::DocsKitError;
use crate::search::{Field, clean_markdown, search_lower};
use dioxus_mdx::{
    ApiOperation, ApiTag, HttpMethod, OpenApiSpec, ParsedDoc, parse_document, parse_openapi,
    slugify,
};
use serde::Deserialize;
use std::collections::HashMap;

/// Navigation configuration for the documentation sidebar.
#[derive(Debug, Clone, Deserialize)]
pub struct NavConfig {
    #[serde(default)]
    pub tabs: Vec<String>,
    pub groups: Vec<NavGroup>,
}

impl NavConfig {
    /// Whether the nav config has multiple tabs.
    pub fn has_tabs(&self) -> bool {
        self.tabs.len() > 1
    }

    /// Get groups belonging to a specific tab.
    pub fn groups_for_tab(&self, tab: &str) -> Vec<&NavGroup> {
        self.groups
            .iter()
            .filter(|g| g.tab.as_deref() == Some(tab))
            .collect()
    }
}

/// A group of navigation items in the sidebar.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct NavGroup {
    pub group: String,
    #[serde(default)]
    pub tab: Option<String>,
    pub pages: Vec<String>,
}

/// A sidebar entry for an API endpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiEndpointEntry {
    /// URL prefix of the spec that owns this endpoint (e.g. "api-reference").
    pub prefix: String,
    /// URL slug (e.g. "list-pets").
    pub slug: String,
    /// Display title (summary or fallback).
    pub title: String,
    /// HTTP method.
    pub method: HttpMethod,
}

/// A searchable entry in the documentation.
///
/// One entry per document *section* (split on h2–h4 headings): content before
/// the first heading is a page-level "intro" entry with an empty `heading` /
/// `anchor`, and each heading starts a new entry whose `anchor` deep-links to
/// the rendered heading id. OpenAPI operations are indexed as single page-level
/// entries. The `*_lower` fields are lowercased once at build time so search
/// never re-lowercases per keystroke.
#[derive(PartialEq)]
pub struct SearchEntry {
    /// Content path of the owning page (e.g. "getting-started/introduction").
    pub path: String,
    /// Anchor id of the section heading, empty for page-level / intro entries.
    /// Matches the rendered heading `id` (`dioxus_mdx::slugify`).
    pub anchor: String,
    /// Page title (frontmatter title or humanised slug / API summary).
    pub title: String,
    /// Section heading text, empty for page-level / intro entries.
    pub heading: String,
    /// Page/operation description.
    pub description: String,
    /// Cleaned section body text used for matching and snippet extraction.
    pub body: String,
    /// Sidebar breadcrumb (nav group, or API group + tag).
    pub breadcrumb: String,
    /// HTTP method for API operation entries (drives the result badge).
    pub api_method: Option<HttpMethod>,
    pub(crate) title_lower: String,
    pub(crate) heading_lower: String,
    pub(crate) description_lower: String,
    pub(crate) body_lower: String,
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
        let title_lower = search_lower(&title);
        let heading_lower = search_lower(&heading);
        let description_lower = search_lower(&description);
        let body_lower = search_lower(&body);
        Self {
            path,
            anchor,
            title,
            heading,
            description,
            body,
            breadcrumb,
            api_method,
            title_lower,
            heading_lower,
            description_lower,
            body_lower,
        }
    }
}

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
        if fence.is_none() {
            if let Some(text) = parse_atx_heading(trimmed) {
                sections.push(Section {
                    heading: std::mem::take(&mut heading),
                    anchor: std::mem::take(&mut anchor),
                    body: std::mem::take(&mut body),
                });
                anchor = slugify(text);
                heading = text.to_string();
                continue;
            }
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

/// Return the heading text of an h2–h4 ATX heading line, or `None`.
///
/// Mirrors the TOC heading regex (`^#{2,4}\s+\S`): 2–4 leading `#`, at least one
/// space/tab, then non-empty text (h1/h5/h6 are ignored).
fn parse_atx_heading(line: &str) -> Option<&str> {
    let hashes = line.bytes().take_while(|&b| b == b'#').count();
    if !(2..=4).contains(&hashes) {
        return None;
    }
    let rest = &line[hashes..];
    if !rest.starts_with([' ', '\t']) {
        return None;
    }
    let text = rest.trim();
    if text.is_empty() {
        return None;
    }
    Some(text)
}

/// Central documentation registry holding all parsed content.
///
/// Created via [`DocsConfig`] builder and typically stored in a `Lazy<DocsRegistry>` static.
/// Provide it to UI components via `use_context_provider(|| &*DOCS as &'static DocsRegistry)`.
pub struct DocsRegistry {
    /// Navigation configuration.
    pub nav: NavConfig,
    /// Pre-parsed documentation pages.
    parsed_docs: HashMap<&'static str, ParsedDoc>,
    /// Prebuilt search index.
    search_index: Vec<SearchEntry>,
    /// OpenAPI specs keyed by URL prefix.
    openapi_specs: Vec<(String, OpenApiSpec)>,
    /// Precomputed API sidebar entries grouped by tag.
    api_sidebar_entries: Vec<(ApiTag, Vec<ApiEndpointEntry>)>,
    /// Full docs path ("prefix/slug") → (spec index, operation index).
    api_operation_index: HashMap<String, (usize, usize)>,
    /// Default page path for redirects.
    pub default_path: String,
    /// Display name for the API Reference sidebar group.
    pub api_group_name: String,
    /// Optional theme configuration.
    pub theme: Option<ThemeConfig>,
    /// Syntax-highlighting theme for code blocks.
    #[cfg(feature = "highlight")]
    pub code_theme: CodeThemeConfig,
}

impl DocsRegistry {
    /// Build a registry from a [`DocsConfig`].
    pub(crate) fn try_from_config(config: DocsConfig) -> Result<Self, DocsKitError> {
        let nav: NavConfig =
            serde_json::from_str(config.nav_json()).map_err(DocsKitError::NavParse)?;

        // Parse all documents
        let parsed_docs: HashMap<&'static str, ParsedDoc> = config
            .content_map()
            .iter()
            .map(|(&path, &content)| (path, parse_document(content)))
            .collect();

        // Parse OpenAPI specs
        let openapi_specs: Vec<(String, OpenApiSpec)> = config
            .openapi_specs()
            .iter()
            .map(|(prefix, yaml)| {
                parse_openapi(yaml)
                    .map(|spec| (prefix.clone(), spec))
                    .map_err(|error| DocsKitError::OpenApi {
                        prefix: prefix.clone(),
                        error,
                    })
            })
            .collect::<Result<_, _>>()?;

        // Determine default path
        let default_path = config
            .default_path_value()
            .map(String::from)
            .unwrap_or_else(|| {
                nav.groups
                    .first()
                    .and_then(|g| g.pages.first())
                    .cloned()
                    .unwrap_or_default()
            });

        let api_group_name = config
            .api_group_name_value()
            .map(String::from)
            .unwrap_or_else(|| "API Reference".to_string());

        let theme = config.theme_config().cloned();
        #[cfg(feature = "highlight")]
        let code_theme = config.code_theme_value();

        // Warn if OpenAPI specs were registered but no nav group matches api_group_name
        if !openapi_specs.is_empty() && !nav.groups.iter().any(|g| g.group == api_group_name) {
            tracing::warn!(
                "dioxus-docs-kit: OpenAPI specs registered but no nav group \
                 matches api_group_name \"{api_group_name}\". API endpoints won't appear \
                 in the sidebar. Add a group with `\"group\": \"{api_group_name}\"` to \
                 _nav.json, or call .with_api_group_name(\"<your group name>\") on DocsConfig."
            );
        }

        // Build search index
        let search_index =
            Self::build_search_index(&nav, &parsed_docs, &openapi_specs, &api_group_name);

        let api_sidebar_entries = Self::build_api_sidebar_entries(&openapi_specs);

        let api_operation_index = openapi_specs
            .iter()
            .enumerate()
            .flat_map(|(spec_idx, (prefix, spec))| {
                spec.operations.iter().enumerate().map(move |(op_idx, op)| {
                    (format!("{prefix}/{}", op.slug()), (spec_idx, op_idx))
                })
            })
            .collect();

        Ok(Self {
            nav,
            parsed_docs,
            search_index,
            openapi_specs,
            api_sidebar_entries,
            api_operation_index,
            default_path,
            api_group_name,
            theme,
            #[cfg(feature = "highlight")]
            code_theme,
        })
    }

    /// Get a pre-parsed document by path.
    pub fn get_parsed_doc(&self, path: &str) -> Option<&ParsedDoc> {
        self.parsed_docs.get(path)
    }

    /// Get the sidebar title for a documentation path.
    pub fn get_sidebar_title(&self, path: &str) -> Option<String> {
        // Check if this is an API endpoint
        if let Some(op) = self.get_api_operation(path) {
            return op
                .summary
                .clone()
                .or_else(|| Some(op.slug().replace('-', " ")));
        }

        self.get_parsed_doc(path).and_then(|doc| {
            doc.frontmatter.sidebar_title.clone().or_else(|| {
                if doc.frontmatter.title.is_empty() {
                    None
                } else {
                    Some(doc.frontmatter.title.clone())
                }
            })
        })
    }

    /// Get the document title from frontmatter.
    pub fn get_doc_title(&self, path: &str) -> Option<String> {
        self.get_parsed_doc(path).and_then(|doc| {
            if doc.frontmatter.title.is_empty() {
                None
            } else {
                Some(doc.frontmatter.title.clone())
            }
        })
    }

    /// Get the page title for the document `<head>`.
    ///
    /// Resolves MDX frontmatter titles *and* OpenAPI operation pages (which are
    /// not in `parsed_docs`): for an endpoint path it returns the operation
    /// summary, falling back to a humanised slug. Branding/suffixes are left to
    /// the caller.
    pub fn get_page_title(&self, path: &str) -> Option<String> {
        if let Some(op) = self.get_api_operation(path) {
            return op
                .summary
                .clone()
                .or_else(|| Some(op.slug().replace('-', " ")));
        }
        self.get_doc_title(path)
    }

    /// Get the meta description for the document `<head>`.
    ///
    /// Resolves MDX frontmatter descriptions *and* OpenAPI operation
    /// descriptions, so endpoint pages ship a unique `<meta name="description">`
    /// instead of falling through to a generic default.
    pub fn get_page_description(&self, path: &str) -> Option<String> {
        if let Some(op) = self.get_api_operation(path) {
            return op.description.clone();
        }
        self.get_parsed_doc(path)
            .and_then(|doc| doc.frontmatter.description.clone())
    }

    /// Get the icon for a documentation path from frontmatter.
    pub fn get_doc_icon(&self, path: &str) -> Option<String> {
        self.get_parsed_doc(path)
            .and_then(|doc| doc.frontmatter.icon.clone())
    }

    /// Get raw documentation content by path.
    pub fn get_doc_content(&self, path: &str) -> Option<&str> {
        self.parsed_docs
            .get(path)
            .map(|doc| doc.raw_markdown.as_str())
    }

    /// Get all available documentation paths.
    pub fn get_all_paths(&self) -> Vec<&str> {
        self.parsed_docs.keys().copied().collect()
    }

    // ========================================================================
    // OpenAPI methods
    // ========================================================================

    /// Look up an API operation by its slug across all registered specs.
    ///
    /// The `path` is the full docs path, e.g. "api-reference/list-pets".
    pub fn get_api_operation(&self, path: &str) -> Option<&ApiOperation> {
        self.get_api_operation_with_spec(path).map(|(op, _)| op)
    }

    /// Look up an API operation together with the spec that owns it.
    ///
    /// Use this instead of pairing [`Self::get_api_operation`] with
    /// [`Self::get_first_api_spec`] — with multiple registered specs, the
    /// first spec is not necessarily the one the operation belongs to.
    pub fn get_api_operation_with_spec(&self, path: &str) -> Option<(&ApiOperation, &OpenApiSpec)> {
        let &(spec_idx, op_idx) = self.api_operation_index.get(path)?;
        let (_, spec) = &self.openapi_specs[spec_idx];
        Some((&spec.operations[op_idx], spec))
    }

    /// Get the OpenAPI spec that owns a given path prefix.
    pub fn get_api_spec(&self, prefix: &str) -> Option<&OpenApiSpec> {
        self.openapi_specs
            .iter()
            .find(|(p, _)| p == prefix)
            .map(|(_, spec)| spec)
    }

    /// Get the first OpenAPI spec (convenience for single-spec setups).
    pub fn get_first_api_spec(&self) -> Option<&OpenApiSpec> {
        self.openapi_specs.first().map(|(_, spec)| spec)
    }

    /// Get the prefix of the first OpenAPI spec.
    pub fn get_first_api_prefix(&self) -> Option<&str> {
        self.openapi_specs.first().map(|(p, _)| p.as_str())
    }

    /// Get API endpoint sidebar entries grouped by tag (precomputed).
    pub fn get_api_sidebar_entries(&self) -> &[(ApiTag, Vec<ApiEndpointEntry>)] {
        &self.api_sidebar_entries
    }

    /// Build API endpoint sidebar entries grouped by tag.
    fn build_api_sidebar_entries(
        openapi_specs: &[(String, OpenApiSpec)],
    ) -> Vec<(ApiTag, Vec<ApiEndpointEntry>)> {
        let mut all_groups: Vec<(ApiTag, Vec<ApiEndpointEntry>)> = Vec::new();

        let make_entry = |prefix: &str, op: &ApiOperation| ApiEndpointEntry {
            prefix: prefix.to_string(),
            slug: op.slug(),
            title: op
                .summary
                .clone()
                .unwrap_or_else(|| op.slug().replace('-', " ")),
            method: op.method,
        };

        for (prefix, spec) in openapi_specs {
            for tag in &spec.tags {
                let entries: Vec<ApiEndpointEntry> = spec
                    .operations
                    .iter()
                    .filter(|op| op.tags.contains(&tag.name))
                    .map(|op| make_entry(prefix, op))
                    .collect();

                if !entries.is_empty() {
                    all_groups.push((tag.clone(), entries));
                }
            }

            // Untagged operations
            let tagged_ids: Vec<_> = spec.tags.iter().map(|t| t.name.as_str()).collect();
            let untagged: Vec<ApiEndpointEntry> = spec
                .operations
                .iter()
                .filter(|op| {
                    op.tags.is_empty() || op.tags.iter().all(|t| !tagged_ids.contains(&t.as_str()))
                })
                .map(|op| make_entry(prefix, op))
                .collect();

            if !untagged.is_empty() {
                all_groups.push((
                    ApiTag {
                        name: "Other".to_string(),
                        description: None,
                    },
                    untagged,
                ));
            }
        }

        all_groups
    }

    /// Get all API endpoint paths for navigation ordering.
    pub fn get_api_endpoint_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        for (prefix, spec) in &self.openapi_specs {
            for op in &spec.operations {
                paths.push(format!("{prefix}/{}", op.slug()));
            }
        }
        paths
    }

    /// Determine which tab a given page path belongs to.
    pub fn tab_for_path(&self, path: &str) -> Option<String> {
        // Check static pages in nav groups
        for group in &self.nav.groups {
            if group.pages.iter().any(|p| p == path) {
                return group.tab.clone();
            }
        }

        // Check dynamic API endpoint pages
        for (prefix, _) in &self.openapi_specs {
            if path.starts_with(&format!("{prefix}/")) {
                for group in &self.nav.groups {
                    if group.group == self.api_group_name {
                        return group.tab.clone();
                    }
                }
            }
        }

        None
    }

    // ========================================================================
    // LLMs.txt
    // ========================================================================

    /// Generate an `llms.txt` index listing all doc pages with titles and descriptions.
    ///
    /// `docs_base_url` is the absolute URL of the docs root, e.g.
    /// `"https://example.com/docs"`; page URLs are formed as
    /// `<docs_base_url>/<page>`.
    pub fn generate_llms_txt(
        &self,
        site_title: &str,
        site_description: &str,
        docs_base_url: &str,
    ) -> String {
        let mut out = format!("# {site_title}\n\n> {site_description}\n\n");

        for group in &self.nav.groups {
            for page in &group.pages {
                if let Some(doc) = self.get_parsed_doc(page) {
                    let title = if doc.frontmatter.title.is_empty() {
                        page.split('/').next_back().unwrap_or(page).to_string()
                    } else {
                        doc.frontmatter.title.clone()
                    };
                    let desc = doc.frontmatter.description.as_deref().unwrap_or("");
                    let url = format!("{docs_base_url}/{page}");
                    if desc.is_empty() {
                        out.push_str(&format!("- [{title}]({url})\n"));
                    } else {
                        out.push_str(&format!("- [{title}]({url}): {desc}\n"));
                    }
                }
            }
        }

        out
    }

    /// Generate an `llms-full.txt` with the full MDX content of every doc page.
    ///
    /// See [`Self::generate_llms_txt`] for the `docs_base_url` format.
    pub fn generate_llms_full_txt(
        &self,
        site_title: &str,
        site_description: &str,
        docs_base_url: &str,
    ) -> String {
        let mut out = format!("# {site_title}\n\n> {site_description}\n\n");

        for group in &self.nav.groups {
            for page in &group.pages {
                if let Some(doc) = self.get_parsed_doc(page) {
                    let title = if doc.frontmatter.title.is_empty() {
                        page.split('/').next_back().unwrap_or(page).to_string()
                    } else {
                        doc.frontmatter.title.clone()
                    };
                    let url = format!("{docs_base_url}/{page}");
                    out.push_str(&format!("---\n\n## [{title}]({url})\n\n"));
                    out.push_str(&doc.raw_markdown);
                    out.push_str("\n\n");
                }
            }
        }

        out
    }

    // ========================================================================
    // Sitemap
    // ========================================================================

    /// Generate a sitemap.xml for all documentation pages.
    pub fn generate_sitemap(&self, site_url: &str, docs_path: &str) -> String {
        let mut xml = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
        );

        // Docs index
        xml.push_str(&format!(
            "<url>\n<loc>{site_url}{docs_path}</loc>\n<changefreq>weekly</changefreq>\n<priority>1.0</priority>\n</url>\n"
        ));

        // Individual doc pages
        for group in &self.nav.groups {
            for page in &group.pages {
                let loc = format!("{site_url}{docs_path}/{page}");
                xml.push_str(&format!(
                    "<url>\n<loc>{loc}</loc>\n<changefreq>weekly</changefreq>\n<priority>0.7</priority>\n</url>\n"
                ));
            }
        }

        // API endpoint pages
        for (prefix, spec) in &self.openapi_specs {
            for op in &spec.operations {
                let loc = format!("{site_url}{docs_path}/{prefix}/{}", op.slug());
                xml.push_str(&format!(
                    "<url>\n<loc>{loc}</loc>\n<changefreq>monthly</changefreq>\n<priority>0.5</priority>\n</url>\n"
                ));
            }
        }

        xml.push_str("</urlset>\n");
        xml
    }

    // ========================================================================
    // Search
    // ========================================================================

    /// Search documentation by query string.
    ///
    /// Splits the query on whitespace and returns section-level entries whose
    /// fields all match every term (multi-term AND), ranked title > heading >
    /// description > body with a word-boundary bonus. See [`crate::search`].
    pub fn search_docs(&self, query: &str) -> Vec<&SearchEntry> {
        crate::search::rank(&self.search_index, query, |e, buf| {
            buf.push(Field::title(&e.title_lower));
            if !e.heading_lower.is_empty() {
                buf.push(Field::heading(&e.heading_lower));
            }
            if !e.description_lower.is_empty() {
                buf.push(Field::description(&e.description_lower));
            }
            if !e.body_lower.is_empty() {
                buf.push(Field::body(&e.body_lower));
            }
        })
    }

    /// Build the search index from parsed docs and OpenAPI specs.
    ///
    /// Docs are indexed per section (split on h2–h4 headings); OpenAPI
    /// operations stay page-level.
    fn build_search_index(
        nav: &NavConfig,
        parsed_docs: &HashMap<&'static str, ParsedDoc>,
        openapi_specs: &[(String, OpenApiSpec)],
        api_group_name: &str,
    ) -> Vec<SearchEntry> {
        let mut entries = Vec::new();

        // Index documentation pages from nav config, one entry per section.
        for group in &nav.groups {
            for page in &group.pages {
                if let Some(doc) = parsed_docs.get(page.as_str()) {
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
                        // Drop an empty intro once real sections exist; keep it
                        // for a heading-less page so it stays findable by title.
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
                            group.group.clone(),
                            None,
                        ));
                    }
                }
            }
        }

        // Index API operations (page-level).
        for (prefix, spec) in openapi_specs {
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
                    format!("{api_group_name} > {tag}"),
                    Some(op.method),
                ));
            }
        }

        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DocsKitError;

    const NAV: &str = r#"{
        "tabs": ["Docs", "API Reference"],
        "groups": [
            { "group": "Search Fixtures", "tab": "Docs", "pages": ["g/body-doc", "g/desc-doc", "g/title-doc", "g/sections"] },
            { "group": "Getting Started", "tab": "Docs", "pages": ["getting-started/intro"] },
            { "group": "API Reference", "tab": "API Reference", "pages": ["api-reference/overview"] }
        ]
    }"#;

    const BODY_DOC: &str = "---\ntitle: Body doc\ndescription: nothing here\n---\n\nThe alpha keyword lives in the body.\n";
    const DESC_DOC: &str = "---\ntitle: Desc doc\ndescription: mentions alpha here\n---\n\nplain\n";
    const TITLE_DOC: &str = "---\ntitle: Alpha guide\ndescription: plain\n---\n\nplain\n";
    const INTRO: &str =
        "---\ntitle: Introduction\ndescription: Getting started guide\n---\n\nWelcome.\n";
    const OVERVIEW: &str = "---\ntitle: API Overview\n---\n\nEndpoints below.\n";
    // Multi-section fixture: intro text, then an h2 with a nested h3.
    const SECTIONS_DOC: &str = "---\ntitle: Widget Guide\ndescription: All about widgets\n---\n\nIntro paragraph about widgets.\n\n## Installation Steps\n\nRun the installer to set up widgets.\n\n### Advanced Setup\n\nConfigure the widget cache carefully.\n";

    const PETS_SPEC: &str = r#"
openapi: "3.0.0"
info:
  title: Pets API
  version: "1.0.0"
tags:
  - name: pets
    description: Pet operations
paths:
  /pets:
    get:
      operationId: listPets
      summary: List pets
      tags: [pets]
      responses:
        "200":
          description: OK
    post:
      operationId: createPet
      summary: Create pet
      tags: [pets]
      responses:
        "200":
          description: OK
  /misc:
    get:
      operationId: miscThing
      summary: Misc thing
      responses:
        "200":
          description: OK
"#;

    const ADMIN_SPEC: &str = r#"
openapi: "3.0.0"
info:
  title: Admin API
  version: "1.0.0"
paths:
  /admin/users:
    get:
      operationId: listAdminUsers
      summary: List admin users
      responses:
        "200":
          description: OK
"#;

    fn content_map() -> HashMap<&'static str, &'static str> {
        HashMap::from([
            ("g/body-doc", BODY_DOC),
            ("g/desc-doc", DESC_DOC),
            ("g/title-doc", TITLE_DOC),
            ("g/sections", SECTIONS_DOC),
            ("getting-started/intro", INTRO),
            ("api-reference/overview", OVERVIEW),
        ])
    }

    fn registry() -> DocsRegistry {
        DocsConfig::new(NAV, content_map())
            .with_openapi("api-reference", PETS_SPEC)
            .with_openapi("admin-api", ADMIN_SPEC)
            .build()
    }

    #[test]
    fn try_build_reports_nav_parse_error_with_detail() {
        let Err(err) = DocsConfig::new("{ not json", HashMap::new()).try_build() else {
            panic!("expected nav parse error");
        };
        assert!(matches!(err, DocsKitError::NavParse(_)));
        assert!(err.to_string().contains("_nav.json"));
    }

    #[test]
    fn try_build_reports_openapi_error_with_prefix() {
        let Err(err) = DocsConfig::new(NAV, content_map())
            .with_openapi("api-reference", "openapi: true")
            .try_build()
        else {
            panic!("expected OpenAPI parse error");
        };
        match &err {
            DocsKitError::OpenApi { prefix, .. } => assert_eq!(prefix, "api-reference"),
            other => panic!("expected OpenApi error, got {other:?}"),
        }
        assert!(err.to_string().contains("api-reference"));
    }

    #[test]
    fn search_ranks_title_before_description_before_content() {
        let reg = registry();
        let results = reg.search_docs("alpha");
        let paths: Vec<&str> = results.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(paths, vec!["g/title-doc", "g/desc-doc", "g/body-doc"]);
    }

    #[test]
    fn search_empty_query_returns_nothing() {
        assert!(registry().search_docs("  ").is_empty());
    }

    #[test]
    fn sections_split_on_headings_with_intro_and_slugified_anchors() {
        let sections = split_into_sections(
            "Intro text.\n\n## Installation Steps\n\nRun it.\n\n### Advanced Setup\n\nTweak it.\n",
        );
        assert_eq!(sections.len(), 3);

        // Intro section carries no heading/anchor.
        assert_eq!(sections[0].heading, "");
        assert_eq!(sections[0].anchor, "");
        assert!(sections[0].body.contains("Intro text."));

        // Heading sections carry slugify anchors that match the renderer's ids.
        assert_eq!(sections[1].heading, "Installation Steps");
        assert_eq!(sections[1].anchor, slugify("Installation Steps"));
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
        // A page whose body starts straight with a heading has no intro text.
        let sections = split_into_sections("## First\n\nbody\n");
        assert_eq!(sections[0].heading, "");
        assert!(sections[0].body.trim().is_empty());
    }

    #[test]
    fn search_returns_section_anchor_for_heading_match() {
        let reg = registry();
        let hit = reg
            .search_docs("installation")
            .into_iter()
            .find(|e| e.path == "g/sections")
            .expect("installation heading section");
        assert_eq!(hit.heading, "Installation Steps");
        assert_eq!(hit.anchor, "installation-steps");
        assert!(hit.api_method.is_none());
    }

    #[test]
    fn search_multi_term_requires_all_terms_across_the_page() {
        let reg = registry();
        // "cache" only appears in the "Advanced Setup" section body; "widget"
        // is everywhere on the page. AND matching narrows to that section.
        let results = reg.search_docs("widget cache");
        assert!(
            results.iter().all(|e| e.heading == "Advanced Setup"),
            "expected only the Advanced Setup section, got: {:?}",
            results
                .iter()
                .map(|e| e.heading.as_str())
                .collect::<Vec<_>>()
        );
        assert!(!results.is_empty());
    }

    #[test]
    fn search_index_precomputes_lowercase_fields() {
        let reg = registry();
        let entry = reg
            .search_index
            .iter()
            .find(|e| e.title == "Widget Guide")
            .expect("sectioned doc entry");
        assert_eq!(entry.title_lower, "widget guide");
        assert_eq!(entry.description_lower, "all about widgets");
        assert!(entry.heading_lower == entry.heading.to_lowercase());
        assert!(entry.body_lower.chars().all(|c| !c.is_uppercase()));
    }

    #[test]
    fn api_sidebar_entries_group_by_tag_with_prefix() {
        let reg = registry();
        let groups = reg.get_api_sidebar_entries();
        assert_eq!(groups.len(), 3);

        let (pets_tag, pets_entries) = &groups[0];
        assert_eq!(pets_tag.name, "pets");
        let slugs: Vec<&str> = pets_entries.iter().map(|e| e.slug.as_str()).collect();
        assert_eq!(slugs, vec!["list-pets", "create-pet"]);
        assert!(pets_entries.iter().all(|e| e.prefix == "api-reference"));

        let (other_tag, other_entries) = &groups[1];
        assert_eq!(other_tag.name, "Other");
        assert_eq!(other_entries[0].slug, "misc-thing");
        assert_eq!(other_entries[0].prefix, "api-reference");

        // The second spec's untagged operation carries its own prefix.
        let (admin_tag, admin_entries) = &groups[2];
        assert_eq!(admin_tag.name, "Other");
        assert_eq!(admin_entries[0].slug, "list-admin-users");
        assert_eq!(admin_entries[0].prefix, "admin-api");
    }

    #[test]
    fn operation_lookup_resolves_owning_spec() {
        let reg = registry();

        let (op, spec) = reg
            .get_api_operation_with_spec("api-reference/list-pets")
            .unwrap();
        assert_eq!(op.summary.as_deref(), Some("List pets"));
        assert_eq!(spec.info.title, "Pets API");

        // Multi-spec regression: the second spec's operations must resolve to
        // the second spec, not the first.
        let (op, spec) = reg
            .get_api_operation_with_spec("admin-api/list-admin-users")
            .unwrap();
        assert_eq!(op.summary.as_deref(), Some("List admin users"));
        assert_eq!(spec.info.title, "Admin API");

        assert!(
            reg.get_api_operation_with_spec("api-reference/nope")
                .is_none()
        );
        assert!(
            reg.get_api_operation_with_spec("unknown/list-pets")
                .is_none()
        );
    }

    #[test]
    fn raw_doc_content_present_for_mdx_absent_for_api() {
        let reg = registry();
        // MDX pages expose their raw Markdown source (drives the copy-page button).
        assert!(
            reg.get_doc_content("getting-started/intro")
                .unwrap()
                .contains("Welcome.")
        );
        // OpenAPI endpoint paths have no Markdown source, so no button renders.
        assert!(reg.get_doc_content("api-reference/list-pets").is_none());
        assert!(reg.get_doc_content("admin-api/list-admin-users").is_none());
    }

    #[test]
    fn tab_for_path_covers_static_and_api_pages() {
        let reg = registry();
        assert_eq!(
            reg.tab_for_path("getting-started/intro").as_deref(),
            Some("Docs")
        );
        assert_eq!(
            reg.tab_for_path("api-reference/list-pets").as_deref(),
            Some("API Reference")
        );
        assert_eq!(
            reg.tab_for_path("admin-api/list-admin-users").as_deref(),
            Some("API Reference")
        );
        assert_eq!(reg.tab_for_path("nope/nothing"), None);
    }

    #[test]
    fn llms_txt_lists_pages_under_docs_base_url() {
        let out = registry().generate_llms_txt("My Site", "My docs", "https://example.com/docs");
        assert!(out.starts_with("# My Site\n\n> My docs\n"));
        assert!(out.contains(
            "- [Introduction](https://example.com/docs/getting-started/intro): Getting started guide\n"
        ));
    }

    #[test]
    fn sitemap_includes_index_pages_and_api_endpoints() {
        let out = registry().generate_sitemap("https://example.com", "/docs");
        assert!(out.contains("<loc>https://example.com/docs</loc>"));
        assert!(out.contains("<loc>https://example.com/docs/getting-started/intro</loc>"));
        assert!(out.contains("<loc>https://example.com/docs/api-reference/list-pets</loc>"));
        assert!(out.contains("<loc>https://example.com/docs/admin-api/list-admin-users</loc>"));
    }

    #[test]
    fn default_path_falls_back_to_first_nav_page() {
        assert_eq!(registry().default_path, "g/body-doc");
    }

    #[test]
    fn sidebar_title_resolves_api_summaries_and_frontmatter() {
        let reg = registry();
        assert_eq!(
            reg.get_sidebar_title("api-reference/list-pets").as_deref(),
            Some("List pets")
        );
        assert_eq!(
            reg.get_sidebar_title("getting-started/intro").as_deref(),
            Some("Introduction")
        );
    }
}
