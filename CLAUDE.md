# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Dev Commands

```sh
dx serve                    # Dev server with hot reload
dx build --release          # Production build
dx bundle --web --release --debug-symbols false   # Bundle for deployment (without the flag wasm-opt aborts on DWARF and dx ships the unoptimized wasm)
```

**Linting & testing (matches CI):**
```sh
cargo fmt --all --check     # Format check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo machete               # Unused dependency check
cargo test --workspace      # Run all tests
```

**Tailwind CSS generation** (required before clippy/test if `assets/tailwind.css` is missing):
```sh
bun install --frozen-lockfile
bunx @tailwindcss/cli -i tailwind.css -o assets/tailwind.css
```

**Shortcuts via justfile:** `just build`, `just test`, `just serve`

Requires Dioxus CLI (`dx`): `curl -sSL http://dioxus.dev/install.sh | sh`

## Project Architecture

**Dioxus 0.7 documentation site framework** — renders MDX docs, OpenAPI API reference pages, and a search modal. All content is embedded at compile time.

### Crate Layout

| Crate | Path | Purpose |
|-------|------|---------|
| `dioxus-docs-kit-example` | `src/main.rs` | Example app: routes, custom pages (Home, Blog, Navbar), docs glue |
| `dioxus-docs-kit` | `crates/dioxus-docs-kit/` | **Reusable docs shell** — layout, sidebar, search, page nav, theme toggle, OpenAPI |
| `dioxus-docs-kit-build` | `crates/dioxus-docs-kit-build/` | Build-time pipeline: parses `_nav.json` + all MDX + OpenAPI into one JSON bundle |
| `dioxus-mdx` | `crates/dioxus-mdx/` | Standalone MDX parser + renderer (Mintlify-style components) |
| `hl-lite` | `crates/hl-lite/` | Zero-dependency syntax highlighter (12 hand-written lexers → `hl-*` spans); not yet wired into the renderer |

**Dependency direction:** `dioxus-docs-kit` depends on `dioxus-mdx` with `features = ["components"]` (renderer only). `dioxus-docs-kit-build` depends on `dioxus-mdx` with `default-features = false, features = ["parse", "openapi-parse"]` (parser only, no dioxus). The mdx and docs-kit crates use `dioxus = { features = ["lib"] }` (NOT fullstack). Only the root example uses fullstack.

### Content Pipeline

**All parsing happens at build time.** The wasm client links no MDX parser, no `markdown`, no `regex`/`regex-lite`, no `openapiv3` and no `serde_yaml` — it only runs `serde_json` over the bundle, once.

1. `build.rs` calls `dioxus_docs_kit_build::DocsBuild::new("docs/_nav.json").with_openapi(prefix, spec_path).generate()` (and `BlogBuild::new("blog/_blog.json").generate()`)
2. The build crate reads `_nav.json`, parses every `.mdx` into `Vec<DocNode>` with prose rendered to HTML (`DocNode::Html`), parses the OpenAPI spec into `OpenApiSpec`, builds the section-level search index (lowercase fields included) and writes `$OUT_DIR/docs_bundle.json` / `blog_bundle.json`. `cargo:rerun-if-changed` is emitted per file.
3. `docs_bundle!()` / `blog_bundle!()` macros embed the JSON with `include_str!`
4. `DocsConfig::new(docs_bundle!()).build()` creates a `DocsRegistry` — it deserializes the bundle (inside the consumer's `LazyLock`, so on first page render) and derives only the cheap runtime bits (API sidebar entries, operation index, blog categories)
5. `DocsPageContent` checks `registry.get_api_operation(&path)` first, then falls back to `registry.get_parsed_doc(&path)`

The bundle carries a `version` field (`BUNDLE_VERSION`, mirrored in `crates/dioxus-docs-kit/src/bundle.rs` and `crates/dioxus-docs-kit-build/src/bundle.rs`); bump it whenever the layout changes incompatibly. The build crate's `bundle.rs` holds serialize-only mirrors of `SearchEntry`, `BlogPost`, `BlogFrontmatter` and `BlogSearchEntry` — the kit's tests dev-depend on the build crate and generate real bundles, which is what keeps the two definitions in sync.

**Adding a new doc page:** create `docs/<group>/<slug>.mdx` and add the path to `docs/_nav.json`.

### `_nav.json` Structure

```json
{
  "tabs": ["Docs", "Guides", "API Reference", "Changelog"],
  "groups": [
    { "group": "Getting Started", "tab": "Docs", "pages": ["getting-started/introduction"] }
  ]
}
```

- `tabs`: displayed in the tab bar above the sidebar
- `groups[].tab`: which tab a sidebar group belongs to (optional)
- `groups[].pages`: paths relative to `docs/`, without `.mdx` extension

### Routing Pattern

```
Route enum (main.rs):
  #[layout(Navbar)]        → Home (/), Blog (/blog/:id)
  #[layout(MyDocsLayout)]  → DocsIndex (/docs), DocsPage (/docs/:..slug)
```

- `MyDocsLayout` creates a `DocsContext` and calls `use_docs_providers(&DOCS, docs_ctx)` — one-call context setup
- `/docs/:..slug` is a catch-all; slug `Vec<String>` joined with `/` resolves content
- API endpoint slugs are kebab-cased from camelCase operationIds

### Key Types (dioxus-docs-kit)

- **`DocsConfig`** — Builder: `.new(docs_bundle!())` → `.with_theme_toggle()` → `.with_default_path()` → `.with_api_group_name()` → `.build()`. Content and OpenAPI specs come from the bundle, so they are configured in `build.rs`, not here
- **`DocsRegistry`** — Holds parsed docs, nav config, search index, OpenAPI specs. Key methods: `get_parsed_doc()`, `search_docs()`, `get_api_operation()`, `get_api_sidebar_entries()`, `tab_for_path()`, `generate_llms_txt()`, `generate_llms_full_txt()`
- **`DocsContext`** — Route decoupling bridge (`current_path`, `base_path`, `navigate` callback). Consumer provides this so library components don't depend on the consumer's Route enum
- **`use_docs_providers(registry, docs_ctx)`** → returns `DocsProviders { search_open, drawer_open }` for use in custom headers
- **UI components** — `DocsLayout`, `DocsPageContent`, `DocsSidebar`, `SearchModal`, `SearchButton`, `DocsPageNav`, `MobileDrawer`, `ThemeToggle`
- **`docs_bundle!()` / `blog_bundle!()`** — Macros that `include_str!` the build script's `$OUT_DIR/docs_bundle.json` / `blog_bundle.json`

### Styling

- **Tailwind CSS 4** + **DaisyUI 5** (dark theme default, light theme available)
- `@tailwindcss/typography` for prose content
- Input: `tailwind.css` → processed by `@tailwindcss/cli`
- Icons: Lucide SVGs vendored in `crates/dioxus-mdx/src/lucide.rs` — `Icon { class, icon: LdX }`,
  re-exported as `dioxus_docs_kit::lucide`. Regenerate with `scripts/vendor-lucide-icons.py`
  after adding a new `Ld*` name; nothing generates it at build time
- **Safelist pattern**: when crates are git/crates.io deps, Tailwind can't scan `~/.cargo/` — ship `safelist.html` files with all CSS classes (especially dynamic ones from match arms like `HttpMethod::badge_class()`)

### Key Conventions

- Components use `#[component]` macro with owned prop types (`String`, `Vec`, `Signal`)
- `use_signal()` for local state, `use_context_provider()` for shared state
- Syntax highlighting: `dioxus-code`'s `Code` component, behind the `highlight` feature (in `default`); with the feature off, code blocks render as escaped plain text
- Cargo features (docs-kit): `default = ["web", "mermaid", "highlight", "openapi", lang-*]` where the default `lang-*` set is bash, css, dockerfile, html, javascript, json, markdown, python, toml, typescript, yaml (`lang-c-sharp`/`lang-cpp`/`lang-tsx` exist but are off — C#+C++ were ~8 MB of wasm, TSX 1.5 MB; Rust is always highlighted). The example app's own `default` list is trimmed to what `docs/` fences: bash, css, json, python, typescript. Plus `server` (SeoRouter/Axum routes). `openapi` gates the API-reference rendering path only — the spec parser lives in the build crate. The workspace `dioxus` dep is `default-features = false`; the example enables `launch`/`devtools`/`logger` itself
- Cargo features (dioxus-mdx): `components` gates `src/components/**` and is the *only* thing that pulls `dioxus` in; `parse` gates `src/parser/**` implementation (markdown-rs + regex) while the AST types and `src/text.rs` are always compiled; `openapi` is types + viewer components (no deps), `openapi-parse` adds `openapiv3`/`serde_yaml`. Frontmatter uses `dioxus_mdx::parse_yaml_lite`, not `serde_yaml`
- CI toolchain: Rust 1.96.0, Dioxus CLI 0.7.10, Bun for Tailwind

---

## Dioxus 0.7 Reference

**Dioxus 0.7 changes every API.** `cx`, `Scope`, and `use_state` are gone. Only use this reference.

### Launching

```rust
use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! { "Hello, Dioxus!" }
}
```

### RSX

```rust
rsx! {
    div {
        class: "container",
        color: "red",
        width: if condition { "100%" },
        "Hello, Dioxus!"
    }
    for i in 0..5 {
        div { "{i}" }
    }
    if condition {
        div { "Condition is true!" }
    }
    {children}
    {(0..5).map(|i| rsx! { span { "Item {i}" } })}
}
```

### Assets

```rust
rsx! {
    img { src: asset!("/assets/image.png"), alt: "An image" }
    document::Stylesheet { href: asset!("/assets/styles.css") }
}
```

### Components & Props

- Annotate with `#[component]`, name must start uppercase or contain underscore
- Props must be owned (`String`, `Vec<T>`), implement `PartialEq` + `Clone`
- Wrap in `ReadOnlySignal` for reactive props
- Re-renders when props change or internal reactive state updates

### State

```rust
let mut count = use_signal(|| 0);                    // Local state
let doubled = use_memo(move || count() * 2);          // Memoized derived state
*count.write() += 1;                                  // Mutate (triggers re-render)
count.with_mut(|c| *c += 1);                          // Alternative mutation

// Context API
use_context_provider(|| my_signal);                   // Parent provides
let val = use_context::<Signal<String>>();             // Child consumes
```

### Async

```rust
let data = use_resource(move || async move { /* fetch */ });
match data() {
    Some(value) => rsx! { "{value}" },
    None => rsx! { "Loading..." },
}
```

### Routing

```rust
#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[layout(NavBar)]
        #[route("/")] Home {},
        #[route("/blog/:id")] BlogPost { id: i32 },
}
```

Requires `dioxus = { features = ["router"] }`.

### Server Functions (Fullstack)

```rust
#[post("/api/double/:path/&query")]
async fn double_server(number: i32, path: String, query: i32) -> Result<i32, ServerFnError> {
    Ok(number * 2)
}
```

Requires `dioxus = { features = ["fullstack"] }`. On server: generates endpoint. On client: generates HTTP call.

### Hydration

- Use `use_server_future` instead of `use_resource` for SSR data (serializes result to client)
- Browser-only APIs (e.g. `localStorage`) must go in `use_effect` (runs after hydration)
- Client initial render must match server render exactly
