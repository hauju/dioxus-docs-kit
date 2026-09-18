# dioxus-docs-kit-build

Build-time content pipeline for [dioxus-docs-kit](https://github.com/hauju/dioxus-docs-kit). Reads your `_nav.json` / `_blog.json`, parses every MDX page and OpenAPI spec, renders the prose to HTML, precomputes the search index, and writes one JSON bundle into `OUT_DIR`.

Because all of that happens here, the wasm client links no MDX parser, no markdown-rs, no regex engine, no `openapiv3` and no `serde_yaml` — and never re-parses a page on navigation.

## Usage

Add it as a build dependency:

```toml
[build-dependencies]
dioxus-docs-kit-build = "0.9"
```

Create a `build.rs`:

```rust
fn main() {
    dioxus_docs_kit_build::DocsBuild::new("docs/_nav.json")
        // Optional: parse an OpenAPI spec into the same bundle
        .with_openapi("api-reference", "docs/api-reference/spec.yaml")
        .generate();

    // Optional: blog bundle from a `_blog.json` manifest
    dioxus_docs_kit_build::BlogBuild::new("blog/_blog.json").generate();
}
```

Then load the bundles with the `docs_bundle!()` / `blog_bundle!()` macros from `dioxus-docs-kit`:

```rust
static DOCS: LazyLock<DocsRegistry> = LazyLock::new(|| {
    DocsConfig::new(dioxus_docs_kit::docs_bundle!()).build()
});

static BLOG: LazyLock<BlogRegistry> = LazyLock::new(|| {
    BlogConfig::new(dioxus_docs_kit::blog_bundle!()).build()
});
```

Keep `dioxus-docs-kit` and `dioxus-docs-kit-build` on the same version: the
bundle carries a format version and the runtime refuses one it does not
understand.

## Validation

For release builds, opt into strict validation:

```rust
use dioxus_docs_kit_build::{DocsBuild, ValidationMode};

fn main() {
    DocsBuild::new("docs/_nav.json")
        .with_validation(ValidationMode::Strict)
        .generate();
}
```

Strict mode reports all detected problems, then fails on missing files,
duplicate navigation entries, unsupported frontmatter, and detected broken
internal links or heading anchors. The default (`ValidationMode::Warn`) keeps
reporting these as warnings. External links, application routes, and inferred
dynamic OpenAPI routes are skipped by the link validator; strict mode does
not validate runtime default paths or OpenAPI operation collisions.

`BlogBuild::with_validation(ValidationMode::Strict)` also rejects missing or
duplicate blog posts and `categories` keys that match no post tag. Invalid blog
frontmatter fails in either mode; blog links are not validated.

## What it does

1. Reads `_nav.json` to discover all doc pages
2. Emits `cargo:rerun-if-changed` for `_nav.json`, every `.mdx` file and every OpenAPI spec
3. Parses each page into the `dioxus-mdx` node tree, rendering its Markdown to HTML
4. Parses each OpenAPI spec into the kit's own `OpenApiSpec` types
5. Builds the section-level search index, lowercase match fields included
6. Writes `docs_bundle.json` (and `blog_bundle.json`) to `OUT_DIR`

The docs directory is inferred from the parent of the nav path (e.g. `"docs/_nav.json"` uses `"docs/"`).

If your content does not come from the filesystem, call
[`docs_bundle_json`](https://docs.rs/dioxus-docs-kit-build/latest/dioxus_docs_kit_build/fn.docs_bundle_json.html)
or `blog_bundle_json` with the sources you have and write the result yourself.

## License

MIT
