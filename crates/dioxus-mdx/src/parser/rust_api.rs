//! Distilled Rust API model — the deserialize side of the JSON contract
//! produced by `dioxus-docs-kit-build`'s `distill-rustdoc`.
//!
//! The model is a small, committed JSON file distilled from nightly rustdoc
//! JSON at development time (the raw rustdoc output is megabytes and
//! format-version-locked, so it never ships). See the build crate's `rustdoc`
//! module for the serialize side; the JSON is the contract between the two.

use serde::Deserialize;
use std::fmt;

/// Model versions this renderer understands (see `MODEL_VERSION` in the build
/// crate's serialize side).
const SUPPORTED_MODEL_VERSION: u32 = 1;

/// Error parsing a Rust API model file.
#[derive(Debug)]
pub struct RustApiError(pub String);

impl fmt::Display for RustApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for RustApiError {}

/// The public API of one crate, distilled for rendering.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RustApiModel {
    pub model_version: u32,
    /// Crate name as it appears in code (`dioxus_docs_kit`).
    pub crate_name: String,
    #[serde(default)]
    pub crate_version: Option<String>,
    /// Root module documentation (markdown).
    #[serde(default)]
    pub crate_docs: Option<String>,
    pub items: Vec<RustApiItem>,
}

/// One public item (struct, enum, trait, function, …).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RustApiItem {
    /// URL slug in rustdoc style (`struct.DocsConfig`).
    pub slug: String,
    pub name: String,
    pub kind: RustItemKind,
    /// Module path segments below the crate root (empty for root re-exports).
    #[serde(default)]
    pub path: Vec<String>,
    /// Rendered declaration (`pub fn use_docs_providers(...) -> DocsProviders`).
    pub signature: String,
    /// Item documentation (markdown).
    #[serde(default)]
    pub docs: Option<String>,
    /// Fields, variants, methods, and associated items.
    #[serde(default)]
    pub members: Vec<RustApiMember>,
    /// Names of implemented (non-auto, non-blanket) traits.
    #[serde(default)]
    pub trait_impls: Vec<String>,
}

impl RustApiItem {
    /// First sentence of the docs, for search results and meta descriptions.
    pub fn summary(&self) -> Option<String> {
        let docs = self.docs.as_ref()?;
        let first_line = docs.lines().find(|l| !l.trim().is_empty())?;
        Some(first_line.trim().to_string())
    }

    /// Members of one kind, in declaration order.
    pub fn members_of(&self, kind: RustMemberKind) -> Vec<&RustApiMember> {
        self.members.iter().filter(|m| m.kind == kind).collect()
    }
}

/// Item kinds surfaced in the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustItemKind {
    Struct,
    Enum,
    Trait,
    Function,
    TypeAlias,
    Constant,
    Macro,
}

impl RustItemKind {
    /// Display order for sidebar groups and item pages.
    pub const ALL: [RustItemKind; 7] = [
        RustItemKind::Struct,
        RustItemKind::Enum,
        RustItemKind::Trait,
        RustItemKind::Function,
        RustItemKind::TypeAlias,
        RustItemKind::Constant,
        RustItemKind::Macro,
    ];

    /// Singular label ("Struct").
    pub fn label(&self) -> &'static str {
        match self {
            RustItemKind::Struct => "Struct",
            RustItemKind::Enum => "Enum",
            RustItemKind::Trait => "Trait",
            RustItemKind::Function => "Function",
            RustItemKind::TypeAlias => "Type Alias",
            RustItemKind::Constant => "Constant",
            RustItemKind::Macro => "Macro",
        }
    }

    /// Plural group label ("Structs").
    pub fn group_label(&self) -> &'static str {
        match self {
            RustItemKind::Struct => "Structs",
            RustItemKind::Enum => "Enums",
            RustItemKind::Trait => "Traits",
            RustItemKind::Function => "Functions",
            RustItemKind::TypeAlias => "Type Aliases",
            RustItemKind::Constant => "Constants",
            RustItemKind::Macro => "Macros",
        }
    }

    /// One-letter abbreviation for compact sidebar chips.
    pub fn abbrev(&self) -> &'static str {
        match self {
            RustItemKind::Struct => "S",
            RustItemKind::Enum => "E",
            RustItemKind::Trait => "T",
            RustItemKind::Function => "F",
            RustItemKind::TypeAlias => "A",
            RustItemKind::Constant => "C",
            RustItemKind::Macro => "M",
        }
    }

    /// Badge color classes, mirroring rust-analyzer's item colors.
    ///
    /// Dynamic classes: keep in sync with `safelist.html`.
    pub fn badge_class(&self) -> &'static str {
        match self {
            RustItemKind::Struct => "badge-info",
            RustItemKind::Enum => "badge-secondary",
            RustItemKind::Trait => "badge-warning",
            RustItemKind::Function => "badge-success",
            RustItemKind::TypeAlias => "badge-accent",
            RustItemKind::Constant => "badge-primary",
            RustItemKind::Macro => "badge-error",
        }
    }
}

/// Member kinds within an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustMemberKind {
    Field,
    Variant,
    Method,
    AssocType,
    AssocConst,
}

impl RustMemberKind {
    /// Section heading on the item page ("Methods").
    pub fn section_label(&self) -> &'static str {
        match self {
            RustMemberKind::Field => "Fields",
            RustMemberKind::Variant => "Variants",
            RustMemberKind::Method => "Methods",
            RustMemberKind::AssocType => "Associated Types",
            RustMemberKind::AssocConst => "Associated Constants",
        }
    }

    /// All member kinds in page display order.
    pub const ALL: [RustMemberKind; 5] = [
        RustMemberKind::Variant,
        RustMemberKind::Field,
        RustMemberKind::Method,
        RustMemberKind::AssocType,
        RustMemberKind::AssocConst,
    ];
}

/// A member of an item: a field, variant, method, or associated item.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RustApiMember {
    pub kind: RustMemberKind,
    pub name: String,
    pub signature: String,
    #[serde(default)]
    pub docs: Option<String>,
}

/// Parse a distilled Rust API model from its JSON.
pub fn parse_rust_api(json: &str) -> Result<RustApiModel, RustApiError> {
    let model: RustApiModel = serde_json::from_str(json)
        .map_err(|e| RustApiError(format!("failed to parse Rust API model JSON: {e}")))?;
    if model.model_version != SUPPORTED_MODEL_VERSION {
        return Err(RustApiError(format!(
            "Rust API model has model_version {}, but this renderer supports \
             {SUPPORTED_MODEL_VERSION}. Regenerate the model with a matching \
             dioxus-docs-kit-build (distill-rustdoc).",
            model.model_version
        )));
    }
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODEL: &str = r#"{
        "model_version": 1,
        "crate_name": "demo",
        "crate_version": "1.0.0",
        "items": [
            {
                "slug": "struct.Config",
                "name": "Config",
                "kind": "struct",
                "signature": "pub struct Config",
                "docs": "Builder for things.\n\nMore detail.",
                "members": [
                    {"kind": "method", "name": "new", "signature": "pub fn new() -> Self"},
                    {"kind": "field", "name": "path", "signature": "pub path: String"}
                ],
                "trait_impls": ["Clone"]
            }
        ]
    }"#;

    #[test]
    fn parses_model_and_reads_summary() {
        let model = parse_rust_api(MODEL).unwrap();
        assert_eq!(model.crate_name, "demo");
        let item = &model.items[0];
        assert_eq!(item.kind, RustItemKind::Struct);
        assert_eq!(item.summary().as_deref(), Some("Builder for things."));
        assert_eq!(item.members_of(RustMemberKind::Method).len(), 1);
        assert_eq!(item.members_of(RustMemberKind::Field).len(), 1);
    }

    #[test]
    fn rejects_unknown_model_version() {
        let err =
            parse_rust_api(r#"{"model_version": 99, "crate_name": "x", "items": []}"#).unwrap_err();
        assert!(err.to_string().contains("model_version 99"));
    }
}
