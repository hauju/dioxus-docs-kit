# topcoat-docs-kit

Documentation site shell for [Topcoat](https://github.com/tokio-rs/topcoat)
apps, on the same build-time content pipeline as
[dioxus-docs-kit](https://crates.io/crates/dioxus-docs-kit).

Write Mintlify-style MDX plus a `_nav.json`, bundle it at build time with
[`dioxus-docs-kit-build`](https://crates.io/crates/dioxus-docs-kit-build), and
render it with server-side Topcoat components. No runtime script and no wasm:
tabs, accordions and copy buttons work in plain HTML.

**Early.** Topcoat itself is experimental, and so is this crate. Requires Rust 1.98.

## What renders

- Docs shell: tab bar, sidebar with active page, table of contents,
  previous/next links, 404 page with status code, `<title>`/description.
- OpenAPI: endpoint pages (method, parameters, request body, responses with
  expandable schemas, curl and example-response samples) and the endpoints in
  the sidebar, grouped by tag.
- Search: a header search box (plain GET form) and a results page with ranked
  hits and highlighted snippets — the same index and ranking as the Dioxus
  kit's search modal.
- MDX components: callouts, cards and card groups, tabs, steps, accordions,
  code blocks and code groups (syntax highlighted), param/response fields,
  request/response examples, changelog updates.

Not ported yet: the inline `<OpenAPI>` MDX viewer, the blog, the theme toggle,
Mermaid diagrams (shown as code).

## Usage

```toml
[dependencies]
topcoat-docs-kit = "0.10"

[build-dependencies]
dioxus-docs-kit-build = "0.10"
```

```rust,ignore
// build.rs
fn main() {
    dioxus_docs_kit_build::DocsBuild::new("docs/_nav.json").generate();
}
```

```rust,ignore
use std::sync::LazyLock;
use topcoat_docs_kit::{DocsConfig, DocsRegistry, STYLESHEET, docs_head, docs_page};

static DOCS: LazyLock<DocsRegistry> =
    LazyLock::new(|| DocsConfig::new(topcoat_docs_kit::docs_bundle!()).build());

#[component]
async fn document(path: &str) -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html data-theme="dark">
            <head>
                docs_head(registry: &DOCS, path: path)
                <link rel="stylesheet" href="/docs-kit.css">
            </head>
            <body>docs_page(registry: &DOCS, base_path: "/docs", path: path)</body>
        </html>
    })
}
```

Serve `STYLESHEET` at `/docs-kit.css`, route `/docs/{*doc_path}` to
`document`, and route `/docs/search` to a page that renders
`docs_search_head(query)` + `docs_search_page(registry, base_path, query)` with
the `q` query parameter. The full app is
[`examples/topcoat-docs`](https://github.com/hauju/dioxus-docs-kit/tree/main/examples/topcoat-docs):

```sh
cargo run -p topcoat-docs-example   # http://127.0.0.1:3000/docs
```
