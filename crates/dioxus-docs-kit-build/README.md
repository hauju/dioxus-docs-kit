# dioxus-docs-kit-build

Build-time helper for [dioxus-docs-kit](https://github.com/hauju/dioxus-docs-kit). Reads your `_nav.json` navigation file and generates a content map that embeds all MDX files via `include_str!()` at compile time.

## Usage

Add it as a build dependency:

```toml
[build-dependencies]
dioxus-docs-kit-build = "0.2"
```

Create a `build.rs`:

```rust
fn main() {
    dioxus_docs_kit_build::generate_content_map("docs/_nav.json");
}
```

Then use the `doc_content_map!()` macro from `dioxus-docs-kit` to consume the generated file:

```rust
dioxus_docs_kit::doc_content_map!();

// Now `doc_content_map()` returns HashMap<&'static str, &'static str>
```

## What it does

1. Reads the `_nav.json` file to discover all doc pages
2. Emits `cargo:rerun-if-changed` for `_nav.json` and every `.mdx` file
3. Writes `doc_content_generated.rs` to `OUT_DIR` containing `include_str!()` calls for each page

The docs directory is inferred from the parent of the nav path (e.g. `"docs/_nav.json"` uses `"docs/"`).

## License

MIT
