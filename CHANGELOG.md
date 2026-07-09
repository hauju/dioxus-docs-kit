# Changelog

All notable changes to this project are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions
apply to all three crates (`dioxus-docs-kit`, `dioxus-docs-kit-build`,
`dioxus-mdx`), which are released together from this workspace.

## [0.5.0] — Unreleased

### Added

- **`server` feature with `SeoRouter`** (`dioxus_docs_kit::server`): one
  builder generates all crawler-facing Axum routes — per-page raw Markdown
  (`<page>.md`), `/llms.txt`, `/llms-full.txt`, per-surface sitemaps plus a
  `/sitemap.xml` index, blog RSS, and a robots.txt with explicit AI-crawler
  entries. These are plain routes with correct content types (server functions
  would JSON-encode the bodies).
- `DocsContext::new()` / `BlogContext::new()` constructors with defaulted meta
  fields, plus `.with_site_url()`, `.with_auto_meta()`,
  `.with_markdown_alternate()` setters.
- `DocsConfig::try_build()` / `BlogConfig::try_build()` returning
  `Result<_, DocsKitError>`; `build()` now panics with the underlying parse
  error (including serde line/column detail) instead of a generic message.
- `DocsRegistry::get_api_operation_with_spec()` — resolves an operation
  together with the spec that owns it.
- `use_theme_provider()` public hook — theme persistence + `CurrentTheme`
  context for navbars outside `DocsLayout`/`BlogLayout`.
- `SearchOpen` / `ActiveTab` / `ActiveTag` / `CurrentPage` newtype context
  keys (previously bare `Signal<...>` values that could collide with consumer
  contexts).
- `rust-version = "1.85"` declared; per-crate LICENSE files included in
  published packages.

### Fixed

- **Multi-spec OpenAPI**: sidebar links and endpoint pages previously always
  used the *first* registered spec — wrong URLs and server lists for every
  spec after it. Entries now carry their owning prefix and pages render
  against the owning spec.
- **Windows builds** (`dioxus-docs-kit-build`): backslashes in
  `CARGO_MANIFEST_DIR` produced invalid `include_str!` literals; paths are now
  normalized. Missing `.mdx` files also emit `rerun-if-changed` so creating
  the file triggers a rebuild, and the warning names the full expected path.
- Malformed MDX frontmatter no longer renders the raw `---` block as page
  text; blog posts with invalid frontmatter now log a `tracing` warning
  instead of silently disappearing from the site.
- TOC scroll listeners and the Cmd/Ctrl-K keydown listener no longer
  accumulate across navigations/layout remounts.
- The blog search modal now shares the docs modal shell and carries the
  stable `dk-search-*` classes, so theme presets style both.

### Changed

- Performance: all parser/renderer regexes are compiled once
  (`LazyLock`), API sidebar entries are precomputed, and operation lookup is
  O(1) via a path index (previously per-render rebuilds and linear scans with
  per-call allocations).
- `generate_llms_txt` / `generate_llms_full_txt` now take the docs base URL
  (e.g. `https://site.com/docs`) instead of a site root with `/docs` appended
  internally.
- `get_api_sidebar_entries()` returns a slice of precomputed entries;
  `ApiEndpointEntry` gained a `prefix` field.
- `extract_blog_frontmatter` returns `Result<_, String>` (was `Option`).
- Workspace-level dependency and package metadata management
  (`[workspace.package]` / `[workspace.dependencies]`).

### Migration notes

- **wasm `stderr` shim**: the kit now defines the wasm32 `stderr` symbol in
  the library. Downstream binaries that added their own
  `#[no_mangle] static mut stderr` workaround must **delete it** — keeping
  both causes a duplicate-symbol link error.
- Construct `DocsContext` / `BlogContext` via the new constructors; struct
  literals still compile today but require every field and will break when
  fields are added.
- If you provided the search-open signal as a bare `Signal<bool>` context,
  provide `SearchOpen(signal)` instead.
- If you called `generate_llms_txt`/`generate_llms_full_txt` directly, pass
  the docs base URL (`{site_url}/docs`) — or use `SeoRouter`, which wires all
  crawler endpoints for you.

## [0.4.x] — 2026-03/04

Published to crates.io as `dioxus-docs-kit` 0.4.0/0.4.1 and
`dioxus-docs-kit-build` 0.4.0: `use_docs_providers` one-call context setup,
tabbed navigation, blog engine polish, SEO meta groundwork.

## [0.3.x] — 2026-02

Published as `dioxus-mdx` 0.3.0/0.3.1: CSS-class syntax highlighting via
`dioxus-code`, duplicate-H1 stripping, parser fixes.

## [0.2.0] — 2026-01

First public release: MDX docs shell, sidebar navigation from `_nav.json`,
full-text search, OpenAPI reference pages, mobile drawer, theme toggle.
