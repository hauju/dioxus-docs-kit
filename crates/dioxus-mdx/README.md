# dioxus-mdx

MDX parsing and rendering components for [Dioxus](https://dioxuslabs.com/) applications.

Parse Mintlify-style MDX into an AST and render it with pre-built Dioxus components — callouts, cards, tabs, code groups, accordions, steps, and more. Includes dependency-free syntax highlighting, frontmatter extraction, and OpenAPI spec parsing.

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

Code blocks are highlighted by [hl-lite](https://crates.io/crates/hl-lite),
which `components` pulls in. `DocCodeBlock` and `DocCodeGroup` render through
it on both server and wasm targets — no extra wiring, no grammar features, no C
toolchain, and about 48 KB of wasm for every language at once.

Supported: `bash`, `css`, `dockerfile`, `html`, `javascript`, `json`,
`markdown`, `python`, `rust`, `toml`, `typescript`, `yaml`, plus the usual
fence aliases (`rs`, `sh`, `zsh`, `console`, `yml`, `jsx`, `tsx`, `jsonc`,
`md`, `htm`, …). A fence whose language is unknown renders as plain text in the
same markup; a fence with no language falls back to the block's filename. The
lexer is re-exported as `dioxus_mdx::hl` if you want to call it yourself.

### Token colors are CSS

A block renders as `<pre class="dk-code"><code>` with one
`<span class="hl-{kind}">` per classified token — unclassified text has no
span. There are thirteen kinds, one per `hl_lite::Kind`:

`hl-keyword`, `hl-string`, `hl-comment`, `hl-number`, `hl-type`, `hl-function`,
`hl-attribute`, `hl-property`, `hl-tag`, `hl-operator`, `hl-punctuation`,
`hl-constant`, `hl-variable`.

This crate ships no colors for them. `dioxus-docs-kit` wires them to its
`--dk-hl-*` tokens; standalone, paste this (GitHub Light / Tokyo Night, flipped
by the page's `color-scheme`):

```css
.dk-code .hl-keyword     { color: light-dark(#cf222e, #bb9af7); }
.dk-code .hl-string      { color: light-dark(#0a3069, #9ece6a); }
.dk-code .hl-comment     { color: light-dark(#6e7781, #565f89); font-style: italic; }
.dk-code .hl-number      { color: light-dark(#0550ae, #ff9e64); }
.dk-code .hl-type        { color: light-dark(#953800, #2ac3de); }
.dk-code .hl-function    { color: light-dark(#8250df, #7aa2f7); }
.dk-code .hl-attribute   { color: light-dark(#0550ae, #e0af68); }
.dk-code .hl-property    { color: light-dark(#953800, #7dcfff); }
.dk-code .hl-tag         { color: light-dark(#116329, #f7768e); }
.dk-code .hl-operator    { color: light-dark(#cf222e, #89ddff); }
.dk-code .hl-punctuation { color: light-dark(#57606a, #9aa5ce); }
.dk-code .hl-constant    { color: light-dark(#0550ae, #ff9e64); }
.dk-code .hl-variable    { color: light-dark(#953800, #c0caf5); }
```

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
- `openapi` — the `OpenApiSpec` types and the viewer components (`OpenApiViewer`, `EndpointPage`, …). Costs no dependencies; always compiled.
- `openapi-parse` (default) — the spec *parser*: `parse_openapi()` and inline `<OpenAPI>…</OpenAPI>` blocks, plus `openapiv3` and `serde_yaml` (and its `unsafe-libyaml`). Disable to drop them; an `<OpenAPI>` block then falls through to the markdown branch and its body renders as text. Frontmatter parsing is unaffected: it uses the built-in `parse_yaml_lite` subset parser, not `serde_yaml`.

## License

MIT
