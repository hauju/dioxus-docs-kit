# dioxus-mdx

MDX parsing and rendering components for [Dioxus](https://dioxuslabs.com/) applications.

Parse Mintlify-style MDX into an AST and render it with pre-built Dioxus components — callouts, cards, tabs, code groups, accordions, steps, and more. Includes syntax highlighting, frontmatter extraction, and OpenAPI spec parsing.

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
dioxus-mdx = "0.1"
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

Use the parser directly without rendering:

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

Code blocks get automatic syntax highlighting via [syntect](https://crates.io/crates/syntect):

```rust
use dioxus_mdx::highlight_code;

let html = highlight_code("let x = 42;", Some("rust"));
```

## Styling

Components use [Tailwind CSS](https://tailwindcss.com/) with [DaisyUI](https://daisyui.com/) classes. Ensure your project includes both. The components use semantic DaisyUI classes (`bg-base-200`, `text-base-content`, `text-primary`, etc.) so they adapt to any DaisyUI theme.

## Features

- `web` (default) — enables web-specific features like clipboard copy buttons on code blocks

## License

MIT
