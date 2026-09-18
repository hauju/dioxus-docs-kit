//! # dioxus-mdx
//!
//! MDX parsing and rendering components for Dioxus applications.
//!
//! This crate provides a complete solution for rendering Mintlify-style MDX documentation
//! in Dioxus applications, including:
//!
//! - **Parser**: Extracts frontmatter, code blocks, and custom components from MDX
//! - **Components**: Pre-built Dioxus components for callouts, cards, tabs, steps, etc.
//! - **Syntax Highlighting**: Code blocks with language-aware highlighting
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use dioxus::prelude::*;
//! use dioxus_mdx::{parse_document, MdxContent};
//!
//! #[component]
//! fn DocsPage(content: String) -> Element {
//!     rsx! {
//!         MdxContent { content }
//!     }
//! }
//! ```
//!
//! ## Parsing Only
//!
//! If you want to parse MDX without using the components:
//!
//! ```rust
//! use dioxus_mdx::{parse_document, parse_mdx, DocNode};
//!
//! let mdx_content = r#"---
//! title: Getting Started
//! ---
//!
//! <Tip>This is a helpful tip!</Tip>
//!
//! ## Introduction
//!
//! Welcome to the documentation.
//! "#;
//!
//! // Parse with frontmatter
//! let doc = parse_document(mdx_content);
//! assert_eq!(doc.frontmatter.title, "Getting Started");
//!
//! // Parse content only
//! let nodes = parse_mdx("## Hello\n\n<Note>A note</Note>");
//! ```
//!
//! ## Supported Components
//!
//! - **Callouts**: `<Tip>`, `<Note>`, `<Warning>`, `<Info>`
//! - **Cards**: `<Card>`, `<CardGroup>`
//! - **Tabs**: `<Tabs>`, `<Tab>`
//! - **Steps**: `<Steps>`, `<Step>`
//! - **Accordion**: `<AccordionGroup>`, `<Accordion>`
//! - **Code**: `<CodeGroup>`, fenced code blocks with syntax highlighting
//! - **API Docs**: `<ParamField>`, `<ResponseField>`, `<Expandable>`
//! - **Examples**: `<RequestExample>`, `<ResponseExample>`
//! - **Changelog**: `<Update>`
//!
//! ## Styling
//!
//! Components use Tailwind CSS with DaisyUI classes. Ensure your project has
//! Tailwind and DaisyUI configured. The components use:
//!
//! - Base/neutral classes: `bg-base-200`, `text-base-content`, etc.
//! - Color classes: `text-primary`, `bg-success/10`, etc.
//! - Typography: `prose`, `prose-sm`
//!
//! ## Features
//!
//! - `web` (default): Enables web-specific features like clipboard copy
//! - `mermaid` (default): Renders ` ```mermaid ` fences as diagrams
//!
//! Syntax highlighting is always on and comes from `hl-lite`, which the
//! `components` feature pulls in: no grammars to select, no C to compile, and
//! token colors are CSS (`.hl-*` classes), not Rust. A fence whose language
//! `hl_lite::Lang::from_slug` does not know renders as plain text.
//! - `openapi` (default): Parses OpenAPI specs — `parse_openapi` and inline
//!   `<OpenAPI>…</OpenAPI>` blocks. Turning it off drops `openapiv3` and
//!   `serde_yaml` from the build; the `OpenApiSpec` types and the viewer
//!   components stay, and an `<OpenAPI>` block renders as plain markdown.
//!
//! ## Custom Link Handling
//!
//! For internal navigation, components accept an `on_link` callback:
//!
//! ```rust,ignore
//! use dioxus::prelude::*;
//! use dioxus_mdx::DocCardGroup;
//!
//! #[component]
//! fn DocsPage(group: CardGroupNode) -> Element {
//!     let nav = use_navigator();
//!
//!     rsx! {
//!         DocCardGroup {
//!             group,
//!             on_link: move |href: String| nav.push(&href),
//!         }
//!     }
//! }
//! ```

#[cfg(feature = "components")]
pub mod components;
#[cfg(feature = "components")]
pub mod lucide;
pub mod parser;
#[cfg(feature = "parse")]
mod re;
mod text;

// Re-export parser types and functions
pub use parser::{
    AccordionGroupNode, AccordionNode, ApiInfo, ApiOperation, ApiParameter, ApiRequestBody,
    ApiResponse, ApiServer, ApiTag, CalloutNode, CalloutType, CardGroupNode, CardNode,
    CodeBlockNode, CodeGroupNode, DocFrontmatter, DocNode, ExpandableNode, HttpMethod,
    MediaTypeContent, OpenApiNode, OpenApiSpec, ParamFieldNode, ParamLocation, ParameterLocation,
    ParsedDoc, RequestExampleNode, ResponseExampleNode, ResponseFieldNode, SchemaDefinition,
    SchemaType, StepNode, StepsNode, TabNode, TabsNode, UpdateNode, YamlLiteError, YamlMap,
    YamlValue, parse_yaml_lite,
};

// Heading helpers are free of both dioxus and regex, so they are available in
// every configuration (the renderer, the TOC and the build-time bundle
// generator all have to agree on anchor ids).
pub use text::{extract_headers, parse_atx_heading, slugify, strip_markdown_links};

// Parsing lives behind the `parse` feature (default). A docs-kit app turns it
// off: its content is parsed into a bundle at build time.
#[cfg(feature = "parse")]
pub use parser::{
    extract_frontmatter, parse_body, parse_document, parse_mdx, strip_leading_h1, to_html,
    to_html_with_heading_ids,
};

// Spec parsing lives behind the `openapi-parse` feature (default); the
// `OpenApiSpec` types and the viewer components are always available.
#[cfg(feature = "openapi-parse")]
pub use parser::{OpenApiError, parse_openapi};

// The highlighter itself, so consumers can lex a snippet (or map a fence slug
// to a language) without adding their own dependency on it.
#[cfg(feature = "components")]
pub use hl_lite as hl;

// Re-export components
#[cfg(feature = "components")]
pub use components::{
    ApiInfoHeader, DocAccordionGroup, DocAccordionItem, DocCallout, DocCard, DocCardGroup,
    DocCodeBlock, DocCodeGroup, DocContent, DocExpandable, DocNodeRenderer, DocParamField,
    DocRequestExample, DocResponseExample, DocResponseField, DocSteps, DocTableOfContents, DocTabs,
    DocUpdate, EndpointCard, EndpointPage, MdxIcon, MethodBadge, OpenApiViewer, ParameterItem,
    ParametersList, RequestBodySection, ResponseItem, ResponsesList, SchemaDefinitions,
    SchemaTypeLabel, SchemaViewer, TagGroup, UngroupedEndpoints,
};

// Runtime MDX parsing components need both features.
#[cfg(all(feature = "components", feature = "parse"))]
pub use components::{MdxContent, MdxRenderer};

#[cfg(feature = "mermaid")]
pub use components::MermaidDiagram;
