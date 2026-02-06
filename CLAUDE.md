# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Serve

```sh
dx serve              # Dev server with hot reload (web)
dx build --release    # Production build
```

Requires the Dioxus CLI (`dx`): `curl -sSL http://dioxus.dev/install.sh | sh`

## Project Architecture

**Dioxus 0.7 fullstack documentation site** (package name: `dioxus-docs-kit-example`). Renders MDX docs, OpenAPI API reference pages, and a search modal.

### Crate Layout

- `src/main.rs` — Routes, app-specific pages (Home, Blog, Navbar), plus ~50 lines of docs glue code
- `crates/dioxus-docs-kit/` — **Reusable docs site shell** (layout, sidebar, search, page nav, content registry)
- `crates/dioxus-mdx/` — MDX parser/renderer subcrate (uses `dioxus = { features = ["lib"] }`, NOT fullstack)
- `build.rs` — Reads `docs/_nav.json`, generates `doc_content_generated.rs` with `include_str!()` calls for all `.mdx` files

### Content Pipeline

All doc content is **embedded at compile time**:
1. `build.rs` reads `docs/_nav.json` → generates HashMap of path→content via `include_str!()`
2. `main.rs` creates a `DocsRegistry` via `DocsConfig` builder (parses all docs, builds search index, parses OpenAPI specs)
3. OpenAPI spec at `docs/api-reference/petstore.yaml` passed to the registry via `.with_openapi()`

When adding a new doc page: create `docs/<group>/<slug>.mdx` and add the path to `docs/_nav.json`.

### dioxus-docs-kit Crate

Reusable documentation site shell. Key types:
- **`DocsConfig`** — Builder: `DocsConfig::new(nav_json, content_map).with_openapi(prefix, yaml).build()`
- **`DocsRegistry`** — Holds parsed docs, nav config, search index, OpenAPI specs. Methods: `get_parsed_doc()`, `search_docs()`, `get_api_operation()`, `get_api_sidebar_entries()`, `get_sidebar_title()`, `tab_for_path()`
- **`DocsContext`** — Route decoupling bridge (current_path, base_path, navigate callback). Consumer provides via `use_context_provider`
- **UI components** — `DocsLayout`, `DocsSidebar`, `DocsPageContent`, `SearchModal`, `DocsPageNav`, `MobileDrawer`, `SearchButton`

### Routing

```
Route enum (main.rs):
  #[layout(Navbar)]        → Home (/), Blog (/blog/:id)
  #[layout(MyDocsLayout)]  → DocsIndex (/docs), DocsPage (/docs/:..slug)
```

- `MyDocsLayout` wires `DocsContext` + `DocsRegistry` into the library's `DocsLayout`
- `/docs/:..slug` is a catch-all; slug is `Vec<String>` joined with `/` to resolve content
- `DocsPageContent` checks `registry.get_api_operation(&path)` first for API endpoints, then falls back to `registry.get_parsed_doc(&path)`
- API endpoint slugs are kebab-cased from camelCase operationIds

### Styling

- **Tailwind CSS 4** + **DaisyUI 5** (dark theme default)
- `@tailwindcss/typography` for prose content
- Input: `tailwind.css` → processed automatically by Dioxus CLI
- Icons: `dioxus-free-icons` with `lucide` feature

### Key Conventions

- Components use `#[component]` macro with owned prop types (String, Vec, Signal)
- Context API shares state: `search_open`, `active_tab` signals managed by `DocsLayout`; `DocsContext` + `DocsRegistry` provided by consumer wrapper
- Syntax highlighting: `highlight_code(code, Some("lang"))` returns HTML for `dangerous_inner_html`
- Main crate uses `dioxus = { features = ["router", "fullstack"] }`; the docs-site and mdx subcrates use `features = ["lib"]`
- Cargo features: `default = ["web"]`, `server = ["dioxus/server", "dep:mongodb"]`

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
