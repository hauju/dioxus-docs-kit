# dioxus-docs-kit-example

Example documentation site built with [dioxus-docs-kit](crates/dioxus-docs-kit/) and [dioxus-mdx](crates/dioxus-mdx/).

Demonstrates a full-featured docs site with MDX content, sidebar navigation, full-text search, OpenAPI API reference pages, and theme switching — all in a Dioxus 0.7 fullstack app.

## Project Structure

```
├── src/main.rs                      # App routes, navbar, docs layout glue
├── build.rs                         # Generates content map from docs/_nav.json
├── docs/                            # MDX documentation content
│   ├── _nav.json                    # Navigation config
│   └── api-reference/petstore.yaml  # OpenAPI spec
├── crates/
│   ├── dioxus-mdx/                  # MDX parser + renderer (standalone crate)
│   └── dioxus-docs-kit/             # Docs site shell (layout, sidebar, search)
```

## Getting Started

Install the Dioxus CLI:

```sh
curl -sSL http://dioxus.dev/install.sh | sh
```

Run the dev server:

```sh
dx serve
```

Build for production:

```sh
dx build --release
```

## Crates

### [dioxus-mdx](crates/dioxus-mdx/)

Standalone MDX parser and renderer. Use this crate directly for blogs, content pages, or any project that needs to render MDX without the full docs site shell.

### [dioxus-docs-kit](crates/dioxus-docs-kit/)

Complete documentation site shell with sidebar, search, page navigation, OpenAPI support, and mobile responsiveness. Depends on `dioxus-mdx` for content rendering.

## License

MIT
