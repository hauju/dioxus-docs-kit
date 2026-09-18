# dioxus-docs-kit

Reusable documentation site kit for [Dioxus](https://dioxuslabs.com/) applications.

Drop-in layout with sidebar navigation, full-text search, page navigation, OpenAPI API reference pages, mobile drawer, and theme toggle. Built on [dioxus-mdx](https://crates.io/crates/dioxus-mdx) for content rendering.

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
dioxus-docs-kit = "0.8"
```

### 1. Parse your content in `build.rs`

[`dioxus-docs-kit-build`](https://crates.io/crates/dioxus-docs-kit-build) turns
your pages and specs into a single JSON bundle at compile time, so the wasm
client ships no MDX/Markdown/YAML parser and never re-parses a page:

```rust
// build.rs
fn main() {
    dioxus_docs_kit_build::DocsBuild::new("docs/_nav.json")
        .with_openapi("api-reference", "docs/api-reference/spec.yaml")
        .generate();
}
```

### 2. Build a Registry

The `DocsRegistry` holds all parsed content, the navigation tree, the search index, and any OpenAPI specs — it just deserializes the bundle:

```rust
use dioxus_docs_kit::{DocsConfig, DocsRegistry};
use std::sync::LazyLock;

static DOCS: LazyLock<DocsRegistry> = LazyLock::new(|| {
    DocsConfig::new(dioxus_docs_kit::docs_bundle!()).build()
});
```

### 3. Wire Into Your Router

Create a thin layout wrapper that provides the `DocsContext` and registry to the library components. The `use_docs_providers` hook bundles all the context setup into one call:

```rust
use dioxus::prelude::*;
use dioxus_docs_kit::{DocsLayout, use_docs_context, use_docs_providers};

#[component]
fn MyDocsLayout() -> Element {
    let nav = use_navigator();
    let route = use_route::<Route>();

    // A plain String — extract the slug from your route, e.g. slug.join("/").
    let current_path = /* ... */;

    // `use_docs_context` rewraps the path reactively for you. Building the
    // signal yourself with a plain `use_memo(move || ...)` captures the first
    // route and never updates — the sidebar highlight would freeze.
    let docs_ctx = use_docs_context(
        current_path,
        "/docs",
        Callback::new(move |path: String| {
            nav.push(/* build route from path */);
        }),
    )
    .with_site_url("https://your-site.com"); // optional: canonical/OG URLs

    let providers = use_docs_providers(&DOCS, docs_ctx);
    // providers.search_open / providers.drawer_open are available for
    // wiring a custom header.

    rsx! {
        DocsLayout {
            Outlet::<Route> {}
        }
    }
}
```

### 4. Add Routes

```rust
#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[layout(MyDocsLayout)]
        #[route("/docs")]
        DocsIndex {},
        #[route("/docs/:..slug")]
        DocsPage { slug: Vec<String> },
}
```

That's it. The library handles sidebar rendering, search, page content, previous/next navigation, and mobile responsiveness.

## Components

| Component | Description |
|-----------|-------------|
| `DocsLayout` | Full page layout with sidebar, content area, and table of contents |
| `DocsSidebar` | Navigation sidebar built from `_nav.json` |
| `DocsPageContent` | Renders MDX docs or OpenAPI endpoint pages |
| `DocsPageNav` | Previous/next page navigation |
| `SearchModal` | Full-text search across all docs |
| `SearchButton` | Trigger button for the search modal |
| `MobileDrawer` | Mobile navigation drawer |
| `ThemeToggle` | Light/dark theme switcher |

## Navigation Config

Define your docs structure in `_nav.json`. `tabs` is a flat list of tab names, and each group optionally names the tab it belongs to:

```json
{
  "tabs": ["Docs", "Guides"],
  "groups": [
    {
      "group": "Getting Started",
      "tab": "Docs",
      "pages": [
        "getting-started/introduction",
        "getting-started/quickstart"
      ]
    }
  ]
}
```

## Content Pipeline

All doc content is parsed at compile time. `dioxus-docs-kit-build` reads
`_nav.json`, parses every referenced `.mdx` file into the renderer's node tree
(prose already rendered to HTML), parses any OpenAPI spec, builds the search
index, and writes one `docs_bundle.json` into `OUT_DIR`. `docs_bundle!()`
embeds it with `include_str!` and the registry deserializes it once, on first
use.

Keep both crates on the same version: the bundle carries a format version and
the runtime rejects one it does not understand.

See the [example project](https://github.com/hauju/dioxus-docs-kit) for a complete `build.rs` implementation.

## Styling Setup

### Zero-setup: the precompiled stylesheet

The crate ships a compiled stylesheet (`DOCS_KIT_CSS`) covering every class its
components emit — Tailwind utilities, DaisyUI dark/light themes, typography
prose, and the `--dk-*` theming tokens. Link it and skip the rest of this
section — no Tailwind, no Bun, no safelist:

```rust
rsx! {
    document::Stylesheet { href: dioxus_docs_kit::DOCS_KIT_CSS }
}
```

It only contains the *kit's* classes; if your own pages use Tailwind utilities
the kit doesn't, run your own build instead. That path requires **Tailwind CSS
4**, **DaisyUI 5**, and **@tailwindcss/typography**:

### Install dependencies

```sh
bun add tailwindcss @tailwindcss/typography daisyui
```

### Configure Tailwind

Add to your `tailwind.css`:

```css
@import "tailwindcss";
@plugin "@tailwindcss/typography";
@plugin "daisyui" {
    themes: dark --default, light;
}

@source "./src/**/*.{rs,html,css}";
```

### Include dioxus-docs-kit classes

When using as a **crates.io dependency**, Tailwind can't scan the crate source
(it lives in `~/.cargo` with machine-specific paths). Copy `safelist.html` from the
crate into your project root and add it as a source:

```css
@source "./safelist.html";
```

The safelist includes all classes from both `dioxus-docs-kit` and `dioxus-mdx`, including
dynamic runtime classes that Tailwind cannot detect from source scanning alone.

When using as a **workspace path dependency**, you can point directly at the source instead:

```css
@source "./crates/dioxus-docs-kit/src/**/*.rs";
@source "./crates/dioxus-mdx/src/**/*.rs";
```

## Theming

The kit exposes a public theming surface so consumers can restyle without
forking or fighting DaisyUI internals.

### Public CSS variables

Copy `theme.css` from the crate into your project and import it alongside
Tailwind:

```css
@import "tailwindcss";
@import "./theme.css";
```

All tokens have DaisyUI fallbacks, so DaisyUI users inherit the current theme;
non-DaisyUI users get a sensible neutral default. Override any of them:

```css
.my-site .dk-root {
  --dk-accent: #f0a57c;
  --dk-font-heading: 'Instrument Serif', serif;
  --dk-radius-lg: 20px;
}
```

| Token | Purpose |
|-------|---------|
| `--dk-bg`, `--dk-bg-sub`, `--dk-bg-alt` | Surface colors |
| `--dk-fg`, `--dk-muted`, `--dk-dim` | Foreground / text colors |
| `--dk-border` | Border color |
| `--dk-accent`, `--dk-accent-fg`, `--dk-accent-soft` | Accent |
| `--dk-radius-sm`, `--dk-radius`, `--dk-radius-lg` | Corner radii |
| `--dk-font-body`, `--dk-font-heading`, `--dk-font-mono` | Typography |
| `--dk-article-width`, `--dk-sidebar-width`, `--dk-toc-width` | Layout widths |

### Stable `dk-*` classes

Structural nodes carry semver-stable `dk-*` class names. Target these in your
own CSS instead of DaisyUI or internal classes:

| Class | What it wraps |
|-------|---------------|
| `dk-root`, `dk-docs-root` | Outermost docs wrapper |
| `dk-header`, `dk-shell`, `dk-sidebar`, `dk-main`, `dk-toc` | Layout regions |
| `dk-tabs`, `dk-tab`, `dk-tab-active` | Tab bar |
| `dk-nav`, `dk-nav-group`, `dk-nav-group-title`, `dk-nav-item`, `dk-nav-item-active` | Sidebar navigation |
| `dk-article`, `dk-article-header`, `dk-article-title`, `dk-article-description`, `dk-article-body` | Article regions |
| `dk-pagination`, `dk-page-prev`, `dk-page-next` | Previous/next links |
| `dk-search-trigger`, `dk-search-dialog`, `dk-search-input`, `dk-search-results`, `dk-search-result` | Search |
| `dk-drawer` | Mobile drawer |

### Slots on `DocsLayout`

`DocsLayout` accepts optional element slots. Each slot is wrapped in a
`dk-*-slot` class for CSS hooks.

```rust
DocsLayout {
    announcement_bar: Some(rsx!{ AnnouncementBar {} }),
    sidebar_header: Some(rsx!{ MyProductSwitcher {} }),
    sidebar_footer: Some(rsx!{ EditOnGitHub {} }),
    footer: Some(rsx!{ SiteFooter {} }),
    Outlet::<Route> {}
}
```

`DocsPageContent` additionally accepts an `article_footer` slot (rendered
below the article body, before pagination) — useful for "Was this helpful?"
widgets.

### Density variants

`DocsLayout` accepts a `variant` prop for two built-in density presets:

```rust
DocsLayout { variant: DocsVariant::Reference, Outlet::<Route> {} }
```

| Variant | Feel | `--dk-article-width` |
|---------|------|----------------------|
| `Prose` (default) | Wide margins, serif-friendly, long-form reading | `72ch` |
| `Reference` | Tighter column, smaller type, denser headings | `64ch` |

The variant is emitted as a class on `dk-root` (`dk-variant-prose` or
`dk-variant-reference`) so consumers can layer further tweaks.

### Pre-built theme examples

`examples/themes/` ships three drop-in visual identities built entirely on
top of `--dk-*` tokens:

- `warm-editorial.css` — amber accent, serif headings, cream surfaces
- `brutalist-light.css` — high contrast, zero-radius, monospace
- `default.css` — baseline (nothing overridden)

See [THEMING.md](./THEMING.md) for the full roadmap and proposals open for
community contribution.

## Features

- `web` (default) — enables web-specific features (propagated to `dioxus-mdx`)
- `mermaid` (default) — renders ` ```mermaid ` fences as diagrams
- `openapi` (default) — renders API reference pages from OpenAPI specs. The spec is parsed by `dioxus-docs-kit-build`, so this costs no extra dependency; disabling it (`default-features = false`) drops the endpoint pages, the API sidebar and the API search entries, and the registry ignores any specs in the bundle. Docs pages and the blog are unaffected.
- `server` — Axum route builders for crawler-facing endpoints

With the `server` feature, `SeoRouter` generates per-page raw-Markdown routes, `llms.txt` / `llms-full.txt`, sitemaps, blog RSS, and robots.txt as plain Axum routes (server functions would JSON-encode the bodies):

```rust
use dioxus_docs_kit::server::SeoRouter;

dioxus::server::serve(|| async {
    let seo = SeoRouter::new("https://your-site.com", "My Docs", "Documentation")
        .with_docs(&DOCS, "/docs")
        .into_router();
    Ok(dioxus::server::router(App).merge(seo))
});
```

## Syntax Highlighting

Code blocks are highlighted by [`hl-lite`](https://crates.io/crates/hl-lite),
a dependency-free lexer set that ships with the kit. There is nothing to
enable and no grammar to pick: twelve languages cost about 48 KB of wasm in
total, no C is compiled, and no toolchain beyond `cargo` is required.

`bash`, `css`, `dockerfile`, `html`, `javascript`, `json`, `markdown`,
`python`, `rust`, `toml`, `typescript`, `yaml` — plus the usual fence aliases
(`rs`, `sh`, `zsh`, `console`, `yml`, `jsx`, `tsx`, `jsonc`, `md`, `htm`, …).
A fence whose language is unknown renders as plain, uncolored text in the same
markup; a fence with no language falls back to the block's filename.

### Colors

Each token is a `<span class="hl-{kind}">` inside `<pre class="dk-code">`, so
colors live in CSS. `theme.css` defines one `--dk-hl-*` token per kind with a
`light-dark()` pair (GitHub Light / Tokyo Night) that follows the active
theme's `color-scheme`. Recolor any of them the way you would any other
`--dk-*` token:

```css
.dk-root {
    --dk-hl-keyword: #d73a49;
    --dk-hl-string:  #032f62;
    --dk-hl-comment: #6a737d;
}
```

The full list: `--dk-hl-keyword`, `--dk-hl-string`, `--dk-hl-comment`,
`--dk-hl-number`, `--dk-hl-type`, `--dk-hl-function`, `--dk-hl-attribute`,
`--dk-hl-property`, `--dk-hl-tag`, `--dk-hl-operator`, `--dk-hl-punctuation`,
`--dk-hl-constant`, `--dk-hl-variable`.

### Highlighting your own snippets

`DocCodeBlock` is public, so any code outside a docs page renders identically:

```rust
use dioxus_docs_kit::{CodeBlockNode, DocCodeBlock};

rsx! {
    DocCodeBlock {
        block: CodeBlockNode {
            language: Some("rust".to_string()),
            code: snippet,
            filename: Some("main.rs".to_string()),
        },
    }
}
```

For raw access to the lexer, the kit re-exports it as `dioxus_docs_kit::hl`.

## Production build

Bundle with debug symbols off:

```sh
dx bundle --web --release --debug-symbols false
```

`dx` keeps DWARF by default and `wasm-opt` aborts on it ("compile unit size was incorrect"), after which `dx` silently ships the *unoptimized* wasm-bindgen output — for this repo's own site that was 25 MB instead of 7 MB. Two more things worth copying from this repo's root `Cargo.toml` and CI workflow:

- `[profile.wasm-release]` / `[profile.server-release]` — the profiles `dx` actually builds with (`opt-level = "z"`, fat LTO, `panic = "abort"` for the wasm; `strip = true` for the server).
- Brotli sidecars: write a `.br` next to every `.wasm`/`.js`/`.css` under the bundle's `public/` directory (`brotli -q 11 -k`). `dioxus-server` serves them automatically with `content-encoding: br`, taking the wasm from ~7 MB to ~1.4 MB on the wire. `scripts/wasm-size.sh` in this repo reports both numbers and can gate CI on the raw size.

## License

MIT
