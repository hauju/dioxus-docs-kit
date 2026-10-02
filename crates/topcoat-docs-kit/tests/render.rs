//! Renders real build-time bundles through the Topcoat components.

use docs_kit_core::{DocsConfig, DocsRegistry};
use topcoat::{
    context::Cx,
    view::{View, ViewExt, view},
};
use topcoat_docs_kit::{doc_content, docs_head, docs_page, docs_search_head, docs_search_page};

const NAV: &str = r#"{
    "tabs": ["Docs", "More", "API Reference"],
    "groups": [
        { "group": "Guide", "tab": "Docs", "pages": ["guide/intro", "guide/next"] },
        { "group": "Extra", "tab": "More", "pages": ["extra/page"] },
        { "group": "API Reference", "tab": "API Reference", "pages": ["api/overview"] }
    ]
}"#;

const OVERVIEW: &str = "---\ntitle: API Overview\n---\n\nEndpoints below.\n";

const SPEC: &str = r#"
openapi: "3.0.0"
info: { title: Pets, version: "1" }
servers: [{ url: "https://pets.example.com" }]
tags: [{ name: pets }]
paths:
  /pets/{id}:
    delete:
      operationId: deletePet
      summary: Delete a pet
      tags: [pets]
      parameters:
        - { name: id, in: path, required: true, schema: { type: string } }
      responses:
        "200":
          description: Deleted
          content:
            application/json:
              schema:
                type: object
                properties:
                  ok: { type: boolean }
"#;

const NEXT: &str = "---\ntitle: Next page\n---\n\n## Section one\n\nText about widgets.\n";
const EXTRA: &str = "---\ntitle: Extra\n---\n\nText.\n";

const INTRO: &str = r#"---
title: Intro
description: The first page
---

Some **prose** & more.

<Tip>Use the kit.</Tip>

```rust
fn main() {}
```

<Tabs>
  <Tab title="One">First tab</Tab>
  <Tab title="Two">Second tab</Tab>
</Tabs>

<Steps>
  <Step title="Step 1: Install">Run it.</Step>
</Steps>

<AccordionGroup>
  <Accordion title="More">Hidden text</Accordion>
</AccordionGroup>
"#;

fn registry() -> &'static DocsRegistry {
    let bundle = dioxus_docs_kit_build::docs_bundle_json(
        NAV,
        &[
            ("guide/intro", INTRO),
            ("guide/next", NEXT),
            ("extra/page", EXTRA),
            ("api/overview", OVERVIEW),
        ],
        &[("api", SPEC)],
    )
    .expect("generate bundle")
    .leak();
    Box::leak(Box::new(DocsConfig::new(bundle).build()))
}

async fn render(v: impl View) -> String {
    v.single().await.unwrap().render(&Cx::default())
}

#[tokio::test]
async fn renders_every_basic_node() {
    let reg = registry();
    let doc = reg.get_parsed_doc("guide/intro").unwrap();
    let cx = &Cx::default();
    let html = render(view! { cx => doc_content(nodes: &doc.content, base_path: "/docs") }).await;

    // Prose is pre-rendered HTML and must not be escaped again.
    assert!(html.contains("<strong>prose</strong>"), "{html}");
    assert!(html.contains("role=\"alert\""), "{html}");
    assert!(
        html.contains("<span class=\"hl-keyword\">fn</span>"),
        "{html}"
    );
    // Tabs: one radio group, first tab checked.
    assert_eq!(html.matches("name=\"dk-tabs-c-").count(), 2, "{html}");
    assert!(html.contains("aria-label=\"One\" checked"), "{html}");
    // Steps drop the redundant "Step 1:" prefix.
    assert!(html.contains(">Install</h4>"), "{html}");
    assert!(html.contains("<details"), "{html}");
}

#[tokio::test]
async fn page_shell_marks_the_active_page_and_links_neighbors() {
    let reg = registry();
    let cx = &Cx::default();
    let html =
        render(view! { cx => docs_page(registry: reg, base_path: "/docs", path: "guide/next") })
            .await;

    assert!(
        html.contains(r#"href="/docs/guide/next" aria-current="page""#),
        "{html}"
    );
    // Only the active tab's groups are in the sidebar.
    assert!(!html.contains("Extra</h3>"), "{html}");
    // Previous link to the intro, no next link (end of the tab).
    assert!(html.contains("dk-page-prev"), "{html}");
    assert!(!html.contains("dk-page-next"), "{html}");
    // Table of contents from the page's headings.
    assert!(html.contains(r##"href="#section-one""##), "{html}");
}

#[tokio::test]
async fn empty_path_renders_the_default_page() {
    let reg = registry();
    let cx = &Cx::default();
    let html = render(view! { cx => docs_page(registry: reg, base_path: "/docs", path: "") }).await;
    assert!(html.contains(">Intro</h1>"), "{html}");
}

#[tokio::test]
async fn unknown_path_renders_not_found() {
    let reg = registry();
    let cx = &Cx::default();
    let html =
        render(view! { cx => docs_page(registry: reg, base_path: "/docs", path: "nope") }).await;
    assert!(html.contains("Page not found: nope"), "{html}");
}

#[tokio::test]
async fn head_omits_a_missing_description() {
    let reg = registry();
    let cx = &Cx::default();
    let with = render(view! { cx => docs_head(registry: reg, path: "guide/intro") }).await;
    assert_eq!(
        with,
        r#"<title>Intro</title><meta name="description" content="The first page">"#
    );
    let without = render(view! { cx => docs_head(registry: reg, path: "guide/next") }).await;
    assert_eq!(without, "<title>Next page</title>");
}

#[tokio::test]
async fn header_search_form_submits_to_the_search_page() {
    let reg = registry();
    let cx = &Cx::default();
    let html =
        render(view! { cx => docs_page(registry: reg, base_path: "/docs", path: "guide/intro") })
            .await;
    assert!(
        html.contains(r#"action="/docs/search" method="get" role="search""#),
        "{html}"
    );
    assert!(html.contains(r#"name="q" value="""#), "{html}");
}

#[tokio::test]
async fn search_page_links_hits_and_highlights_terms() {
    let reg = registry();
    let cx = &Cx::default();
    let html = render(
        view! { cx => docs_search_page(registry: reg, base_path: "/docs", query: "widgets") },
    )
    .await;

    assert!(
        html.contains("1 result for \u{201c}widgets\u{201d}"),
        "{html}"
    );
    assert!(
        html.contains(r#"href="/docs/guide/next#section-one""#),
        "{html}"
    );
    assert!(html.contains(r#"<mark class="dk-search-mark"#), "{html}");
    // The query is echoed back into the search box.
    assert!(html.contains(r#"name="q" value="widgets""#), "{html}");
}

#[tokio::test]
async fn search_page_handles_blank_and_unmatched_queries() {
    let reg = registry();
    let cx = &Cx::default();
    let blank =
        render(view! { cx => docs_search_page(registry: reg, base_path: "/docs", query: "  ") })
            .await;
    assert!(blank.contains("Type a word or phrase"), "{blank}");
    let none = render(
        view! { cx => docs_search_page(registry: reg, base_path: "/docs", query: "zzzqqq") },
    )
    .await;
    assert!(none.contains("0 results for"), "{none}");
    assert!(none.contains("No pages match"), "{none}");
}

#[tokio::test]
async fn search_head_is_noindex() {
    let cx = &Cx::default();
    let html = render(view! { cx => docs_search_head(query: " tabs ") }).await;
    assert_eq!(
        html,
        r#"<title>Search: tabs</title><meta name="robots" content="noindex">"#
    );
}

#[tokio::test]
async fn endpoint_page_renders_operation_and_samples() {
    let reg = registry();
    let cx = &Cx::default();
    let html = render(
        view! { cx => docs_page(registry: reg, base_path: "/docs", path: "api/delete-pet") },
    )
    .await;

    assert!(html.contains(">Delete a pet</h1>"), "{html}");
    assert!(html.contains(">/pets/{id}</code>"), "{html}");
    assert!(html.contains("https://pets.example.com"), "{html}");
    // Parameter, response with an expandable schema, curl sample.
    assert!(html.contains(">id</code>"), "{html}");
    assert!(
        html.contains("<details class=\"group border border-base-300"),
        "{html}"
    );
    assert!(html.contains("curl"), "{html}");
    // Sidebar: the endpoint is listed under its tag, active, with a short badge.
    assert!(
        html.contains(r#"href="/docs/api/delete-pet" aria-current="page""#),
        "{html}"
    );
    assert!(html.contains(">DEL</span>"), "{html}");
    // Previous page is the API overview.
    assert!(
        html.contains(r#"href="/docs/api/overview" class="dk-page-prev"#),
        "{html}"
    );
}

#[tokio::test]
async fn search_includes_endpoint_hits_with_a_method_badge() {
    let reg = registry();
    let cx = &Cx::default();
    let html = render(
        view! { cx => docs_search_page(registry: reg, base_path: "/docs", query: "delete") },
    )
    .await;
    assert!(html.contains(r#"href="/docs/api/delete-pet""#), "{html}");
    assert!(html.contains(">DELETE</span>"), "{html}");
}
