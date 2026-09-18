---
title: Getting started
description: Install the kit and render your first page.
---

# Getting started

Ship a documentation site from **Markdown** and *Rust* in about ten minutes.
The shell is a normal Dioxus component, so it drops into an app you already
have — see [the architecture notes](/docs/concepts/architecture) for how the
pieces fit.

## Install

1. Add the crate: `cargo add dioxus-docs-kit`
2. Add the build helper to `[build-dependencies]`
3. Create `docs/_nav.json`

> **Note**
> The build script calls `include_str!()` for every page listed in
> `_nav.json`, so a path that does not exist fails the build.

### Wire up the content map

```rust
let docs = DocsConfig::new(NAV_JSON, doc_content_map())
    .with_openapi("/api", OPENAPI_YAML)
    .build();
```

Nothing inside that fence is highlighted as Rust by this lexer — the renderer
splits fences out first.

## What you get

- A sidebar grouped by `tab`
- Full-text search with a `⌘K` modal
- `llms.txt` and `llms-full.txt`, generated from the same registry
- Dark and light themes

***

![The docs shell](/assets/hero.png)

| Feature | Status |
| ------- | ------ |
| MDX     | stable |
| OpenAPI | stable |
| Search  | stable |

See the [changelog](https://github.com/hauju/dioxus-docs-kit/blob/main/CHANGELOG.md)
for what moved in `0.8.0`, and `docs/_nav.json` for the tab layout.

~~~bash
cargo add dioxus-docs-kit
~~~
