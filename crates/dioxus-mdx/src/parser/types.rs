//! Type definitions for parsed MDX documentation.

use super::openapi_types::OpenApiSpec;
use serde::{Deserialize, Serialize};

/// Parsed documentation page with frontmatter and content.
///
/// Produced at build time by `dioxus-docs-kit-build` and carried into the app
/// through the generated content bundle, so the browser only deserializes it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedDoc {
    /// Extracted frontmatter metadata.
    pub frontmatter: DocFrontmatter,
    /// Parsed content as a tree of doc nodes.
    pub content: Vec<DocNode>,
    /// Raw markdown content (after stripping imports and MDX components).
    ///
    /// Kept alongside the rendered nodes because `llms.txt`, the copy-page
    /// button and the table of contents all read the Markdown source.
    pub raw_markdown: String,
}

/// YAML frontmatter metadata from MDX files.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DocFrontmatter {
    /// Page title (used in H1 and browser tab).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub title: String,
    /// Short description (used in meta tags and previews).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Sidebar title (shorter than main title), from the `sidebarTitle` key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sidebar_title: Option<String>,
    /// Icon name (Lucide icon identifier).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

/// A node in the parsed documentation tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DocNode {
    /// Prose rendered to HTML at build time (with heading `id`s injected).
    ///
    /// Was `Markdown(String)` before 0.9.0, when the browser still ran
    /// markdown-rs on every page view.
    Html(String),
    /// Callout box (Tip, Note, Warning, Info).
    Callout(CalloutNode),
    /// Card with title, icon, optional link, and content.
    Card(CardNode),
    /// Group of cards in a grid layout.
    CardGroup(CardGroupNode),
    /// Tabbed content container.
    Tabs(TabsNode),
    /// Sequential steps guide.
    Steps(StepsNode),
    /// Collapsible accordion group.
    AccordionGroup(AccordionGroupNode),
    /// Code block with optional language.
    CodeBlock(CodeBlockNode),
    /// Code group with multiple language variants.
    CodeGroup(CodeGroupNode),
    /// API parameter field.
    ParamField(ParamFieldNode),
    /// API response field.
    ResponseField(ResponseFieldNode),
    /// Expandable section.
    Expandable(ExpandableNode),
    /// Request example container.
    RequestExample(RequestExampleNode),
    /// Response example container.
    ResponseExample(ResponseExampleNode),
    /// Changelog update entry.
    Update(UpdateNode),
    /// OpenAPI specification viewer.
    OpenApi(OpenApiNode),
}

/// Callout variant type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CalloutType {
    Tip,
    Note,
    Warning,
    Info,
}

impl CalloutType {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "tip" => Some(Self::Tip),
            "note" => Some(Self::Note),
            "warning" => Some(Self::Warning),
            "info" => Some(Self::Info),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tip => "Tip",
            Self::Note => "Note",
            Self::Warning => "Warning",
            Self::Info => "Info",
        }
    }

    /// DaisyUI alert class suffix.
    pub fn alert_class(&self) -> &'static str {
        match self {
            Self::Tip => "alert-success",
            Self::Note => "alert-info",
            Self::Warning => "alert-warning",
            Self::Info => "alert-info",
        }
    }

    /// Icon name for the callout type.
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Tip => "lightbulb",
            Self::Note => "info",
            Self::Warning => "alert-triangle",
            Self::Info => "info",
        }
    }
}

/// Callout box node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalloutNode {
    pub callout_type: CalloutType,
    /// Body rendered to HTML at build time.
    pub content_html: String,
}

/// Card node with optional link and icon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardNode {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    /// Body rendered to HTML at build time.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub content_html: String,
}

/// Grid group of cards.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardGroupNode {
    pub cols: u8,
    pub cards: Vec<CardNode>,
}

/// Tab in a tabbed interface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabNode {
    pub title: String,
    /// Content as parsed doc nodes (may contain nested components).
    pub content: Vec<DocNode>,
}

/// Tabbed content container.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabsNode {
    pub tabs: Vec<TabNode>,
}

/// Individual step in a steps guide.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepNode {
    pub title: String,
    /// Content as parsed doc nodes (may contain nested components).
    pub content: Vec<DocNode>,
}

/// Sequential steps container.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepsNode {
    pub steps: Vec<StepNode>,
}

/// Collapsible accordion item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccordionNode {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Content as parsed doc nodes (may contain nested components).
    pub content: Vec<DocNode>,
}

/// Accordion group container.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccordionGroupNode {
    pub items: Vec<AccordionNode>,
}

/// Fenced code block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeBlockNode {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
}

/// Code group with multiple language variants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeGroupNode {
    pub blocks: Vec<CodeBlockNode>,
}

/// Location of a parameter in an API request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ParamLocation {
    Header,
    Path,
    Query,
    Body,
}

impl ParamLocation {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "header" => Some(Self::Header),
            "path" => Some(Self::Path),
            "query" => Some(Self::Query),
            "body" => Some(Self::Body),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Header => "header",
            Self::Path => "path",
            Self::Query => "query",
            Self::Body => "body",
        }
    }

    /// Badge color class for the location.
    pub fn badge_class(&self) -> &'static str {
        match self {
            Self::Header => "badge-warning",
            Self::Path => "badge-primary",
            Self::Query => "badge-info",
            Self::Body => "badge-secondary",
        }
    }
}

/// API parameter documentation field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParamFieldNode {
    /// Parameter name (from header/path/query/body attribute).
    pub name: String,
    /// Where the parameter appears.
    pub location: ParamLocation,
    /// Data type (string, number, boolean, etc.).
    pub param_type: String,
    /// Whether the parameter is required.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub required: bool,
    /// Default value if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    /// Description content as parsed doc nodes (may contain nested components).
    pub content: Vec<DocNode>,
}

/// API response field documentation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseFieldNode {
    /// Field name in the response.
    pub name: String,
    /// Data type (string, array, object, etc.).
    pub field_type: String,
    /// Whether the field is always present.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub required: bool,
    /// Description, rendered to HTML at build time.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub content_html: String,
    /// Nested expandable sections (for object properties).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expandable: Option<ExpandableNode>,
}

/// Expandable section for nested content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpandableNode {
    /// Section title.
    pub title: String,
    /// Nested response fields.
    pub fields: Vec<ResponseFieldNode>,
}

/// Container for API request examples.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestExampleNode {
    /// Code blocks with different language examples.
    pub blocks: Vec<CodeBlockNode>,
}

/// Container for API response examples.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseExampleNode {
    /// Code blocks with different response scenarios.
    pub blocks: Vec<CodeBlockNode>,
}

/// Changelog version update entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateNode {
    /// Version label (e.g., "v0.9.0").
    pub label: String,
    /// Date description (e.g., "December 2025").
    pub description: String,
    /// Changelog content as parsed doc nodes.
    pub content: Vec<DocNode>,
}

/// OpenAPI specification viewer node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenApiNode {
    /// Parsed OpenAPI specification.
    pub spec: OpenApiSpec,
    /// Optional tag filter (only show endpoints with these tags).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Whether to show schema definitions section.
    pub show_schemas: bool,
}
