# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Dev Commands

```sh
dx serve                    # Dev server with hot reload
dx build --release          # Production build
dx bundle --web --release   # Bundle for deployment
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
| `dioxus-docs-kit-build` | `crates/dioxus-docs-kit-build/` | Build-time helper: reads `_nav.json` → generates `include_str!()` content map |
| `dioxus-mdx` | `crates/dioxus-mdx/` | Standalone MDX parser + renderer (Mintlify-style components) |

**Dependency direction:** `dioxus-docs-kit` depends on `dioxus-mdx`. The build crate is independent. The mdx and docs-kit crates use `dioxus = { features = ["lib"] }` (NOT fullstack). Only the root example uses fullstack.

### Content Pipeline

1. `build.rs` calls `dioxus_docs_kit_build::generate_content_map("docs/_nav.json")`
2. Build script reads `_nav.json`, generates `$OUT_DIR/doc_content_generated.rs` with `include_str!()` for each `.mdx` file
3. `doc_content_map!()` macro in `main.rs` includes the generated file as a `HashMap<&str, &str>`
4. `DocsConfig::new(nav_json, content_map).with_openapi(prefix, yaml).build()` creates a `DocsRegistry` (parses all docs, builds search index)
5. `DocsPageContent` checks `registry.get_api_operation(&path)` first, then falls back to `registry.get_parsed_doc(&path)`

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

- **`DocsConfig`** — Builder: `.new(nav_json, content_map)` → `.with_openapi()` → `.with_theme_toggle()` → `.with_default_path()` → `.build()`
- **`DocsRegistry`** — Holds parsed docs, nav config, search index, OpenAPI specs. Key methods: `get_parsed_doc()`, `search_docs()`, `get_api_operation()`, `get_api_sidebar_entries()`, `tab_for_path()`, `generate_llms_txt()`, `generate_llms_full_txt()`
- **`DocsContext`** — Route decoupling bridge (`current_path`, `base_path`, `navigate` callback). Consumer provides this so library components don't depend on the consumer's Route enum
- **`use_docs_providers(registry, docs_ctx)`** → returns `DocsProviders { search_open, drawer_open }` for use in custom headers
- **UI components** — `DocsLayout`, `DocsPageContent`, `DocsSidebar`, `SearchModal`, `SearchButton`, `DocsPageNav`, `MobileDrawer`, `ThemeToggle`
- **`doc_content_map!()`** — Macro that generates `fn doc_content_map() -> HashMap<&'static str, &'static str>` from build script output

### Styling

- **Tailwind CSS 4** + **DaisyUI 5** (dark theme default, light theme available)
- `@tailwindcss/typography` for prose content
- Input: `tailwind.css` → processed by `@tailwindcss/cli`
- Icons: `dioxus-free-icons` with `lucide` feature
- **Safelist pattern**: when crates are git/crates.io deps, Tailwind can't scan `~/.cargo/` — ship `safelist.html` files with all CSS classes (especially dynamic ones from match arms like `HttpMethod::badge_class()`)

### Key Conventions

- Components use `#[component]` macro with owned prop types (`String`, `Vec`, `Signal`)
- `use_signal()` for local state, `use_context_provider()` for shared state
- Syntax highlighting: `highlight_code(code, Some("lang"))` returns HTML for `dangerous_inner_html`
- Cargo features: `default = ["web"]`, `server = ["dioxus/server"]`
- CI toolchain: Rust 1.91.0, Dioxus CLI 0.7.3, Bun for Tailwind

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
