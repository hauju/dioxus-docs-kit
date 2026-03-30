//! Builder for constructing a `BlogRegistry`.

use crate::blog::registry::BlogRegistry;
use crate::config::ThemeConfig;
use std::collections::HashMap;

/// Builder for constructing a [`BlogRegistry`].
///
/// # Example
///
/// ```rust,ignore
/// let registry = BlogConfig::new(include_str!("../blog/_blog.json"), blog_content_map())
///     .with_posts_per_page(9)
///     .with_theme_toggle("light", "dark", "dark")
///     .build();
/// ```
pub struct BlogConfig {
    manifest_json: String,
    content_map: HashMap<&'static str, &'static str>,
    posts_per_page: usize,
    date_format: String,
    theme: Option<ThemeConfig>,
}

impl BlogConfig {
    /// Create a new builder from a `_blog.json` string and a content map.
    pub fn new(manifest_json: &str, content_map: HashMap<&'static str, &'static str>) -> Self {
        Self {
            manifest_json: manifest_json.to_string(),
            content_map,
            posts_per_page: 9,
            date_format: "%B %d, %Y".to_string(),
            theme: None,
        }
    }

    /// Set the number of posts per page for pagination (default: 9).
    pub fn with_posts_per_page(mut self, n: usize) -> Self {
        self.posts_per_page = n;
        self
    }

    /// Set the date display format (default: "%B %d, %Y").
    pub fn with_date_format(mut self, fmt: &str) -> Self {
        self.date_format = fmt.to_string();
        self
    }

    /// Set a single theme (no toggle button).
    pub fn with_theme(mut self, theme: &str) -> Self {
        self.theme = Some(ThemeConfig {
            default_theme: theme.to_string(),
            toggle_themes: None,
            storage_key: "docs-theme".to_string(),
        });
        self
    }

    /// Enable a light/dark theme toggle.
    pub fn with_theme_toggle(mut self, light: &str, dark: &str, default: &str) -> Self {
        self.theme = Some(ThemeConfig {
            default_theme: default.to_string(),
            toggle_themes: Some((light.to_string(), dark.to_string())),
            storage_key: "docs-theme".to_string(),
        });
        self
    }

    /// Build the [`BlogRegistry`].
    pub fn build(self) -> BlogRegistry {
        BlogRegistry::from_config(self)
    }

    pub(crate) fn manifest_json(&self) -> &str {
        &self.manifest_json
    }

    pub(crate) fn content_map(&self) -> &HashMap<&'static str, &'static str> {
        &self.content_map
    }

    pub(crate) fn posts_per_page(&self) -> usize {
        self.posts_per_page
    }

    pub(crate) fn date_format(&self) -> &str {
        &self.date_format
    }

    pub(crate) fn theme_config(&self) -> Option<&ThemeConfig> {
        self.theme.as_ref()
    }
}
