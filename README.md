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
dioxus-docs-kit = "0.5"

[build-dependencies]
dioxus-docs-kit-build = "0.5"

[features]
default = ["web"]
web = ["dioxus/web", "dioxus-docs-kit/web"]
server = ["dioxus/server", "dioxus-docs-kit/server"]
```

The kit's default features are `web` + `mermaid` + `highlight`; if you disable default features, re-enable `mermaid` and `highlight` too, or ` ```mermaid ` fences stop rendering as diagrams and code blocks lose syntax coloring.

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
      "group": "Getting Started",
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

    let current_path = match route {
        Route::DocsPage { slug } => slug.join("/"),
        _ => String::new(),
    };

    // `use_docs_context` rewraps the path reactively for you. Building the
    // signal yourself with a plain `use_memo(move || ...)` captures the first
    // route and never updates — the sidebar highlight would freeze.
    let docs_ctx = use_docs_context(
        current_path,
        "/docs",
        Callback::new(move |path: String| {
            let slug: Vec<String> = path.split('/').map(String::from).collect();
            nav.push(Route::DocsPage { slug });
        }),
    );

    let providers = use_docs_providers(&DOCS, docs_ctx);
    let search_open = providers.search_open;

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

### 5. Add the styles

**Option A — precompiled stylesheet (zero setup, recommended to start).** The
crate ships a compiled stylesheet covering everything its components emit
(Tailwind utilities, DaisyUI dark/light themes, typography prose, the `--dk-*`
theming tokens). Link it and you're done — no Tailwind, no Bun, no safelist:

```rust
rsx! {
    document::Stylesheet { href: dioxus_docs_kit::DOCS_KIT_CSS }
}
```

The sheet only contains the *kit's* classes. If your own pages use Tailwind
utilities the kit doesn't, switch to Option B.

**Option B — your own Tailwind build.** When `dioxus-docs-kit` is a crates.io
dependency, Tailwind CSS 4 cannot scan `~/.cargo/` paths. Copy the safelist file into your project and reference it:

`safelist.html` ships at the root of the published crate, so copy it out of the
vendored source:

```sh
cp ~/.cargo/registry/src/*/dioxus-docs-kit-*/safelist.html safelist-docs-kit.html
```

(From a git checkout of this repo the path is `crates/dioxus-docs-kit/safelist.html`.)

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

## Used in production

- [Stepshots](https://stepshots.com/)

## License

MIT
