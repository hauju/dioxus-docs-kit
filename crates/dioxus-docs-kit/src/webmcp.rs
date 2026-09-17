//! WebMCP tools: hand the page's docs to whatever agent is driving the browser.
//!
//! An in-browser agent that can call [`docs_search`](DocsWebMcp) reads the
//! search index that is already compiled into the wasm instead of scraping the
//! rendered DOM, and gets a page's Markdown source instead of its markup.
//!
//! Mount [`DocsWebMcp`] anywhere below [`use_docs_providers`]:
//!
//! ```rust,ignore
//! rsx! {
//!     DocsLayout {
//!         header: rsx! { /* ... */ },
//!         DocsWebMcp {}
//!         Outlet::<Route> {}
//!     }
//! }
//! ```
//!
//! The registrations are tied to the component's lifetime, so leaving the docs
//! section unregisters them — an agent is never offered a tool that reads a
//! registry the user has navigated away from.
//!
//! Off wasm (SSR, tests) registration is a silent no-op, and in a browser
//! without WebMCP nothing happens either. Chrome 153+ has it natively (149 to
//! 152 behind the origin trial or `chrome://flags/#enable-webmcp-testing`); for
//! everyone else, load Google's WebMCP polyfill before the wasm starts: a plain
//! `<script src>` in `index.html` pointing at an unhashed asset (the example
//! app does exactly that, see its `index.html` and the `WEBMCP_POLYFILL` asset
//! in `src/main.rs`).
//! A `document::Script` from a component is too late; the tools register on
//! first render.

use dioxus::prelude::*;
use dioxus_mdx::{
    ApiOperation, ApiParameter, ApiResponse, HttpMethod, MediaTypeContent, OpenApiSpec,
    SchemaDefinition,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use webmcp::{Tool, ToolResult, use_tool};

use crate::DocsContext;
use crate::registry::DocsRegistry;

/// Longest snippet returned per search hit, in characters.
const SNIPPET_CHARS: usize = 220;
/// Default and maximum number of search hits returned.
const DEFAULT_LIMIT: usize = 10;
const MAX_LIMIT: usize = 50;

// ============================================================================
// Component
// ============================================================================

/// Registers this site's docs as WebMCP tools for the duration of the mount.
///
/// Requires a [`DocsRegistry`] and [`DocsContext`] in context, which
/// [`use_docs_providers`](crate::use_docs_providers) supplies. Renders nothing.
///
/// The tools are read-only: `docs_search`, `docs_get_page`, `docs_list_pages`
/// and `docs_get_api_operation`. None of them can change what the user is
/// looking at. The `docs_` prefix keeps them clear of whatever else the host
/// page registers: WebMCP rejects a second tool with an existing name.
#[component]
pub fn DocsWebMcp() -> Element {
    let registry = use_context::<&'static DocsRegistry>();
    let ctx = use_context::<DocsContext>();

    // Handlers run on a browser callback, outside the Dioxus runtime, so they
    // capture plain owned data — never a Signal. Everything they read is either
    // `'static` (the registry is a `&'static` the consumer built at startup) or
    // a clone of the context's URL configuration, which cannot change for the
    // lifetime of the layout.
    let urls = UrlBase {
        base_path: ctx.base_path.trim_end_matches('/').to_string(),
        site_url: ctx
            .site_url
            .as_deref()
            .map(|u| u.trim_end_matches('/').to_string()),
    };

    let search_urls = urls.clone();
    use_tool(move || {
        let urls = search_urls.clone();
        Tool::new("docs_search")
            .description(
                "Search this documentation site. Returns ranked section-level matches \
                 with a text snippet and the URL of each. Prefer this over reading the \
                 page to find where something is documented.",
            )
            .register_typed(move |args: SearchArgs| {
                let urls = urls.clone();
                async move {
                    let limit = args.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
                    let hits: Vec<SearchHit> = registry
                        .search_docs(&args.query)
                        .into_iter()
                        .take(limit)
                        .map(|e| SearchHit {
                            url: urls.url(&e.path, &e.anchor),
                            path: e.path.clone(),
                            title: e.title.clone(),
                            heading: e.heading.clone(),
                            anchor: e.anchor.clone(),
                            breadcrumb: e.breadcrumb.clone(),
                            method: e.api_method.as_ref().map(HttpMethod::as_str),
                            snippet: snippet(if e.body.is_empty() {
                                &e.description
                            } else {
                                &e.body
                            }),
                        })
                        .collect();
                    Ok::<_, std::convert::Infallible>(ToolResult::json(&SearchOutput {
                        query: args.query,
                        count: hits.len(),
                        results: hits,
                    }))
                }
            })
    });

    let doc_urls = urls.clone();
    use_tool(move || {
        let urls = doc_urls.clone();
        Tool::new("docs_get_page")
            .description(
                "Read one documentation page as its original Markdown source, which is \
                 shorter and cleaner than the rendered page. Use docs_search or \
                 docs_list_pages to find the path.",
            )
            .register_typed(move |args: PathArgs| {
                let urls = urls.clone();
                async move {
                    let path = urls.normalize(&args.path);
                    let Some(markdown) = registry.get_doc_content(&path) else {
                        return Ok::<_, std::convert::Infallible>(not_found(&path, registry));
                    };
                    Ok(ToolResult::json(&DocOutput {
                        url: urls.url(&path, ""),
                        title: registry.get_page_title(&path),
                        description: registry.get_page_description(&path),
                        tab: registry.tab_for_path(&path),
                        path,
                        markdown: markdown.to_string(),
                    }))
                }
            })
    });

    let list_urls = urls.clone();
    use_tool(move || {
        let urls = list_urls.clone();
        Tool::new("docs_list_pages")
            .description(
                "List every page on this documentation site, grouped the way the sidebar \
                 groups them. Use it to find the exact path for docs_get_page, or to see \
                 what \
                 the site covers.",
            )
            .register_typed(move |_: NoArgs| {
                let urls = urls.clone();
                async move {
                    Ok::<_, std::convert::Infallible>(ToolResult::json(&list_docs(registry, &urls)))
                }
            })
    });

    let api_urls = urls.clone();
    use_tool(move || {
        let urls = api_urls.clone();
        Tool::new("docs_get_api_operation")
            .description(
                "Get one API reference endpoint as structured data: HTTP method, URL, \
                 parameters with their types, request body, response codes and a ready \
                 curl command. Use this instead of reading the rendered endpoint page.",
            )
            .register_typed(move |args: PathArgs| {
                let urls = urls.clone();
                async move {
                    let path = urls.normalize(&args.path);
                    let Some((op, spec)) = registry.get_api_operation_with_spec(&path) else {
                        return Ok::<_, std::convert::Infallible>(ToolResult::error(format!(
                            "no API operation at \"{path}\". Call docs_list_pages for the endpoints \
                             this site documents."
                        )));
                    };
                    Ok(ToolResult::json(&operation_output(
                        op,
                        spec,
                        &urls.url(&path, ""),
                    )))
                }
            })
    });

    rsx! {}
}

// ============================================================================
// Tool arguments
// ============================================================================

#[derive(Deserialize, JsonSchema)]
struct SearchArgs {
    /// Words to search for. All terms must match; title and heading matches
    /// rank above body matches.
    query: String,
    /// Maximum number of results to return. Defaults to 10, capped at 50.
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
struct PathArgs {
    /// Content path of the page, e.g. "getting-started/introduction". A full
    /// URL, a leading slash, a ".md" suffix or a "#section" anchor are all
    /// accepted and stripped.
    path: String,
}

#[derive(Deserialize, JsonSchema)]
struct NoArgs {}

// ============================================================================
// Tool output
// ============================================================================

#[derive(Serialize)]
struct SearchOutput {
    query: String,
    count: usize,
    results: Vec<SearchHit>,
}

#[derive(Serialize)]
struct SearchHit {
    path: String,
    url: String,
    title: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    heading: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    anchor: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    breadcrumb: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<&'static str>,
    snippet: String,
}

#[derive(Serialize)]
struct DocOutput {
    path: String,
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tab: Option<String>,
    markdown: String,
}

#[derive(Serialize)]
struct DocsIndex {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tabs: Vec<String>,
    default_path: String,
    groups: Vec<GroupOutput>,
}

#[derive(Serialize)]
struct GroupOutput {
    group: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tab: Option<String>,
    pages: Vec<PageOutput>,
}

#[derive(Serialize)]
struct PageOutput {
    path: String,
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<&'static str>,
}

#[derive(Serialize)]
struct OperationOutput {
    method: &'static str,
    endpoint: String,
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    operation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(skip_serializing_if = "is_false")]
    deprecated: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    parameters: Vec<ParameterOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_body: Option<BodyOutput>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    responses: Vec<ResponseOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    curl: Option<String>,
}

#[derive(Serialize)]
struct ParameterOutput {
    name: String,
    #[serde(rename = "in")]
    location: &'static str,
    required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    example: Option<String>,
    #[serde(skip_serializing_if = "is_false")]
    deprecated: bool,
}

#[derive(Serialize)]
struct BodyOutput {
    required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    content: Vec<ContentOutput>,
}

#[derive(Serialize)]
struct ContentOutput {
    media_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    example: Option<String>,
}

#[derive(Serialize)]
struct ResponseOutput {
    status: String,
    description: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    content: Vec<ContentOutput>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

// ============================================================================
// Shaping
// ============================================================================

/// The URL configuration the handlers need, cloned out of [`DocsContext`] so
/// nothing reactive is captured.
#[derive(Clone)]
struct UrlBase {
    base_path: String,
    site_url: Option<String>,
}

impl UrlBase {
    /// Absolute URL when the site origin is configured, root-relative otherwise.
    fn url(&self, path: &str, anchor: &str) -> String {
        let origin = self.site_url.as_deref().unwrap_or("");
        let fragment = if anchor.is_empty() {
            String::new()
        } else {
            format!("#{anchor}")
        };
        format!("{origin}{}/{path}{fragment}", self.base_path)
    }

    /// Reduce whatever the agent passed to a content path.
    ///
    /// Agents reliably hand back the URL they were given rather than the
    /// `path` beside it, so accept a full URL, a leading slash, the docs base
    /// prefix, a `.md`/`.mdx` extension and a `#anchor`.
    fn normalize(&self, raw: &str) -> String {
        let mut path = raw.trim();
        if let Some(rest) = path.split_once("://").map(|(_, rest)| rest) {
            path = rest.split_once('/').map_or("", |(_, rest)| rest);
        }
        path = path.split('#').next().unwrap_or(path);
        path = path.split('?').next().unwrap_or(path);
        path = path.trim_matches('/');
        let base = self.base_path.trim_matches('/');
        if !base.is_empty() {
            path = path.strip_prefix(base).unwrap_or(path).trim_matches('/');
        }
        path = path
            .strip_suffix(".mdx")
            .or_else(|| path.strip_suffix(".md"))
            .unwrap_or(path);
        path.to_string()
    }
}

/// Truncate on a character boundary so multi-byte content cannot panic.
fn snippet(text: &str) -> String {
    let mut out: String = text.chars().take(SNIPPET_CHARS).collect();
    if out.chars().count() < text.chars().count() {
        out.push('…');
    }
    out
}

/// A miss is a chance to correct the agent, so name the closest paths we have
/// rather than only reporting the failure.
///
/// The realistic miss is a wrong slug under a real group ("guides/theming-x"),
/// so suggest that group's pages; a wrong group falls back to substring matches
/// on the slug.
fn not_found(path: &str, registry: &'static DocsRegistry) -> ToolResult {
    let (group, slug) = path.rsplit_once('/').unwrap_or(("", path));
    let mut near: Vec<&str> = registry
        .get_all_paths()
        .into_iter()
        .filter(|p| {
            if !group.is_empty() && p.starts_with(&format!("{group}/")) {
                return true;
            }
            !slug.is_empty() && p.rsplit('/').next().is_some_and(|s| s.contains(slug))
        })
        .collect();
    near.sort_unstable();
    near.truncate(5);
    if near.is_empty() {
        ToolResult::error(format!(
            "no page at \"{path}\". Call docs_list_pages for every path on this site."
        ))
    } else {
        ToolResult::error(format!(
            "no page at \"{path}\". Did you mean one of: {}?",
            near.join(", ")
        ))
    }
}

fn list_docs(registry: &'static DocsRegistry, urls: &UrlBase) -> DocsIndex {
    // API endpoint paths are generated from the OpenAPI spec, never listed in
    // `_nav.json`, so the API group's `pages` is empty and has to be filled in
    // from the sidebar entries the registry precomputed.
    let api_pages = || -> Vec<PageOutput> {
        registry
            .get_api_sidebar_entries()
            .iter()
            .flat_map(|(_, entries)| entries)
            .map(|e| {
                let path = format!("{}/{}", e.prefix, e.slug);
                PageOutput {
                    url: urls.url(&path, ""),
                    path,
                    title: Some(e.title.clone()),
                    method: Some(e.method.as_str()),
                }
            })
            .collect()
    };

    let groups = registry
        .nav
        .groups
        .iter()
        .map(|group| {
            let pages = if group.group == registry.api_group_name {
                api_pages()
            } else {
                group
                    .pages
                    .iter()
                    .map(|path| PageOutput {
                        url: urls.url(path, ""),
                        path: path.clone(),
                        title: registry.get_page_title(path),
                        method: None,
                    })
                    .collect()
            };
            GroupOutput {
                group: group.group.clone(),
                tab: group.tab.clone(),
                pages,
            }
        })
        .filter(|g| !g.pages.is_empty())
        .collect();

    DocsIndex {
        tabs: registry.nav.tabs.clone(),
        default_path: registry.default_path.clone(),
        groups,
    }
}

fn operation_output(op: &ApiOperation, spec: &OpenApiSpec, url: &str) -> OperationOutput {
    let server = spec.servers.first().map(|s| s.url.as_str());
    OperationOutput {
        method: op.method.as_str(),
        endpoint: op.path.clone(),
        url: url.to_string(),
        operation_id: op.operation_id.clone(),
        summary: op.summary.clone(),
        description: op.description.clone(),
        tags: op.tags.clone(),
        deprecated: op.deprecated,
        parameters: op.parameters.iter().map(parameter_output).collect(),
        request_body: op.request_body.as_ref().map(|body| BodyOutput {
            required: body.required,
            description: body.description.clone(),
            content: body.content.iter().map(content_output).collect(),
        }),
        responses: op.responses.iter().map(response_output).collect(),
        curl: server.map(|base| op.generate_curl(base)),
    }
}

fn parameter_output(param: &ApiParameter) -> ParameterOutput {
    ParameterOutput {
        name: param.name.clone(),
        location: param.location.as_str(),
        required: param.required,
        r#type: param.schema.as_ref().map(type_name),
        description: param.description.clone(),
        example: param.example.clone(),
        deprecated: param.deprecated,
    }
}

fn content_output(content: &MediaTypeContent) -> ContentOutput {
    ContentOutput {
        media_type: content.media_type.clone(),
        r#type: content.schema.as_ref().map(type_name),
        example: content.example.clone(),
    }
}

fn response_output(response: &ApiResponse) -> ResponseOutput {
    ResponseOutput {
        status: response.status_code.clone(),
        description: response.description.clone(),
        content: response.content.iter().map(content_output).collect(),
    }
}

/// One-line type for a schema: enough for an agent to build a valid call, far
/// smaller than the full schema tree. The rendered page has the whole thing.
fn type_name(schema: &SchemaDefinition) -> String {
    if let Some(name) = &schema.ref_name {
        return name.clone();
    }
    if !schema.enum_values.is_empty() {
        return schema.enum_values.join(" | ");
    }
    if let Some(items) = &schema.items {
        return format!("array<{}>", type_name(items));
    }
    match &schema.format {
        Some(format) => format!("{}<{format}>", schema.schema_type.as_str()),
        None => schema.schema_type.as_str().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus_mdx::SchemaType;

    fn urls() -> UrlBase {
        UrlBase {
            base_path: "/docs".to_string(),
            site_url: Some("https://example.com".to_string()),
        }
    }

    fn schema(schema_type: SchemaType) -> SchemaDefinition {
        SchemaDefinition {
            schema_type,
            ..Default::default()
        }
    }

    #[test]
    fn urls_include_the_origin_and_anchor() {
        assert_eq!(
            urls().url("guides/theming", "colors"),
            "https://example.com/docs/guides/theming#colors"
        );
        assert_eq!(
            urls().url("guides/theming", ""),
            "https://example.com/docs/guides/theming"
        );
    }

    #[test]
    fn urls_stay_relative_without_a_site_url() {
        let urls = UrlBase {
            base_path: "/docs".to_string(),
            site_url: None,
        };
        assert_eq!(urls.url("guides/theming", ""), "/docs/guides/theming");
    }

    #[test]
    fn normalize_accepts_every_shape_an_agent_hands_back() {
        let urls = urls();
        for input in [
            "guides/theming",
            "/guides/theming",
            "/docs/guides/theming",
            "docs/guides/theming",
            "guides/theming.md",
            "guides/theming.mdx",
            "guides/theming#colors",
            "guides/theming?utm_source=x",
            "  /docs/guides/theming/  ",
            "https://example.com/docs/guides/theming#colors",
        ] {
            assert_eq!(urls.normalize(input), "guides/theming", "input: {input}");
        }
    }

    #[test]
    fn snippet_truncates_on_a_character_boundary() {
        let long = "ä".repeat(SNIPPET_CHARS + 10);
        let out = snippet(&long);
        assert_eq!(out.chars().count(), SNIPPET_CHARS + 1);
        assert!(out.ends_with('…'));
        assert_eq!(snippet("short"), "short");
    }

    #[test]
    fn type_name_prefers_the_reference_then_the_enum_then_the_shape() {
        let mut named = schema(SchemaType::Object);
        named.ref_name = Some("Pet".to_string());
        assert_eq!(type_name(&named), "Pet");

        let mut list = schema(SchemaType::Array);
        list.items = Some(Box::new(named));
        assert_eq!(type_name(&list), "array<Pet>");

        let mut status = schema(SchemaType::String);
        status.enum_values = vec!["available".to_string(), "sold".to_string()];
        assert_eq!(type_name(&status), "available | sold");

        let mut count = schema(SchemaType::Integer);
        count.format = Some("int32".to_string());
        assert_eq!(type_name(&count), "integer<int32>");

        assert_eq!(type_name(&schema(SchemaType::Boolean)), "boolean");
    }
}
