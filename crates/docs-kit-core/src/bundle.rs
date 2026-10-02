//! The build-time content bundle, as read by the runtime.
//!
//! `dioxus-docs-kit-build` writes one JSON document per registry into `OUT_DIR`;
//! the app embeds it with [`docs_bundle!`](crate::docs_bundle) /
//! [`blog_bundle!`](crate::blog_bundle) and the registry deserializes it the
//! first time it is touched. Nothing here parses MDX, YAML or Markdown.

use crate::blog::types::{Author, BlogCategoryMetadata, BlogPost, BlogSearchEntry};
use crate::error::DocsKitError;
use crate::registry::{NavConfig, SearchEntry};
#[cfg(feature = "openapi")]
use dioxus_mdx::OpenApiSpec;
use dioxus_mdx::ParsedDoc;
use serde::Deserialize;
use std::collections::HashMap;

/// Bundle layout this library understands. Must match
/// `dioxus_docs_kit_build::bundle::BUNDLE_VERSION`.
pub(crate) const BUNDLE_VERSION: u32 = 1;

fn check_version(found: u32) -> Result<(), DocsKitError> {
    if found == BUNDLE_VERSION {
        Ok(())
    } else {
        Err(DocsKitError::BundleVersion {
            found,
            expected: BUNDLE_VERSION,
        })
    }
}

/// Everything the docs registry needs, parsed at build time.
#[derive(Deserialize)]
pub(crate) struct DocsBundle {
    version: u32,
    pub nav: NavConfig,
    /// Parsed pages keyed by content path, in `_nav.json` order.
    pub docs: Vec<(String, ParsedDoc)>,
    /// Parsed OpenAPI specs keyed by URL prefix.
    #[cfg(feature = "openapi")]
    #[serde(default)]
    pub openapi: Vec<(String, OpenApiSpec)>,
    /// Section-level search index with its lowercase fields precomputed.
    pub search: Vec<SearchEntry>,
}

impl DocsBundle {
    pub(crate) fn parse(json: &str) -> Result<Self, DocsKitError> {
        let bundle: Self = serde_json::from_str(json).map_err(DocsKitError::DocsBundleParse)?;
        check_version(bundle.version)?;
        Ok(bundle)
    }
}

/// Everything the blog registry needs, parsed at build time.
#[derive(Deserialize)]
pub(crate) struct BlogBundle {
    version: u32,
    #[serde(default)]
    pub authors: HashMap<String, Author>,
    #[serde(default)]
    pub categories: HashMap<String, BlogCategoryMetadata>,
    /// Published posts (drafts already dropped), newest first.
    pub posts: Vec<BlogPost>,
    pub search: Vec<BlogSearchEntry>,
}

impl BlogBundle {
    pub(crate) fn parse(json: &str) -> Result<Self, DocsKitError> {
        let bundle: Self = serde_json::from_str(json).map_err(DocsKitError::BlogBundleParse)?;
        check_version(bundle.version)?;
        Ok(bundle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `DocsBundle`/`BlogBundle` are deliberately not `Debug` (the whole site's
    /// content would be in the output), so unwrap the error side by hand.
    fn err<T>(result: Result<T, DocsKitError>) -> DocsKitError {
        match result {
            Ok(_) => panic!("expected an error"),
            Err(e) => e,
        }
    }

    #[test]
    fn a_bundle_from_another_build_version_is_rejected_by_name() {
        let e = err(DocsBundle::parse(
            r#"{"version":999,"nav":{"groups":[]},"docs":[],"search":[]}"#,
        ));
        assert!(matches!(e, DocsKitError::BundleVersion { found: 999, .. }));
        assert!(e.to_string().contains("dioxus-docs-kit-build"));
    }

    #[test]
    fn malformed_json_reports_a_parse_error() {
        assert!(matches!(
            err(DocsBundle::parse("{ not json")),
            DocsKitError::DocsBundleParse(_)
        ));
        assert!(matches!(
            err(BlogBundle::parse("{ not json")),
            DocsKitError::BlogBundleParse(_)
        ));
    }
}
