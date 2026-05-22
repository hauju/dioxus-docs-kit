---
name: dioxus-docs-kit-integration
description: >-
  Integrate dioxus-docs-kit into a Dioxus 0.7 project. Adds compile-time MDX docs,
  sidebar navigation, full-text search, optional OpenAPI reference pages, an
  optional blog engine, SEO meta + sitemap, and theme switching (DaisyUI themes
  or dk-* CSS presets). Handles Cargo.toml dependencies, build.rs setup,
  doc_content_map / blog_content_map macros, route/layout wiring with
  use_docs_providers + use_blog_providers, Tailwind CSS safelist, and _nav.json
  creation. Use when: (1) Adding documentation to a Dioxus project, (2) Setting up
  dioxus-docs-kit in a new or existing app, (3) "add docs", "integrate docs-kit",
  "set up documentation site", (4) Adding a blog alongside docs, (5) Migrating
  from manual context providers to use_docs_providers. Triggers: "dioxus-docs-kit",
  "add docs to project", "integrate documentation", "docs-kit setup", "add dioxus
  docs", "add a blog".
---

# Dioxus Docs Kit Integration

Integrate `dioxus-docs-kit` (v0.5) into any Dioxus 0.7 fullstack project.

## Pre-flight

Before starting, identify:
1. **The project's `Cargo.toml`** — confirm `dioxus` is 0.7.x and uses the `fullstack` feature (server functions for sitemap / llms.txt need it).
2. **The `Route` enum** — usually in `src/main.rs`. Find existing layout wrappers so the new docs layout fits in.
3. **The `tailwind.css`** — confirm Tailwind CSS 4 + DaisyUI 5 are in use.
4. **Optional surfaces** — ask the user (or infer):
   - OpenAPI reference pages? (Need a spec file.)
   - Blog engine?
   - SEO meta + sitemap? (Need the public site URL.)

## Step 1: Dependencies

Add to the consumer project's `Cargo.toml`:

```toml
[dependencies]
dioxus = { version = "0.7.9", features = ["router", "fullstack"] }
dioxus-docs-kit = { version = "0.5", default-features = false }

[build-dependencies]
dioxus-docs-kit-build = "0.5"
```

Wire features so `web` propagates to the kit:

```toml
[features]
default = ["web"]
web = ["dioxus/web", "dioxus-docs-kit/web"]
server = ["dioxus/server"]
# Optional: keep mermaid rendering
mermaid = ["dioxus-docs-kit/mermaid"]
```

`default-features = false` on the kit is important — the kit's own default
includes `web`, which would conflict with fullstack feature unification.

## Step 2: Build script

Create or extend `build.rs`:

```rust
fn main() {
    dioxus_docs_kit_build::generate_content_map("docs/_nav.json");
    // Only if adding the blog:
    // dioxus_docs_kit_build::generate_blog_content_map("blog/_blog.json");
}
```

The generator reads the nav/manifest JSON and emits an `include_str!()` call
for every listed `.mdx` file. If a path doesn't exist, the build fails at
compile time.

## Step 3: Content files

### Docs nav

Create `docs/_nav.json`:

```json
{
  "tabs": ["Docs", "API Reference"],
  "groups": [
    {
      "group": "Getting Started",
      "tab": "Docs",
      "pages": ["getting-started/introduction"]
    },
    {
      "group": "API Reference",
      "tab": "API Reference",
      "pages": ["api-reference/overview"]
    }
  ]
}
```

- `tabs` is the array of tab labels shown above the sidebar. Omit for a
  single-tab sidebar.
- `groups[].tab` ties a group to one tab. Omit for tab-less layouts.
- `groups[].pages` are paths relative to `docs/`, without the `.mdx` extension.
- **Do not** list OpenAPI operation IDs in `pages` — they're injected
  dynamically into the group whose name matches `api_group_name` (default
  `"API Reference"`).

Create `docs/getting-started/introduction.mdx`:

```mdx
---
title: Introduction
description: Getting started with the project
---

Welcome to the documentation.
```

### Optional: OpenAPI spec

If using the API reference tab, drop the spec at
`docs/api-reference/openapi.yaml` (JSON also works) and an overview MDX at
`docs/api-reference/overview.mdx`.

### Optional: Blog manifest

Create `blog/_blog.json`:

```json
{
  "authors": {
    "alex": {
      "name": "Alex Doe",
      "bio": "Builds things with Rust",
      "url": "https://example.com/@alex"
    }
  },
  "posts": ["hello-world"]
}
```

Each post lives at `blog/<slug>.mdx` with frontmatter:

```mdx
---
title: "Hello World"
description: "First post."
date: "2026-03-20"
author: "alex"
tags: ["announcement"]
featured: true
---

Post body.
```

## Step 4: Registry + macro

At module level in `src/main.rs`:

```rust
use std::sync::LazyLock;
use dioxus_docs_kit::{DocsConfig, DocsRegistry};

dioxus_docs_kit::doc_content_map!();

static DOCS: LazyLock<DocsRegistry> = LazyLock::new(|| {
    DocsConfig::new(include_str!("../docs/_nav.json"), doc_content_map())
        .with_default_path("getting-started/introduction")
        // Optional:
        // .with_openapi("api-reference", include_str!("../docs/api-reference/openapi.yaml"))
        // .with_theme_toggle("light", "dark", "dark")
        .build()
});
```

If adding the blog, add a parallel block:

```rust
use dioxus_docs_kit::{BlogConfig, BlogRegistry};

dioxus_docs_kit::blog_content_map!();

static BLOG: LazyLock<BlogRegistry> = LazyLock::new(|| {
    BlogConfig::new(include_str!("../blog/_blog.json"), blog_content_map())
        .with_posts_per_page(9)
        .with_theme_toggle("light", "dark", "dark")
        .build()
});
```

## Step 5: Routes

Extend the `Route` enum:

```rust
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(MyDocsLayout)]
        #[redirect("/docs", || Route::DocsPage { slug: vec!["getting-started".into(), "introduction".into()] })]
        #[route("/docs/:..slug")]
        DocsPage { slug: Vec<String> },
    #[end_layout]
    // Only if adding the blog:
    #[layout(MyBlogLayout)]
        #[route("/blog")]
        BlogIndex {},
        #[route("/blog/:slug")]
        BlogPage { slug: String },
}
```

## Step 6: Layout wrappers

```rust
use dioxus::prelude::*;
use dioxus_docs_kit::{
    DocsContext, DocsLayout, DocsPageContent, SearchButton, ThemeToggle,
    use_docs_providers,
};

#[component]
fn MyDocsLayout() -> Element {
    let nav = use_navigator();
    let route = use_route::<Route>();

    let current_path = use_memo(use_reactive!(|route| match route {
        Route::DocsPage { slug } => slug.join("/"),
        _ => String::new(),
    }));

    let docs_ctx = DocsContext {
        current_path: current_path.into(),
        base_path: "/docs".into(),
        navigate: Callback::new(move |path: String| {
            let slug: Vec<String> = path.split('/').map(String::from).collect();
            nav.push(Route::DocsPage { slug });
        }),
        site_url: Some("https://example.com".into()), // or None
        auto_meta: true,
    };

    let providers = use_docs_providers(&DOCS, docs_ctx);
    let search_open = providers.search_open;
    let mut drawer_open = providers.drawer_open;

    rsx! {
        DocsLayout {
            header: rsx! {
                // Replace with the project's existing navbar styling
                div { class: "navbar bg-base-200 border-b border-base-300 px-4 lg:px-8",
                    div { class: "flex-1 gap-2",
                        button {
                            class: "btn btn-ghost btn-sm btn-square lg:hidden",
                            onclick: move |_| drawer_open.toggle(),
                            // hamburger icon here
                        }
                        Link {
                            to: Route::DocsPage { slug: vec!["getting-started".into(), "introduction".into()] },
                            class: "text-xl font-semibold tracking-tight",
                            "Project Name"
                        }
                    }
                    div { class: "flex-none flex items-center gap-1",
                        SearchButton { search_open }
                        ThemeToggle {}
                    }
                }
            },
            Outlet::<Route> {}
        }
    }
}

#[component]
fn DocsPage(slug: Vec<String>) -> Element {
    rsx! { DocsPageContent { path: slug.join("/") } }
}
```

**`DocsContext` requires all five fields.** `site_url` and `auto_meta` are not
optional struct-wise; pass `None` / `false` if you don't want SEO meta.

If you added the blog, add the parallel wrapper:

```rust
use dioxus_docs_kit::{
    BlogContext, BlogLayout, BlogList, BlogPostView, BlogSearchButton,
    use_blog_providers,
};

#[component]
fn MyBlogLayout() -> Element {
    let nav = use_navigator();
    let route = use_route::<Route>();

    let current_slug = use_memo(use_reactive!(|route| match route {
        Route::BlogPage { slug } => slug,
        _ => String::new(),
    }));

    let blog_ctx = BlogContext {
        current_slug: current_slug.into(),
        base_path: "/blog".into(),
        navigate: Callback::new(move |slug: String| {
            if slug.is_empty() { nav.push(Route::BlogIndex {}); }
            else { nav.push(Route::BlogPage { slug }); }
        }),
        site_url: Some("https://example.com".into()),
        auto_meta: true,
    };

    let providers = use_blog_providers(&BLOG, blog_ctx);
    let search_open = providers.search_open;

    rsx! {
        BlogLayout {
            header: rsx! { /* navbar with BlogSearchButton { search_open } */ },
            Outlet::<Route> {}
        }
    }
}

#[component]
fn BlogIndex() -> Element { rsx! { BlogList {} } }

#[component]
fn BlogPage(slug: String) -> Element { rsx! { BlogPostView { slug } } }
```

## Step 7: Tailwind safelist

When the kit is a crates.io dep, Tailwind 4's `@source` cannot scan
`~/.cargo/` reliably. Ship a safelist file.

```sh
# From the dioxus-docs-kit repo (or download from GitHub):
cp crates/dioxus-docs-kit/safelist.html <project>/safelist-docs-kit.html
```

In `tailwind.css`:

```css
@import "tailwindcss";
@plugin "@tailwindcss/typography";
@plugin "daisyui" {
    themes: dark --default, light;
}

@source "./src/**/*.{rs,html,css}";
@source "./safelist-docs-kit.html";
```

Remove any old `@source` paths pointing at `~/.cargo/` and any CI symlinks /
cargo-vendor workarounds that existed to support that.

## Step 8 (optional): SEO meta, sitemap, robots, llms.txt

The `auto_meta: true` field on `DocsContext` already emits per-page `<title>`,
description, OpenGraph, and Twitter Card tags from MDX frontmatter. Canonical
URL + `og:url` emit only when `site_url` is also set.

For crawlers, add server functions:

```rust
const SITE_URL: &str = "https://example.com"; // no trailing slash

#[get("/sitemap.xml")]
async fn sitemap_index() -> Result<String, ServerFnError> {
    Ok(format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <sitemapindex xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n\
         <sitemap><loc>{SITE_URL}/sitemap-docs.xml</loc></sitemap>\n\
         </sitemapindex>\n"
    ))
}

#[get("/sitemap-docs.xml")]
async fn sitemap_docs() -> Result<String, ServerFnError> {
    Ok(DOCS.generate_sitemap(SITE_URL, "/docs"))
}

#[get("/robots.txt")]
async fn robots_txt() -> Result<String, ServerFnError> {
    Ok(format!("User-agent: *\nAllow: /\n\nSitemap: {SITE_URL}/sitemap.xml\n"))
}

#[get("/llms.txt")]
async fn llms_txt() -> Result<String, ServerFnError> {
    Ok(DOCS.generate_llms_txt(
        "Project Name",
        "One-line project description.",
        "https://github.com/owner/repo",
    ))
}

#[get("/llms-full.txt")]
async fn llms_full_txt() -> Result<String, ServerFnError> {
    Ok(DOCS.generate_llms_full_txt("Project Name", "...", "https://github.com/owner/repo"))
}
```

If using the blog, add a `/sitemap-blog.xml` entry that calls
`BLOG.generate_sitemap(SITE_URL, "/blog")` and include it in the sitemap
index.

## Step 9 (optional): Theme presets / `dk-*` token surface

Beyond DaisyUI themes, the kit exposes a flat `--dk-*` CSS token surface on
`.dk-root`. Override any subset in your stylesheet:

```css
.dk-root {
  --dk-article-width: 68ch;
  --dk-heading-font: "Inter Tight", system-ui, sans-serif;
  --dk-accent: oklch(0.65 0.20 250);
  --dk-radius: 0.25rem;
}
```

Drop-in presets live in `crates/dioxus-docs-kit/examples/themes/`
(`warm-editorial.css`, `brutalist-light.css`, `shadcn-light.css`,
`shadcn-dark.css`, `default.css`). Copy any to your `assets/` and load via
`document::Stylesheet` or an inline `<style>` block.

For density, pass `variant` to `DocsLayout`:

```rust
use dioxus_docs_kit::DocsVariant;
DocsLayout { variant: DocsVariant::Reference, /* ... */ }
```

`Prose` (default) is wide / serif-friendly; `Reference` is tighter and denser.

## Checklist

After integration, verify:
- [ ] `dx serve` starts without errors
- [ ] `/docs` redirects to the default page
- [ ] Sidebar tabs + groups render
- [ ] Search modal opens (Ctrl/Cmd+K) and returns hits
- [ ] OpenAPI endpoints appear under the API Reference group (if configured)
- [ ] Blog index + post render (if configured)
- [ ] `<head>` shows per-page title/description/OG tags when `auto_meta = true`
- [ ] `/sitemap.xml`, `/robots.txt`, `/llms.txt` return content (if wired)
- [ ] All component styles render — callouts, code blocks, HTTP method badges,
      parameter location badges (depend on safelist being loaded)
- [ ] Mobile drawer opens/closes
- [ ] Theme toggle persists across reloads (if enabled)
