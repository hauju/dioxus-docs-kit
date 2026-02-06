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

pub mod components;
pub mod parser;

// Re-export parser types and functions
pub use parser::{
    extract_frontmatter, get_raw_markdown, highlight_code, parse_document, parse_mdx,
    parse_openapi, AccordionGroupNode, AccordionNode, ApiInfo, ApiOperation, ApiParameter,
    ApiRequestBody, ApiResponse, ApiServer, ApiTag, CalloutNode, CalloutType, CardGroupNode,
    CardNode, CodeBlockNode, CodeGroupNode, DocFrontmatter, DocNode, ExpandableNode, HttpMethod,
    MediaTypeContent, OpenApiError, OpenApiNode, OpenApiSpec, ParamFieldNode, ParamLocation,
    ParameterLocation, ParsedDoc, RequestExampleNode, ResponseExampleNode, ResponseFieldNode,
    SchemaDefinition, SchemaType, StepNode, StepsNode, TabNode, TabsNode, UpdateNode,
};

// Re-export components
pub use components::{
    extract_headers, slugify, ApiInfoHeader, DocAccordionGroup, DocAccordionItem, DocCallout,
    DocCard, DocCardGroup, DocCodeBlock, DocCodeGroup, DocContent, DocExpandable,
    DocNodeRenderer, DocParamField, DocRequestExample, DocResponseExample, DocResponseField,
    DocSteps, DocTableOfContents, DocTabs, DocUpdate, EndpointCard, EndpointPage, MdxContent,
    MdxIcon, MdxRenderer, MethodBadge, OpenApiViewer, ParameterItem, ParametersList,
    RequestBodySection, ResponseItem, ResponsesList, SchemaDefinitions, SchemaTypeLabel,
    SchemaViewer, TagGroup, UngroupedEndpoints,
};
