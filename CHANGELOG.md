# Changelog

All notable changes to this project are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions
apply to all three crates (`dioxus-docs-kit`, `dioxus-docs-kit-build`,
`dioxus-mdx`), which are released together from this workspace.

## [0.5.0] — 2026-07-19

### Added

- **`CopyPageButton`**: a "Copy page" button rendered in the header of every
  MDX doc page that copies the page's raw Markdown to the clipboard (the "copy
  page for LLMs" pattern), showing a "Copied" state for ~2s. Not rendered on
  OpenAPI endpoint pages. Carries the stable `dk-copy-page` class for theming.
- **Ranked, section-level docs search**: the index now splits each MDX page
  into sections on its h2–h4 headings, so results deep-link to the matching
  section (scroll-to-anchor over a few animation frames after navigation) and
  show the section heading beneath its page title. Matching is multi-term AND
  (every whitespace-separated term must hit a field), scored
  title > heading > description > body with a word-boundary/prefix bonus and an
  earlier-position tiebreak (equal scores keep nav order). Each result row
  renders a snippet with the matched terms highlighted via `<mark>` (no
  `dangerous_inner_html`). Blog search shares the same ranking (post-level,
  with snippets). New `dk-search-context` / `dk-search-snippet` /
  `dk-search-mark` hooks added to `safelist.html`.
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
- **Build-time internal link validation** (`dioxus-docs-kit-build`):
  `generate_content_map` now scans every `.mdx` for markdown links and emits
  `cargo:warning` for broken internal targets and missing heading anchors
  (anchors use the same slug algorithm as the renderer). External links,
  images, and fenced code blocks are ignored; runtime OpenAPI reference pages
  are not flagged. Warnings only — link problems never fail the build.
- **Fail-fast frontmatter validation** (`dioxus-docs-kit-build`): malformed
  blog frontmatter — bad YAML, a missing `title`/`date`/`author`, or a
  wrong-typed optional field (e.g. `tags: rust` instead of a sequence) — now
  fails the build with the file path and serde line/column instead of
  silently dropping the post at runtime. Docs pages whose leading `---` block
  is not parseable frontmatter get a `cargo:warning` (the runtime renders
  such blocks as page content, so they never fail the build).
- **`highlight` feature** (default, both `dioxus-docs-kit` and `dioxus-mdx`):
  gates the `dioxus-code` syntax-highlighting dependency. Leaving it enabled
  keeps colored code blocks exactly as before. Disabling it
  (`default-features = false`) drops `dioxus-code` and its `arborium-*`
  tree-sitter grammar crates entirely — no C toolchain (or wasm `stderr`
  linker shim) required for wasm builds and a smaller binary. Code blocks
  still render, as escaped plain text inside the same markup, so layout,
  scrolling, copy buttons, and CodeGroup tabs keep working; only token
  coloring is lost.

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
  text. (Blog posts with invalid frontmatter now fail the build outright —
  see the fail-fast frontmatter bullet under Added; the runtime `tracing`
  warning remains only as a fallback for content maps built without
  `dioxus-docs-kit-build`.)
- Heading anchor ids now agree everywhere for headings containing `&`, `<`,
  `>`, or markdown links: `slugify` decodes standard HTML entities and
  reduces link syntax to its text, so the renderer's injected ids, TOC links,
  search deep-links, and build-time anchor checks all produce the same slug
  (previously a `## Tips & Tricks` heading got the DOM id `tips-amp-tricks`
  while the TOC linked to `#tips-tricks` — TOC anchors for such headings were
  broken).
- `~~~`-fenced code blocks are now skipped by the search section splitter and
  snippet cleaner, matching the ``` handling (a `##` line inside a tilde
  fence no longer produces a phantom search section).
- TOC scroll listeners and the Cmd/Ctrl-K keydown listener no longer
  accumulate across navigations/layout remounts.
- The blog search modal now shares the docs modal shell and carries the
  stable `dk-search-*` classes, so theme presets style both.

### Changed

- Search fields are lowercased once at build time (never re-lowercased per
  keystroke), and indexed body text is lightly stripped of markdown noise
  (fenced code dropped, links reduced to their text, markers removed).
- `SearchEntry` reshaped for section-level results: `content_preview` → `body`,
  with new `anchor` (rendered heading id) and `heading` fields.
  `BlogSearchEntry`: `content_preview` → `body`, now the full cleaned post text
  rather than a 200-char preview. Both gained precomputed `*_lower` fields.
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
- `DocsContext` / `BlogContext` are now `#[non_exhaustive]`: external crates
  can no longer build them with struct literals and must use the `::new()`
  constructors plus `with_*` setters, so future fields stay non-breaking.
- Workspace-level dependency and package metadata management
  (`[workspace.package]` / `[workspace.dependencies]`).

### Migration notes

- **wasm `stderr` shim**: the kit now defines the wasm32 `stderr` symbol in
  the library. Downstream binaries that added their own
  `#[no_mangle] static mut stderr` workaround must **delete it** — keeping
  both causes a duplicate-symbol link error.
- Construct `DocsContext` / `BlogContext` via the `::new()` constructors and
  `with_*` setters. The structs are now `#[non_exhaustive]`, so struct literals
  no longer compile from outside the kit — replace any
  `DocsContext { .. }` / `BlogContext { .. }` with
  `DocsContext::new(current_path, base_path, navigate).with_site_url(..)`
  (and `.with_auto_meta(..)` / `.with_markdown_alternate(..)` as needed).
- If you provided the search-open signal as a bare `Signal<bool>` context,
  provide `SearchOpen(signal)` instead.
- If you called `generate_llms_txt`/`generate_llms_full_txt` directly, pass
  the docs base URL (`{site_url}/docs`) — or use `SeoRouter`, which wires all
  crawler endpoints for you.
- If you build with `default-features = false`, add the `highlight` feature to
  keep colored code blocks and the `dioxus-code` re-exports (`Code`,
  `CodeTheme`, `Language`, `SourceCode`, `Theme`, `CodeThemeOverride`,
  `CodeThemeConfig`, and `DocsConfig::with_code_theme[s]`); omit it to drop the
  `dioxus-code` dependency and render plain (uncolored) code blocks.
- If you read `search_docs()` / `search_posts()` results directly, rename the
  `content_preview` field to `body`; `SearchEntry` also gained `anchor` and
  `heading` (empty for page-level / intro / API entries).

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
