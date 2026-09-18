# Changelog

All notable changes to this project are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions
apply to all four crates (`dioxus-docs-kit`, `dioxus-docs-kit-build`,
`dioxus-mdx`, `hl-lite`), which are released together from this workspace.

## [0.9.0] — 2026-09-18

### Changed

- **Breaking — syntax highlighting is `hl-lite`, not tree-sitter.** Code blocks
  are lexed by the new `hl-lite` workspace crate: twelve hand-written byte-level
  lexers (Rust, Bash, CSS, Dockerfile, HTML, JavaScript, JSON, Markdown, Python,
  TOML, TypeScript, YAML) with **no dependencies at all**, no regex engine and no
  C. It replaces `dioxus-code`/arborium, whose grammars were ~6 MB of wasm and
  ~73 CPU-seconds of cold build for five languages; all twelve of these cost
  about 48 KB (19 KB brotli) and compile in half a second. Highlighting is now
  always on — there is nothing to enable and no grammar to pick — and a wasm
  build needs no C toolchain, which also retires the `stderr` link shim and the
  `CC_wasm32_unknown_unknown` note in `.cargo/config.toml`. This site's release
  wasm went from **7.19 MB raw / 1.36 MB brotli** (five grammars) to
  **1.48 MB raw / 424 KB brotli**.

- **Breaking — token colors are CSS.** A block renders as
  `<pre class="dk-code">` with one `<span class="hl-{kind}">` per classified
  token (`hl-keyword`, `hl-string`, `hl-comment`, `hl-number`, `hl-type`,
  `hl-function`, `hl-attribute`, `hl-property`, `hl-tag`, `hl-operator`,
  `hl-punctuation`, `hl-constant`, `hl-variable`). `theme.css` defines a
  `--dk-hl-*` custom property per kind as a `light-dark()` pair (GitHub Light /
  Tokyo Night) that follows the active theme's `color-scheme`, so code follows
  the site's light/dark toggle with no Rust involved. The `<pre>`'s class is
  `dk-code`, not `dxc`.

- **Breaking — all document parsing moves to build time.** `dioxus-docs-kit-build`
  now parses every `.mdx` page and OpenAPI spec, renders each page's prose to
  HTML, and precomputes the whole search index into a single JSON bundle
  (`$OUT_DIR/docs_bundle.json`, `$OUT_DIR/blog_bundle.json`). The app embeds the
  bundle with `docs_bundle!()` / `blog_bundle!()` and the registry deserializes
  it once, on first use, instead of re-parsing every document on load.

  The wasm client therefore links **no MDX parser, no `markdown` (markdown-rs),
  no `regex`/`regex-lite`, no `openapiv3` and no `serde_yaml`/`unsafe-libyaml`**.
  Measured on a 3-page template consumer (kit `web` only, `wasm-release` +
  `wasm-opt -Oz`): **1,492,487 → 1,277,705 bytes** optimized
  (**443,277 → 362,249** brotli, −18%). The kit's cost on top of a bare Dioxus
  router app drops from 257 KB to 176 KB brotli. The bundle itself is larger
  than the raw MDX it replaces (16.8 KB vs 9.6 KB for those 3 pages), which is
  already included in those figures.

- **Breaking — `dioxus-mdx` gains `components` and `parse` features, both in
  `default`.** `components` gates `src/components/**` and is the only thing that
  pulls `dioxus` in; `parse` gates the parser and its `markdown` + regex
  dependencies. `default-features = false` now builds a plain Rust library of the
  document types with no Dioxus at all, which is how the build crate uses it.
- **Breaking — `dioxus-mdx`'s `openapi` feature is split.** `openapi` is now the
  value types plus the viewer components (no dependencies, always available);
  `openapi-parse` (in `default`) is the spec parser and pulls `openapiv3` +
  `serde_yaml`. `dioxus-docs-kit`'s `openapi` feature is unchanged in name but
  now only gates the API-reference rendering path.
- **Breaking — the parsed AST carries HTML, not Markdown.** `DocNode::Markdown`
  is now `DocNode::Html`, and the `content` field of `CalloutNode`, `CardNode`
  and `ResponseFieldNode` is now `content_html`. All of them hold HTML rendered
  at build time. `ParsedDoc::raw_markdown` is unchanged and still carries the
  Markdown source (`llms.txt`, the copy-page button and the table of contents
  read it).
- **Breaking — `DocsKitError` variants changed**: `NavParse`, `BlogManifestParse`
  and `OpenApi` are replaced by `DocsBundleParse`, `BlogBundleParse` and
  `BundleVersion`. Nav and spec errors are now build-script failures.
- The bundle carries a format version and the registry rejects one it does not
  understand, so a mismatched `dioxus-docs-kit` / `dioxus-docs-kit-build` pair
  fails with a message naming the problem instead of a field-by-field parse error.
- **Dropped the `dioxus-free-icons` dependency.** The 86 Lucide icons the shell
  renders are vendored as inline SVG in `dioxus-mdx`'s new `lucide` module
  (`Icon { class, icon: LdX }`, re-exported as `dioxus_docs_kit::lucide`); the
  `<svg>` carries the same attributes and path data as before, so nothing about
  the rendering or the CSS classes changes. `dioxus-free-icons`
  0.10.0 was the slowest crate in a cold wasm build (55k LOC, ~1,456 `rsx!`
  expansions for the ~86 icons used) and, because its manifest requests
  `dioxus` without `default-features = false`, it re-enabled dioxus's `launch`,
  `logger` and `devtools` in every consumer — pulling `dioxus-logger`,
  `tracing-subscriber`, `regex-automata`, `sharded-slab` and `matchers` back
  into builds that had opted out in 0.7.0. Those crates are now genuinely
  absent. The kit never re-exported `dioxus_free_icons`, so its own API is
  unchanged; anything that reached the crate transitively through the kit must
  now depend on it directly or switch to `dioxus_docs_kit::lucide`.

### Removed

- The `highlight` feature and every `lang-*` feature, from `dioxus-mdx`,
  `dioxus-docs-kit` and the example app, along with the `dioxus-code` and
  `regex` dependencies. `dioxus-mdx`'s `components` feature now pulls `hl-lite`.
- `DocsConfig::with_code_theme`, `with_code_themes`, `CodeThemeConfig`,
  `DocsRegistry::code_theme` and `dioxus_mdx::CodeThemeOverride` — colors are
  the `--dk-hl-*` CSS tokens now.
- The `dioxus_docs_kit::{Code, CodeTheme, Language, SourceCode, Theme}`
  re-exports (and `dioxus_mdx::{CodeTheme, Theme}`). `dioxus_docs_kit::hl` /
  `dioxus_mdx::hl` re-export the highlighter itself instead.
- `dioxus_docs_kit_build::generate_content_map`, `generate_content_map_with_validation`,
  `generate_blog_content_map` and `generate_blog_content_map_with_validation`.
- `dioxus_docs_kit::doc_content_map!()` and `blog_content_map!()`.
- `DocsConfig::with_openapi` (moved to `DocsBuild::with_openapi` in `build.rs`).
- `dioxus_mdx::get_raw_markdown` (it only made sense on an unrendered tree;
  `parse_document` / the new `parse_body` return the Markdown source alongside
  the nodes).
- `dioxus_docs_kit::blog::types::{extract_blog_frontmatter, calculate_reading_time}`
  and `BlogManifest` — blog frontmatter and reading time are computed at build time.

### Added

- `hl-lite` 0.9.0, published from this workspace:
  `highlight(Lang, &str) -> Vec<Span>`, `Lang::from_slug` / `Lang::from_path`,
  and `Kind::class`. The spans always concatenate back to the input, never split
  a `char`, and run to the end of the input on an unterminated construct.
- `dioxus_docs_kit::{CodeBlockNode, DocCodeBlock}` re-exports, so code outside a
  docs page renders through the same component.
- `dioxus_docs_kit_build::{DocsBuild, BlogBuild}` builders, plus
  `docs_bundle_json` / `blog_bundle_json` for content that does not come from the
  filesystem.
- `dioxus_docs_kit::{docs_bundle!, blog_bundle!}` macros.
- `dioxus_mdx::parse_body`, `to_html`, `to_html_with_heading_ids`, and
  `parse_atx_heading` / `strip_markdown_links` alongside the existing `slugify`
  and `extract_headers` (all free of Dioxus and of any regex engine, so the build
  crate and the browser agree on anchor ids by construction).
- `Serialize`/`Deserialize` on the whole document AST (`ParsedDoc`,
  `DocFrontmatter`, `DocNode` and every node type) and the OpenAPI value types.

### Fixed

- Fenced code blocks indented inside a component (`<Step>`, `<Tab>`, `<Accordion>`, ...)
  are dedented by the opening fence's own indentation, as CommonMark specifies. Only
  the first line used to lose its indent, so every following line rendered shifted
  right by the component's nesting depth.
- **Mermaid diagrams only survived the first page load.** Each `<MermaidDiagram>`
  took its element id from a process-wide counter, which the server advances once
  per request while the wasm client starts again at zero — from the second load
  on, the client looked up `mermaid-0` while the hydrated markup said `mermaid-2`
  and no diagram was drawn. The component now renders an id-less
  `<pre class="mermaid">` and the client runs mermaid over
  `pre.mermaid:not([data-processed])`, stashing each block's source in
  `data-mermaid-src` so a theme switch can restore and redraw it. One shared
  `MutationObserver` on `data-theme` now handles the whole page instead of one
  per diagram.
- **"On this page" links to `<Update>`, `<Accordion>` and `<Tab>` titles were
  dead.** The table of contents synthesises a heading for each of them, but the
  rendered title carried no `id`, so the anchor resolved to nothing. All three
  now emit `id="{slugify(title)}"`.
- **Unmapped `icon="…"` names rendered a blank circle.** `file-text`, `layout`,
  `code-2`, `list-ordered`, `message-circle`, `panel-left` and `user` (plus
  aliases) now map to real Lucide glyphs, and a test asserts that every name the
  example content uses resolves to something other than the fallback.
- A long `<Update label="…">` (`v0.3.x - v0.4.x`) wrapped to two lines inside
  DaisyUI's fixed-height badge and spilled out of the pill.
- **No horizontal scroll at 390px.** The navbar brand truncates instead of
  pushing the toggles off-screen, prose inline code breaks with
  `overflow-wrap: anywhere`, and prose tables scroll horizontally inside the
  article instead of widening the page.
- The code-block copy button and the mobile navigation toggle carry
  `aria-label`s, so they announce with a name instead of as unlabelled buttons.
- `dioxus-mdx` depends on `web-sys` with the `Location` feature under its `web`
  feature. `dioxus-web`'s history implementation calls `window.location()` but
  only requests `web-sys/Location` from its own `devtools` feature, so without
  this a `dioxus/web` build that leaves `devtools` off fails to compile
  `dioxus-web`. Until now `dioxus-free-icons` masked the bug by re-enabling
  dioxus's defaults.

### Migration

`build.rs` — parse the content instead of listing it:

```diff
 fn main() {
-    dioxus_docs_kit_build::generate_content_map("docs/_nav.json");
-    dioxus_docs_kit_build::generate_blog_content_map("blog/_blog.json");
+    dioxus_docs_kit_build::DocsBuild::new("docs/_nav.json")
+        // the spec's PATH now, parsed here instead of in the browser
+        .with_openapi("api-reference", "docs/api-reference/petstore.yaml")
+        .generate();
+    dioxus_docs_kit_build::BlogBuild::new("blog/_blog.json").generate();
 }
```

With strict validation:

```diff
-use dioxus_docs_kit_build::{generate_content_map_with_validation, ValidationMode};
-generate_content_map_with_validation("docs/_nav.json", ValidationMode::Strict);
+use dioxus_docs_kit_build::{DocsBuild, ValidationMode};
+DocsBuild::new("docs/_nav.json")
+    .with_validation(ValidationMode::Strict)
+    .generate();
```

`main.rs` — load the bundle instead of the content map:

```diff
-dioxus_docs_kit::doc_content_map!();
-
 static DOCS: LazyLock<DocsRegistry> = LazyLock::new(|| {
-    DocsConfig::new(include_str!("../docs/_nav.json"), doc_content_map())
-        .with_openapi("api-reference", include_str!("../docs/api-reference/petstore.yaml"))
+    DocsConfig::new(dioxus_docs_kit::docs_bundle!())
         .with_default_path("getting-started/introduction")
         .with_theme_toggle("light", "dark", "dark")
         .build()
 });

-dioxus_docs_kit::blog_content_map!();
-
 static BLOG: LazyLock<BlogRegistry> = LazyLock::new(|| {
-    BlogConfig::new(include_str!("../blog/_blog.json"), blog_content_map())
+    BlogConfig::new(dioxus_docs_kit::blog_bundle!())
         .with_posts_per_page(9)
         .build()
 });
```

`with_theme`, `with_theme_toggle`, `with_default_path`, `with_api_group_name`
and every `DocsRegistry` / `BlogRegistry` query method are unchanged.

Drop `highlight` and every `lang-*` from your manifest — highlighting is always
compiled in now:

```diff
 [features]
-default = ["web", "highlight", "lang-bash", "lang-json"]
+default = ["web"]
-highlight = ["dioxus-docs-kit/highlight"]
-lang-bash = ["dioxus-docs-kit/lang-bash"]
-lang-json = ["dioxus-docs-kit/lang-json"]
```

Replace `with_code_theme[s]` with CSS. There is no Rust code-theme API any
more; override the tokens instead (they already follow your light/dark toggle):

```diff
-DocsConfig::new(docs_bundle!())
-    .with_code_themes(Theme::GITHUB_LIGHT, Theme::TOKYO_NIGHT)
+DocsConfig::new(docs_bundle!())
```

```css
.dk-root {
    --dk-hl-keyword: #cf222e;
    --dk-hl-string:  #0a3069;
    --dk-hl-comment: #6e7781;
}
```

If you rendered code yourself through the `Code` / `SourceCode` re-exports, use
`DocCodeBlock { block: CodeBlockNode { language, code, filename } }`, or call
`dioxus_docs_kit::hl::highlight` and emit your own markup. If you styled the
`.dxc` element, restyle `.dk-code`. Consumers who copied the `wasm_sysroot_stderr`
shim into their own `main.rs` (see 0.5.0) should delete it: nothing references
`stderr` any more.

If you render `DocNode`s yourself, rename `DocNode::Markdown(md)` to
`DocNode::Html(html)` and drop your `markdown::to_html*` call — the string is
already HTML. Same for `CalloutNode`/`CardNode`/`ResponseFieldNode`'s
`content` → `content_html`.

If you used `dioxus-mdx` directly as a parser, depend on it with
`default-features = false, features = ["parse"]` to skip Dioxus entirely.

## [0.8.0] — 2026-09-17

### Added

- `webmcp` feature (off by default): `DocsWebMcp` registers the site's docs as
  [WebMCP](https://github.com/webmachinelearning/webmcp) tools, so an agent
  driving the browser can call `docs_search`, `docs_get_page`, `docs_list_pages`
  and `docs_get_api_operation` instead of scraping the rendered page. The tools are
  read-only and read the search index and content map already compiled into the
  wasm; registration is tied to the component's mount, so leaving the docs
  section unregisters them. Built on `webmcp-rs`. Browsers without native
  WebMCP need Google's polyfill loaded from `index.html` before the wasm
  starts; the README shows the two lines, and the example app has them.

## [0.7.1] — 2026-09-08

### Fixed

- Category URLs percent-encode non-ASCII slugs, so sitemap `<loc>`, canonical
  and Open Graph URLs are valid for tags such as `café`.
- Page metadata is rebuilt after a microtask instead of `requestAnimationFrame`,
  so client navigation in a hidden tab no longer leaves stale head tags.
- The blog build script warns about `_blog.json` `categories` keys that match
  no post tag (strict mode fails), instead of silently ignoring their metadata.

## [0.7.0] — 2026-09-08

### Reliability

- Upgrade Dioxus and the CI Dioxus CLI to 0.7.10.
- Compile the README integration example as a test; fix its missing hook import
  and add a complete app entry point and root redirect.
- Add opt-in strict docs/blog build validation for missing and duplicate pages.
  Docs strict mode also rejects detected broken internal links/anchors and
  unsupported frontmatter. The example site now enables strict validation.
- Use a native modal dialog for docs/blog search, with accessible labels,
  explicit Tab wrapping, focus restoration, and arrow-key result selection.
  Enter opens the selected result; Escape closes from any dialog control.
- Run Chromium search smoke checks against the production bundle in CI.
  Locally, run `just browser-test` against a preview on port 18479.
- Supply the missing system build tools for the optional Local CI Docker runner.

### Fixed

- Keep documentation metadata current across client navigation and remove stale
  canonical URLs, Markdown alternates, and structured data when leaving a page.
- Return HTTP 404 with noindex for missing documentation and blog posts when
  server rendering, sharing the same behavior as missing category pages.
- **The production web bundle shipped unoptimized.** `dx bundle --web --release`
  keeps DWARF by default, which makes wasm-opt abort ("compile unit size was
  incorrect") and dx silently fall back to the raw wasm-bindgen output. The
  homepage workflow now builds with `--debug-symbols false` and the root
  manifest defines the `wasm-release` / `server-release` profiles dx actually
  uses (`opt-level = "z"`, fat LTO, one codegen unit and `panic = "abort"` for
  the wasm; `strip = true` for the server). On its own this took the example
  site's wasm from 25.1 MB to 18.2 MB; with the grammar changes below the same
  site now ships 7.2 MB raw / 1.4 MB brotli.

### Added

- **Opt-in blog category pages** backed by published tags, with optional topic
  metadata, linked badges and navigation, URL pagination, canonical/social
  metadata, sitemap entries, and server-rendered 404 responses for invalid
  pages.
- **Brotli sidecars and a size gate in CI.** The homepage workflow writes a
  `.br` file next to every `.wasm`/`.js`/`.css` in the bundle (`dioxus-server`
  serves them with `content-encoding: br`, so the wasm costs 1.4 MB on the wire
  instead of 25 MB) and runs `scripts/wasm-size.sh` (`just size`), which reports
  raw and brotli sizes and fails when the raw wasm exceeds
  `WASM_SIZE_LIMIT_BYTES`.
- **Per-language syntax-highlighting features.** `highlight` on its own compiles
  only the Rust grammar (it ships with `dioxus-code`'s `runtime`); every other
  language is a `lang-*` feature on both `dioxus-mdx` and `dioxus-docs-kit`.
  Default: `lang-bash`, `lang-css`, `lang-dockerfile`, `lang-html`,
  `lang-javascript`, `lang-json`, `lang-markdown`, `lang-python`, `lang-toml`,
  `lang-typescript`, `lang-yaml`. Available but off by default:
  `lang-c-sharp`, `lang-cpp`, `lang-tsx` (plus `lang-rust`, a no-op alias).
- **`openapi` cargo feature** (on by default) on `dioxus-docs-kit` and
  `dioxus-mdx`. It gates spec *parsing* only: `parse_openapi`, `OpenApiError`,
  inline `<OpenAPI>…</OpenAPI>` blocks and `DocsConfig::with_openapi`. With it
  off, `openapiv3`, `serde_yaml` and `unsafe-libyaml` leave the dependency graph
  entirely; the `OpenApiSpec` types, `DocNode::OpenApi` and the viewer
  components stay, the API sidebar/search see an empty spec list, and an
  `<OpenAPI>` block renders as plain markdown.
- **`dioxus_mdx::parse_yaml_lite`** — a small dependency-free parser for the
  flat YAML subset frontmatter uses (plain and quoted scalars with `\"`/`\\`
  escapes, `true`/`false`, block and flow sequences, comments). The build script
  emits a `cargo:warning` when a docs page's frontmatter uses a shape it rejects
  (nested mapping, non-scalar sequence item, multi-line scalar), so a page that
  builds never silently loses its frontmatter at runtime.

### Changed

- Blog posts without a cover image emit `twitter:card` `summary` instead of
  `summary_large_image`.
- **BREAKING: the C#, C++ and TSX grammars are no longer compiled by default.**
  The C# and C++ tree-sitter tables were ~8 MB of the bundle; dropping them took
  the example site's release wasm from 19.0 MB to 9.9 MB (data section 15.2 MB →
  6.3 MB).
  TSX (1.5 MB, React-only) went next. Fences in those languages render as plain
  text unless `lang-c-sharp` / `lang-cpp` / `lang-tsx` is enabled.
- The example site enables only the grammars its own docs fence (bash, css,
  json, python, typescript) instead of the kit default, as a worked example of
  the trimming migration below. Its release wasm is 7.2 MB raw / 1.4 MB brotli,
  down from 25.1 MB uncompressed on the wire before this release.
- **BREAKING: the workspace `dioxus` dependency is `default-features = false`.**
  `dioxus-mdx` and `dioxus-docs-kit` request only `lib` (+ `router` on the kit),
  so they no longer force `launch`, `logger` or `devtools` onto consumers through
  cargo feature unification. (`dioxus-free-icons 0.10.0` still enables dioxus's
  defaults itself, so most builds see no difference until that is fixed
  upstream.)
- **BREAKING: `DocFrontmatter` and `BlogFrontmatter` no longer derive
  `serde::Deserialize`.** Frontmatter (docs and blog) is parsed by
  `parse_yaml_lite` instead of `serde_yaml`. Behaviour on unsupported input is
  unchanged: docs pages warn and fall back to an empty `DocFrontmatter` with the
  block still stripped; blog posts return the existing error.
- A code fence whose grammar is not compiled in renders as escaped plain text
  inside the same `<pre class="dxc">` markup. Previously any unresolved language
  (including a bare fence) was highlighted with the Markdown grammar.
- docs.rs metadata uses explicit feature lists instead of `all-features`.
- **BREAKING: every public enum is now `#[non_exhaustive]`** — `DocNode`,
  `CalloutType`, `ParamLocation`, `YamlValue`, `HttpMethod`, `ParameterLocation`,
  `SchemaType`, `OpenApiError` in `dioxus-mdx`; `CodeThemeConfig`, `DocsKitError`,
  `DocsVariant` in `dioxus-docs-kit`. Adding a variant (a new component, HTTP
  method, error kind, theme preset) is no longer a breaking change.
- `dioxus-mdx` uses `regex-lite` instead of `regex` when `highlight` is off.
  With highlighting on, `arborium-tree-sitter` links `regex` anyway, so the
  crate keeps sharing it; without it, `regex`, `regex-automata`, `regex-syntax`
  and `aho-corasick` (~440 KB of pre-wasm-opt code) leave the build.

### Migration

- **C# / C++ / TSX code blocks.** Add the features back, otherwise those blocks
  render as plain text:

  ```toml
  dioxus-docs-kit = { version = "0.7", features = ["lang-c-sharp", "lang-cpp", "lang-tsx"] }
  ```

- **Trimming further.** Ship only what you actually fence:

  ```toml
  dioxus-docs-kit = { version = "0.7", default-features = false, features = [
      "web", "mermaid", "highlight", "openapi", "lang-bash", "lang-json", "lang-toml",
  ] }
  ```

  `lang-rust` needs no grammar of its own; `highlight` alone highlights Rust.

- **Any other language** (Go, Zig, Kotlin, SQL, …). The kit deliberately carries
  features only for the languages above. For anything else add `dioxus-code` to
  your own manifest with the grammar you want; cargo unifies that feature into
  the copy the kit already uses, so `Language::from_slug` picks it up:

  ```toml
  dioxus-code = { version = "0.1", default-features = false, features = ["runtime", "lang-go"] }
  ```

  Fence it with `dioxus-code`'s canonical slug (` ```go `); the friendly aliases
  (` ```c++ `, ` ```yml `, ` ```sh `) only exist for the kit's own `lang-*`
  features.

- **dioxus features.** Only relevant if you set `default-features = false` on
  `dioxus` yourself: the kit no longer re-enables `launch`, `logger` and
  `devtools` for you, so request them on your own `dioxus` dependency. The stock
  `dioxus = { version = "0.7", features = ["router", "fullstack"] }` line keeps
  dioxus's defaults and needs no change:

  ```toml
  dioxus = { version = "0.7", features = ["lib", "router", "fullstack", "launch", "devtools", "logger"] }
  ```

- **`default-features = false` consumers** must add `"openapi"` to keep
  `DocsConfig::with_openapi` and `<OpenAPI>` blocks working.
- **Exhaustive `match`es** on any of the enums listed above need a `_ =>` arm
  now that they are `#[non_exhaustive]`.
- **Bundling your own site.** Build with
  `dx bundle --web --release --debug-symbols false`; without the flag `wasm-opt`
  aborts on DWARF and `dx` ships the unoptimized wasm. Copy `[profile.wasm-release]`
  from this repo's `Cargo.toml` and write `.br` sidecars into the bundle's
  `public/` directory — `dioxus-server` serves them with `content-encoding: br`
  automatically. See the kit README's "Production build" section.

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
