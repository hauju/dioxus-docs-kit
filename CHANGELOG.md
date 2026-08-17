# Changelog

All notable changes to this project are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions
apply to all three crates (`dioxus-docs-kit`, `dioxus-docs-kit-build`,
`dioxus-mdx`), which are released together from this workspace.

## [Unreleased]

### Added

- **Rustdoc-powered Rust API reference pages.** The kit can now render a
  crate's own public API the way it renders OpenAPI endpoints:
  - `dioxus-docs-kit-build` gains a `distill-rustdoc` binary (and
    `distill_rustdoc` library function) that reduces nightly rustdoc JSON
    (`cargo +nightly rustdoc -- -Z unstable-options --output-format json`,
    megabytes, format-version-locked) to a small committed model JSON. It
    walks public items reachable from the crate root — following re-exports,
    preferring the shortest path — and renders full signatures (generics,
    where clauses, `&mut self`, `impl Trait`), strips intra-doc links to
    their display text, normalizes rustdoc code fences (```` ```rust,ignore ````
    and bare fences become `rust` so samples highlight), and can exclude
    macro-generated noise (`--exclude Props`). Pinned to `rustdoc-types` 0.60
    (rustdoc JSON format 60).
  - `DocsConfig::with_rustdoc(prefix, model_json)` registers a model;
    `.with_rust_api_group_name` (default `"Rust API"`) names the nav group the
    items inject into — same contract as the OpenAPI integration, including
    the startup warning when no nav group matches.
  - Item pages (`dioxus-mdx`'s `RustApiItemPage`) show a kind badge, module
    breadcrumb, highlighted declaration with copy button, doc comments
    rendered as markdown, implemented traits, and member sections (variants,
    fields, methods, associated items). The sidebar groups items by kind with
    one-letter chips; items join search, the sitemap, and prev/next
    navigation.
  - The example site documents the kit's own API under a new "Rust API" tab
    (`just api` regenerates the committed model).

## [0.6.1] — 2026-08-16

### Added

- **Precompiled stylesheet — a Tailwind toolchain is now optional.**
  `dioxus-docs-kit` ships a compiled `docs-kit.css` asset exposed as
  `dioxus_docs_kit::DOCS_KIT_CSS`. It covers every class the kit's and
  `dioxus-mdx`'s components emit (Tailwind utilities, DaisyUI dark/light
  themes, typography prose, and the `--dk-*` token surface from `theme.css`),
  so a consumer can link one stylesheet instead of setting up Bun + Tailwind +
  the safelist copy. The safelist path remains the right choice for apps that
  use their own Tailwind classes. The sheet is rebuilt with `just css`; CI
  fails if the committed output is stale.

## [0.6.0] — 2026-08-12

### Fixed

- **Parser crashes on malformed or non-English input.** Three separate
  crash-class bugs in `dioxus-mdx`, each with a regression test:
  - Non-ASCII prose before the first child tag panicked (`start byte index 1 is
    not a char boundary`) in the Tabs, Accordion and ResponseField scan loops,
    which advanced by a raw byte.
  - An unclosed `<Card` made the Card scan loop reassign its cursor to itself
    and spin forever, hanging the build script or SSR worker.
  - A self-referential OpenAPI schema (`Node.children -> [Node]`) overflowed
    the stack: `$ref` resolution inlined eagerly with no cycle guard. References
    already being expanded now resolve to a name-only stub.
- **Content silently dropped from fenced code blocks.** A docs framework's docs
  are full of samples showing its own components, and four scanners tore them
  apart:
  - Component tags inside a fence were parsed as real components, so a ```` ```mdx ````
    block containing `<Card>` rendered as a card plus two orphan fence lines.
  - `import` lines were stripped from JS/TS/Python samples.
  - `<CodeGroup>` used `\s+` for the filename separator, so a fence with no
    language rendered its first code line as the tab label with an empty body.
  - `parse_document` extracted frontmatter and then called `parse_mdx`, which
    extracted it again — a body starting with a thematic break lost everything
    up to the next `---`.
- **TOC entries for headings inside code fences.** `extract_headers` regexed
  raw markdown, so a `## Setup` inside a sample became an entry linking to an
  anchor the renderer never emits.
- **`~~~` fences were not treated as code at all.** Fence handling was
  backtick-only in the body parser, so a `~~~` block had its `import` lines
  stripped, its component tags parsed as real components, and never became a
  `CodeBlock` — while `toc.rs` and the search splitter *did* skip tilde fences,
  so the same page behaved inconsistently across surfaces. Both remaining
  fence regexes are replaced by one shared line-based scanner
  (`find_fenced_blocks`) that handles ``` and `~~~` with CommonMark's
  same-marker-closes rule: the other marker inside a fence is literal content.
  `~~~` blocks now render as code blocks (with language and filename tabs),
  including inside `<CodeGroup>` / `<RequestExample>` / `<ResponseExample>`,
  where they previously vanished silently.
- **Invalid XML in RSS and sitemaps.** Titles, descriptions and URLs were
  interpolated unescaped; a post titled `Rust & WASM` emitted a bare `&`, which
  makes readers reject the whole feed rather than the single item.
- **Opening tags matched by prefix.** `<Tabs` counted as an opening `<Tab` (and
  `<CardGroup` as a `<Card`) while the closing tags never collide, so nesting
  depth never rebalanced: nested Tabs lost the outer tab entirely, and a Card
  wrapping a CardGroup leaked its raw tags onto the page as visible text.
- **`extract_attr` matched attribute-name suffixes** — asking for `title` on
  `<Card subtitle="Sub" title="Real">` returned `"Sub"`.
- **Wrong sidebar in server-rendered HTML.** The active tab was seeded to
  `tabs[0]` and corrected only by an effect, which does not run during SSR, so
  a crawler requesting an API-reference URL got the Docs sidebar and the tab
  visibly flipped after hydration.

### Added

- **`use_docs_context(path, base_path, navigate)`** — builds a `DocsContext`
  from the current path as a plain `String` and rewraps it reactively inside
  the hook. Both READMEs previously taught
  `use_memo(move || match route { .. })`, which reads no reactive source
  (`use_route` returns a plain value, not a signal): the memo ran once and
  never again, freezing the sidebar highlight, tab sync and drawer auto-close
  on the first page visited. Passing the path by value makes that mistake
  impossible. `DocsContext::new` is unchanged for callers that hold a signal.
- `[package.metadata.docs.rs] all-features` on `dioxus-docs-kit` and
  `dioxus-mdx`. The `server` module (`SeoRouter`) was a 404 on docs.rs for
  0.5.0 because it only builds under a non-default feature.
- CI now runs an MSRV job and `cargo test --workspace --all-features` (the
  `server` module is cfg-gated, so its tests had never run in CI).

### Changed

- **MSRV is now 1.88** (was declared 1.85). The declaration was already false:
  `cargo +1.85 check -p dioxus-mdx` fails with five `E0658` errors — the crates
  use let-chains, which need 1.88. Nothing caught it because every CI job pins
  a much newer toolchain.
- **Search results are capped at 25** before hits are built. The query re-runs
  on every keystroke and each hit costs a snippet scan plus a mounted
  component, so a broad query on a mid-size site built hundreds of hits into a
  modal that shows about six.
- **Parsing is ~7x faster** (244ms → 34ms over this repo's own docs corpus,
  release build). `extract_attr` called `Regex::new` on every attribute lookup;
  regex compilation dominated parse time.
- A zero-line code block (opening fence immediately followed by the closing
  fence) inside `<CodeGroup>` is no longer matched, aligning it with the
  top-level fence parser, which has behaved this way since 0.5.0.

### Migration notes

Nothing was removed from the public API, so most consumers only bump the
version. Three things can still bite:

- **Rust 1.88 is now required** (0.5.0 declared 1.85). Below it the build fails
  inside a dependency with an unrelated-looking `E0658` about `let` expressions,
  which is confusing if you don't know the MSRV moved. Bump your toolchain, or
  stay on 0.5.0 — which, despite its declaration, never actually built on 1.85
  either.
- **Your rendered docs may change**, always in the direction of "the sample is
  now treated as a sample":
  - Component tags inside a fenced block (```` ```mdx ```` showing `<Card>`)
    render as code instead of becoming a real component.
  - `import` lines inside JS/TS/Python samples are no longer stripped.
  - `~~~` blocks become code blocks instead of raw markdown, and are extracted
    inside `<CodeGroup>` / `<RequestExample>` / `<ResponseExample>` instead of
    vanishing.
  - Headings inside fences no longer appear in the table of contents.

  If a page relied on the old behavior — e.g. an unfenced-looking component you
  wanted rendered — it needs the fence removed.
- **`DocsContext::new` is unchanged and still supported.** The new
  [`use_docs_context`] hook is additive, not a replacement. Migrating is
  recommended but optional: it takes the path as a plain `String` and wraps it
  reactively for you, which removes the frozen-sidebar failure mode that the
  0.5.0 READMEs taught (`use_memo(move || match route { .. })` reads no
  reactive source, so it runs once and never updates). If your layout already
  uses `use_memo(use_reactive!(|route| ..))`, it is correct as-is.

Also worth knowing: `safelist.html` lives at the **root** of the published
crate. The 0.5.0 root README documented a `crates/dioxus-docs-kit/` path that
only exists in a git checkout — fixed in this release.

[`use_docs_context`]: https://docs.rs/dioxus-docs-kit/latest/dioxus_docs_kit/hooks/fn.use_docs_context.html

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
