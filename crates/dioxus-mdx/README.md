# dioxus-mdx

MDX parsing and rendering components for [Dioxus](https://dioxuslabs.com/) applications.

Parse Mintlify-style MDX into an AST and render it with pre-built Dioxus components — callouts, cards, tabs, code groups, accordions, steps, and more. Includes syntax highlighting, frontmatter extraction, and OpenAPI spec parsing.

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
dioxus-mdx = "0.7"
```

Render MDX content in a component:

```rust
use dioxus::prelude::*;
use dioxus_mdx::MdxContent;

#[component]
fn BlogPost(content: String) -> Element {
    rsx! {
        article { class: "prose",
            MdxContent { content }
        }
    }
}
```

## Parsing Only

Use the parser directly without rendering — and without pulling in Dioxus at
all, by depending on the crate with `default-features = false, features = ["parse"]`.
That is how `dioxus-docs-kit-build` parses a whole documentation site inside a
build script.

The parsed tree is `serde`-serializable, so you can parse at build time and
deserialize the result in the browser instead of shipping the parser.

```rust
use dioxus_mdx::{parse_document, parse_mdx};

let doc = parse_document(r#"---
title: Getting Started
---

<Tip>This is a helpful tip!</Tip>

## Introduction

Welcome to the documentation.
"#);

assert_eq!(doc.frontmatter.title, "Getting Started");

// Parse content without frontmatter
let nodes = parse_mdx("## Hello\n\n<Note>A note</Note>");
```

## Supported Components

| Component | MDX Syntax |
|-----------|-----------|
| Callouts | `<Tip>`, `<Note>`, `<Warning>`, `<Info>` |
| Cards | `<Card>`, `<CardGroup>` |
| Tabs | `<Tabs>`, `<Tab>` |
| Steps | `<Steps>`, `<Step>` |
| Accordion | `<AccordionGroup>`, `<Accordion>` |
| Code | `<CodeGroup>`, fenced code blocks with syntax highlighting |
| API Docs | `<ParamField>`, `<ResponseField>`, `<Expandable>` |
| Examples | `<RequestExample>`, `<ResponseExample>` |
| Changelog | `<Update>` |

## OpenAPI Support

Parse OpenAPI/Swagger specs and render interactive API reference pages:

```rust
use dioxus_mdx::parse_openapi;

let spec = parse_openapi(include_str!("api.yaml")).unwrap();
```

The `EndpointPage` component renders a two-column Mintlify-style API reference for each operation.

## Syntax Highlighting

Code blocks get automatic syntax highlighting via [dioxus-code](https://crates.io/crates/dioxus-code). `DocCodeBlock` and `DocCodeGroup` render through it on both server and wasm targets — no extra wiring required.

The crate ships a common set of languages baked in (bash, css, dockerfile, html, javascript, json, markdown, python, rust, toml, typescript, yaml; `lang-c-sharp`, `lang-cpp` and `lang-tsx` are available but off by default). To support more, add `dioxus-code` directly to your `Cargo.toml` with the desired `lang-*` features — Cargo unifies them into the transitive dep, so no fork or wrapper feature is needed.

## Styling Setup

Components use **Tailwind CSS 4** with **DaisyUI 5** and **@tailwindcss/typography**.

```sh
bun add tailwindcss @tailwindcss/typography daisyui
```

When using as a **crates.io dependency**, Tailwind can't scan the crate source.
Copy `safelist.html` from the crate into your project root and add it as a source:

```css
@source "./safelist.html";
```

When using as a **workspace path dependency**, point directly at the source:

```css
@source "./crates/dioxus-mdx/src/**/*.rs";
```

Components use semantic DaisyUI classes (`bg-base-200`, `text-base-content`, `text-primary`, etc.)
and adapt to any DaisyUI theme.

## Features

- `components` (default) — the Dioxus renderer components. This is the only thing that pulls `dioxus` in; without it the crate is a plain Rust library of the document types plus (with `parse`) the parser.
- `parse` (default) — the MDX/Markdown parser, including Markdown → HTML rendering. Pulls in `markdown` and a regex engine. A `dioxus-docs-kit` app leaves this off on the client: its pages are parsed into a bundle at build time.
- `web` (default) — enables web-specific features like clipboard copy buttons on code blocks
- `highlight` (default) — syntax-highlights code blocks via [`dioxus-code`](https://crates.io/crates/dioxus-code). Disable (`default-features = false`) to drop the dependency and its C-compiling tree-sitter grammars: no C toolchain is needed for wasm and the binary is smaller, but code blocks render as plain (uncolored) text. Turning it off also removes the `CodeTheme`, `Theme`, and `CodeThemeOverride` re-exports.
- `lang-*` — one tree-sitter grammar each, on top of `highlight`. `highlight` on its own highlights Rust only (`dioxus-code`'s `runtime` always compiles it, so `lang-rust` needs nothing extra). The default set adds `lang-bash`, `lang-css`, `lang-dockerfile`, `lang-html`, `lang-javascript`, `lang-json`, `lang-markdown`, `lang-python`, `lang-toml`, `lang-typescript` and `lang-yaml`; `lang-c-sharp`, `lang-cpp` and `lang-tsx` are available but off by default (their grammars are the largest). For any other language, depend on `dioxus-code` directly — `dioxus-code = { version = "0.1", default-features = false, features = ["runtime", "lang-go"] }` — and cargo unifies the grammar into the copy this crate uses. A fence whose grammar is not compiled in renders as plain text in the same markup.
- `openapi` — the `OpenApiSpec` types and the viewer components (`OpenApiViewer`, `EndpointPage`, …). Costs no dependencies; always compiled.
- `openapi-parse` (default) — the spec *parser*: `parse_openapi()` and inline `<OpenAPI>…</OpenAPI>` blocks, plus `openapiv3` and `serde_yaml` (and its `unsafe-libyaml`). Disable to drop them; an `<OpenAPI>` block then falls through to the markdown branch and its body renders as text. Frontmatter parsing is unaffected: it uses the built-in `parse_yaml_lite` subset parser, not `serde_yaml`.

## License

MIT
