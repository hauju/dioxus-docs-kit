//! Documentation content registry.
//!
//! Holds parsed docs, nav config, search index, and OpenAPI specs.

use crate::config::{DocsConfig, ThemeConfig};
use dioxus_mdx::{
    ApiOperation, ApiTag, HttpMethod, OpenApiSpec, ParsedDoc, parse_document, parse_openapi,
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
    /// URL slug (e.g. "list-pets").
    pub slug: String,
    /// Display title (summary or fallback).
    pub title: String,
    /// HTTP method.
    pub method: HttpMethod,
}

/// A searchable entry in the documentation.
#[derive(PartialEq)]
pub struct SearchEntry {
    pub path: String,
    pub title: String,
    pub description: String,
    pub content_preview: String,
    pub breadcrumb: String,
    pub api_method: Option<HttpMethod>,
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
    /// Default page path for redirects.
    pub default_path: String,
    /// Display name for the API Reference sidebar group.
    pub api_group_name: String,
    /// Optional theme configuration.
    pub theme: Option<ThemeConfig>,
}

impl DocsRegistry {
    /// Build a registry from a [`DocsConfig`].
    pub(crate) fn from_config(config: DocsConfig) -> Self {
        let nav: NavConfig =
            serde_json::from_str(config.nav_json()).expect("Failed to parse _nav.json");

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
                let spec = parse_openapi(yaml)
                    .expect(&format!("Failed to parse OpenAPI spec for {prefix}"));
                (prefix.clone(), spec)
            })
            .collect();

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

        // Build search index
        let search_index =
            Self::build_search_index(&nav, &parsed_docs, &openapi_specs, &api_group_name);

        Self {
            nav,
            parsed_docs,
            search_index,
            openapi_specs,
            default_path,
            api_group_name,
            theme,
        }
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
        for (prefix, spec) in &self.openapi_specs {
            if let Some(slug) = path.strip_prefix(&format!("{prefix}/")) {
                if let Some(op) = spec.operations.iter().find(|op| op.slug() == slug) {
                    return Some(op);
                }
            }
        }
        None
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

    /// Get API endpoint sidebar entries grouped by tag.
    pub fn get_api_sidebar_entries(&self) -> Vec<(ApiTag, Vec<ApiEndpointEntry>)> {
        let mut all_groups: Vec<(ApiTag, Vec<ApiEndpointEntry>)> = Vec::new();

        for (_prefix, spec) in &self.openapi_specs {
            for tag in &spec.tags {
                let entries: Vec<ApiEndpointEntry> = spec
                    .operations
                    .iter()
                    .filter(|op| op.tags.contains(&tag.name))
                    .map(|op| ApiEndpointEntry {
                        slug: op.slug(),
                        title: op
                            .summary
                            .clone()
                            .unwrap_or_else(|| op.slug().replace('-', " ")),
                        method: op.method,
                    })
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
                .map(|op| ApiEndpointEntry {
                    slug: op.slug(),
                    title: op
                        .summary
                        .clone()
                        .unwrap_or_else(|| op.slug().replace('-', " ")),
                    method: op.method,
                })
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
    pub fn generate_llms_txt(
        &self,
        site_title: &str,
        site_description: &str,
        base_url: &str,
    ) -> String {
        let mut out = format!("# {site_title}\n\n> {site_description}\n\n");

        for group in &self.nav.groups {
            for page in &group.pages {
                if let Some(doc) = self.get_parsed_doc(page) {
                    let title = if doc.frontmatter.title.is_empty() {
                        page.split('/').last().unwrap_or(page).to_string()
                    } else {
                        doc.frontmatter.title.clone()
                    };
                    let desc = doc.frontmatter.description.as_deref().unwrap_or("");
                    let url = format!("{base_url}/docs/{page}");
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
    pub fn generate_llms_full_txt(
        &self,
        site_title: &str,
        site_description: &str,
        base_url: &str,
    ) -> String {
        let mut out = format!("# {site_title}\n\n> {site_description}\n\n");

        for group in &self.nav.groups {
            for page in &group.pages {
                if let Some(doc) = self.get_parsed_doc(page) {
                    let title = if doc.frontmatter.title.is_empty() {
                        page.split('/').last().unwrap_or(page).to_string()
                    } else {
                        doc.frontmatter.title.clone()
                    };
                    let url = format!("{base_url}/docs/{page}");
                    out.push_str(&format!("---\n\n## [{title}]({url})\n\n"));
                    out.push_str(&doc.raw_markdown);
                    out.push_str("\n\n");
                }
            }
        }

        out
    }

    // ========================================================================
    // Search
    // ========================================================================

    /// Search documentation by query string.
    ///
    /// Returns matching entries with title matches first, then description, then content.
    pub fn search_docs(&self, query: &str) -> Vec<&SearchEntry> {
        let query = query.trim();
        if query.is_empty() {
            return Vec::new();
        }
        let q = query.to_lowercase();

        let mut title_matches: Vec<&SearchEntry> = Vec::new();
        let mut desc_matches: Vec<&SearchEntry> = Vec::new();
        let mut content_matches: Vec<&SearchEntry> = Vec::new();

        for entry in &self.search_index {
            if entry.title.to_lowercase().contains(&q) {
                title_matches.push(entry);
            } else if entry.description.to_lowercase().contains(&q) {
                desc_matches.push(entry);
            } else if entry.content_preview.to_lowercase().contains(&q) {
                content_matches.push(entry);
            }
        }

        title_matches.extend(desc_matches);
        title_matches.extend(content_matches);
        title_matches
    }

    /// Build the search index from parsed docs and OpenAPI specs.
    fn build_search_index(
        nav: &NavConfig,
        parsed_docs: &HashMap<&'static str, ParsedDoc>,
        openapi_specs: &[(String, OpenApiSpec)],
        _api_group_name: &str,
    ) -> Vec<SearchEntry> {
        let mut entries = Vec::new();

        // Index documentation pages from nav config
        for group in &nav.groups {
            for page in &group.pages {
                if let Some(doc) = parsed_docs.get(page.as_str()) {
                    let title = if doc.frontmatter.title.is_empty() {
                        page.split('/').last().unwrap_or(page).replace('-', " ")
                    } else {
                        doc.frontmatter.title.clone()
                    };
                    let description = doc.frontmatter.description.clone().unwrap_or_default();
                    let preview: String = doc.raw_markdown.chars().take(200).collect();

                    entries.push(SearchEntry {
                        path: page.clone(),
                        title,
                        description,
                        content_preview: preview,
                        breadcrumb: group.group.clone(),
                        api_method: None,
                    });
                }
            }
        }

        // Index API operations
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

                entries.push(SearchEntry {
                    path: format!("{prefix}/{}", op.slug()),
                    title,
                    description: description.clone(),
                    content_preview: description,
                    breadcrumb: format!("API Reference > {tag}"),
                    api_method: Some(op.method),
                });
            }
        }

        entries
    }
}
