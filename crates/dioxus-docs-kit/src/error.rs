//! Error types for registry construction.

use dioxus_mdx::OpenApiError;

/// Errors produced when building a [`DocsRegistry`](crate::DocsRegistry) or
/// [`BlogRegistry`](crate::blog::BlogRegistry) from configuration.
#[derive(Debug)]
pub enum DocsKitError {
    /// `_nav.json` failed to parse.
    NavParse(serde_json::Error),
    /// `_blog.json` failed to parse.
    BlogManifestParse(serde_json::Error),
    /// An OpenAPI spec failed to parse.
    OpenApi {
        /// URL prefix the spec was registered under.
        prefix: String,
        /// The underlying parse error.
        error: OpenApiError,
    },
}

impl std::fmt::Display for DocsKitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NavParse(e) => write!(f, "failed to parse _nav.json: {e}"),
            Self::BlogManifestParse(e) => write!(f, "failed to parse _blog.json: {e}"),
            Self::OpenApi { prefix, error } => write!(
                f,
                "failed to parse OpenAPI spec for prefix \"{prefix}\": {error}"
            ),
        }
    }
}

impl std::error::Error for DocsKitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NavParse(e) | Self::BlogManifestParse(e) => Some(e),
            Self::OpenApi { error, .. } => Some(error),
        }
    }
}
