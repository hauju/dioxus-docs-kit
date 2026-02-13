---
name: dioxus-docs-kit-integration
description: >-
  Integrate dioxus-docs-kit into a Dioxus 0.7 project. Adds compile-time MDX docs,
  sidebar navigation, full-text search, optional OpenAPI reference pages, and theme
  switching. Handles Cargo.toml dependencies, build.rs setup, doc_content_map macro,
  route/layout wiring with use_docs_providers, Tailwind CSS safelist, and _nav.json
  creation. Use when: (1) Adding documentation to a Dioxus project, (2) Setting up
  dioxus-docs-kit in a new or existing app, (3) "add docs", "integrate docs-kit",
  "set up documentation site", (4) Migrating from manual context providers to
  use_docs_providers. Triggers: "dioxus-docs-kit", "add docs to project",
  "integrate documentation", "docs-kit setup", "add dioxus docs".
---

# Dioxus Docs Kit Integration

Integrate `dioxus-docs-kit` into any Dioxus 0.7 fullstack project in 5 steps.

## Pre-flight

Before starting, identify:
1. **The project's `Cargo.toml`** — confirm `dioxus` version is 0.7+
2. **The `Route` enum** — find it (usually `src/main.rs`) to know existing routes and layout wrappers
3. **The `tailwind.css`** — confirm Tailwind CSS 4 + DaisyUI 5 are in use
4. **Whether OpenAPI is needed** — ask the user if not obvious from context

## Step 1: Dependencies

Add to the project's `Cargo.toml`:

```toml
[dependencies]
dioxus-docs-kit = { version = "0.1", default-features = false }

[build-dependencies]
dioxus-docs-kit-build = "0.1"
```

Add feature flags:

```toml
[features]
web = ["dioxus/web", "dioxus-docs-kit/web"]  # add to existing web feature
```

## Step 2: Build script

Create or replace `build.rs`:

```rust
fn main() {
    dioxus_docs_kit_build::generate_content_map("docs/_nav.json");
}
```

If the project already has a `build.rs`, add the call to the existing `main()`.

## Step 3: Content files

Create `docs/_nav.json`:

```json
{
  "groups": [
    {
      "title": "Getting Started",
      "pages": [
        "getting-started/introduction"
      ]
    }
  ]
}
```

Create `docs/getting-started/introduction.mdx` with starter content:

```mdx
---
title: Introduction
description: Getting started with the project
---

Welcome to the documentation.
```

## Step 4: Route and layout wiring

Add the macro at module level in `main.rs` (or wherever the Route enum lives):

```rust
dioxus_docs_kit::doc_content_map!();
```

Create the static registry:

```rust
use std::sync::LazyLock;
use dioxus_docs_kit::{DocsConfig, DocsRegistry};

static DOCS: LazyLock<DocsRegistry> = LazyLock::new(|| {
    DocsConfig::new(include_str!("../docs/_nav.json"), doc_content_map())
        .with_default_path("getting-started/introduction")
        // Optional: .with_openapi("api-reference", include_str!("../docs/api-reference/spec.yaml"))
        // Optional: .with_theme_toggle("light", "dark", "dark")
        .build()
});
```

Add docs routes to the `Route` enum:

```rust
#[layout(MyDocsLayout)]
    #[redirect("/docs", || Route::DocsPage { slug: vec!["getting-started".into(), "introduction".into()] })]
    #[route("/docs/:..slug")]
    DocsPage { slug: Vec<String> },
#[end_layout]
```

Create the layout wrapper and page component:

```rust
use dioxus::prelude::*;
use dioxus_docs_kit::{
    DocsContext, DocsLayout, DocsPageContent, SearchButton, use_docs_providers,
};

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
                // Adapt this navbar to match the project's existing design
                div { class: "navbar bg-base-200 border-b border-base-300 px-4 lg:px-8",
                    div { class: "flex-1 gap-2",
                        button {
                            class: "btn btn-ghost btn-sm btn-square lg:hidden",
                            onclick: move |_| drawer_open.toggle(),
                            // hamburger icon
                        }
                        // project logo / name link
                    }
                    div { class: "flex-none flex items-center gap-1",
                        SearchButton { search_open }
                        // optional: ThemeToggle {}
                    }
                }
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

**Key adaptation points:**
- The header RSX should match the project's existing navbar style
- The `Route::DocsPage` variant name and redirect target should match the project's first doc page
- If the project uses `ThemeToggle`, import and add it to the header

## Step 5: Tailwind CSS safelist

Copy the safelist from this repo into the consumer project:

```sh
# From the dioxus-docs-kit repo:
cp crates/dioxus-docs-kit/safelist.html <project>/safelist-docs-kit.html
```

In the project's `tailwind.css`, add:

```css
@source "./safelist-docs-kit.html";
```

Remove any fragile `@source` paths scanning `~/.cargo/`:

```css
/* REMOVE lines like these: */
@source "../../.cargo/git/checkouts/dioxus-docs-kit-*/**/*.{rs,html,css}";
@source "./.cargo-vendor/dioxus-docs-kit/**/*.{rs,html,css}";
```

Also remove any CI workarounds (symlinks, cargo-vendor directories) that existed to support the old approach.

## Optional: LLMs.txt endpoints

```rust
#[get("/llms.txt")]
async fn llms_txt() -> Result<String, ServerFnError> {
    Ok(DOCS.generate_llms_txt("Project Name", "Description", "https://github.com/..."))
}

#[get("/llms-full.txt")]
async fn llms_full_txt() -> Result<String, ServerFnError> {
    Ok(DOCS.generate_llms_full_txt("Project Name", "Description", "https://github.com/..."))
}
```

## Checklist

After integration, verify:
- [ ] `dx serve` starts without errors
- [ ] `/docs` redirects to the default page
- [ ] Sidebar navigation works
- [ ] Search modal opens (Ctrl/Cmd+K)
- [ ] All component styles render (callouts, code blocks, badges)
- [ ] Mobile drawer opens/closes
- [ ] Theme toggle works (if enabled)
