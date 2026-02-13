# dioxus-docs-kit

A documentation site framework for [Dioxus 0.7](https://dioxuslabs.com/) with MDX content, sidebar navigation, full-text search, OpenAPI API reference pages, and theme switching — all embedded at compile time.

## Crates

| Crate | Description |
|-------|-------------|
| [`dioxus-docs-kit`](crates/dioxus-docs-kit/) | Docs site shell (layout, sidebar, search, page nav, OpenAPI) |
| [`dioxus-docs-kit-build`](crates/dioxus-docs-kit-build/) | Build-time helper that generates content maps from `_nav.json` |
| [`dioxus-mdx`](crates/dioxus-mdx/) | Standalone MDX parser + renderer (usable without the full shell) |

## Integration Guide

### 1. Add dependencies

```toml
# Cargo.toml
[dependencies]
dioxus = { version = "0.7", features = ["router", "fullstack"] }
dioxus-docs-kit = { git = "https://github.com/hauju/dioxus-docs-kit.git", default-features = false }

[build-dependencies]
dioxus-docs-kit-build = { git = "https://github.com/hauju/dioxus-docs-kit.git" }

[features]
default = ["web"]
web = ["dioxus/web", "dioxus-docs-kit/web"]
server = ["dioxus/server"]
```

### 2. Set up `build.rs`

```rust
fn main() {
    dioxus_docs_kit_build::generate_content_map("docs/_nav.json");
}
```

This reads your `_nav.json`, generates `include_str!()` calls for every `.mdx` file, and writes the result to `$OUT_DIR/doc_content_generated.rs`.

### 3. Create content

Create `docs/_nav.json` with your navigation structure:

```json
{
  "groups": [
    {
      "title": "Getting Started",
      "pages": [
        "getting-started/introduction",
        "getting-started/installation"
      ]
    }
  ]
}
```

Then create matching MDX files at `docs/getting-started/introduction.mdx`, etc.

### 4. Wire up routes and layout

In `src/main.rs`:

```rust
use dioxus::prelude::*;
use dioxus_docs_kit::{
    DocsConfig, DocsContext, DocsLayout, DocsPageContent, DocsRegistry,
    SearchButton, use_docs_providers,
};
use std::sync::LazyLock;

// Generate the content map function from build.rs output
dioxus_docs_kit::doc_content_map!();

// Build the registry (parses all docs, builds search index)
static DOCS: LazyLock<DocsRegistry> = LazyLock::new(|| {
    DocsConfig::new(include_str!("../docs/_nav.json"), doc_content_map())
        .with_default_path("getting-started/introduction")
        .with_theme_toggle("light", "dark", "dark")
        // Optional: .with_openapi("api-reference", include_str!("../docs/api-reference/spec.yaml"))
        .build()
});

#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[layout(MyDocsLayout)]
        #[redirect("/docs", || Route::DocsPage { slug: vec!["getting-started".into(), "introduction".into()] })]
        #[route("/docs/:..slug")]
        DocsPage { slug: Vec<String> },
}

/// Layout wrapper — wires DocsContext + DocsRegistry into the library
#[component]
fn MyDocsLayout() -> Element {
    let nav = use_navigator();
    let route = use_route::<Route>();

    let current_path = use_memo(move || match route.clone() {
        Route::DocsPage { slug } => slug.join("/"),
        _ => String::new(),
    });

    let docs_ctx = DocsContext {
        current_path: current_path.into(),
        base_path: "/docs".into(),
        navigate: Callback::new(move |path: String| {
            let slug: Vec<String> = path.split('/').map(String::from).collect();
            nav.push(Route::DocsPage { slug });
        }),
    };

    let providers = use_docs_providers(&DOCS, docs_ctx);
    let search_open = providers.search_open;
    let mut drawer_open = providers.drawer_open;

    rsx! {
        DocsLayout {
            header: rsx! {
                // Your custom navbar here — use search_open and drawer_open as needed
                SearchButton { search_open }
            },
            Outlet::<Route> {}
        }
    }
}

#[component]
fn DocsPage(slug: Vec<String>) -> Element {
    rsx! {
        DocsPageContent { path: slug.join("/") }
    }
}
```

### 5. Set up Tailwind CSS

When `dioxus-docs-kit` is a git dependency, Tailwind CSS 4 cannot scan `~/.cargo/` paths. Copy the safelist file into your project and reference it:

```sh
cp path/to/dioxus-docs-kit/crates/dioxus-docs-kit/safelist.html safelist-docs-kit.html
```

Then in your `tailwind.css`:

```css
@import "tailwindcss";
@plugin "daisyui";

@source "./src/**/*.{rs,html,css}";
@source "./safelist-docs-kit.html";
```

The safelist file includes a version comment at the top — check it periodically and re-copy when the crate updates.

## Claude Code Skill

This repo ships a [Claude Code skill](skills/dioxus-docs-kit-integration/) that automates the full integration. Install it globally:

```sh
cp -r skills/dioxus-docs-kit-integration ~/.claude/skills/
```

Then open any Dioxus project with Claude Code and say:

> "Add dioxus-docs-kit documentation to this project"

The skill walks Claude through all 5 steps: dependencies, build.rs, content files, route/layout wiring, and Tailwind safelist.

## Running the Example

```sh
curl -sSL http://dioxus.dev/install.sh | sh
dx serve
```

## License

MIT
