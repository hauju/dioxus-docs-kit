# dioxus-docs-kit-build

Build-time helper for [dioxus-docs-kit](https://github.com/hauju/dioxus-docs-kit). Reads your `_nav.json` navigation file and generates a content map that embeds all MDX files via `include_str!()` at compile time.

## Usage

Add it as a build dependency:

```toml
[build-dependencies]
dioxus-docs-kit-build = "0.7"
```

Create a `build.rs`:

```rust
fn main() {
    dioxus_docs_kit_build::generate_content_map("docs/_nav.json");

    // Optional: blog content map from a `_blog.json` manifest
    dioxus_docs_kit_build::generate_blog_content_map("blog/_blog.json");
}
```

Then use the `doc_content_map!()` / `blog_content_map!()` macros from `dioxus-docs-kit` to consume the generated files:

```rust
dioxus_docs_kit::doc_content_map!();
dioxus_docs_kit::blog_content_map!();

// Now `doc_content_map()` / `blog_content_map()` return
// HashMap<&'static str, &'static str>
```

## What it does

For release builds, opt into strict validation:

```rust
use dioxus_docs_kit_build::{generate_content_map_with_validation, ValidationMode};

fn main() {
    generate_content_map_with_validation("docs/_nav.json", ValidationMode::Strict);
}
```

Strict mode reports all detected problems, then fails on missing files,
duplicate navigation entries, unsupported frontmatter, and detected broken
internal links or heading anchors. The original `generate_content_map` keeps
reporting these as warnings. External links, application routes, and inferred
dynamic OpenAPI routes are skipped by the link validator; strict mode does
not validate runtime default paths or OpenAPI operation collisions.

`generate_blog_content_map_with_validation(path, ValidationMode::Strict)`
also rejects missing or duplicate blog posts and `categories` keys that match
no post tag. Invalid blog frontmatter fails
in either mode; blog links are not validated.

1. Reads the `_nav.json` file to discover all doc pages
2. Emits `cargo:rerun-if-changed` for `_nav.json` and every `.mdx` file
3. Writes `doc_content_generated.rs` to `OUT_DIR` containing `include_str!()` calls for each page

The docs directory is inferred from the parent of the nav path (e.g. `"docs/_nav.json"` uses `"docs/"`).

## License

MIT
