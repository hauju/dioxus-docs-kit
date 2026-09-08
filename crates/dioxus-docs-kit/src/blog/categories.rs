use std::collections::{HashMap, HashSet};

use super::types::{BlogCategory, BlogCategoryMetadata};
use crate::DocsKitError;

pub(super) fn build_categories(
    tags: &[String],
    metadata: &HashMap<String, BlogCategoryMetadata>,
) -> Result<Vec<BlogCategory>, DocsKitError> {
    let mut slugs = HashSet::new();
    tags.iter().map(|tag| {
        let data = metadata.get(tag).cloned().unwrap_or_default();
        let slug = data.slug.unwrap_or_else(|| dioxus_mdx::slugify(tag));
        if slug.is_empty() || !slug.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(DocsKitError::BlogConfig(format!(
                "tag {tag:?} needs a non-empty category slug containing only letters, numbers or hyphens"
            )));
        }
        if !slugs.insert(slug.clone()) {
            return Err(DocsKitError::BlogConfig(format!(
                "duplicate category slug {slug:?}; set distinct slugs in _blog.json categories"
            )));
        }
        let title = data.title.filter(|s| !s.trim().is_empty()).unwrap_or_else(|| tag.clone());
        let description = data.description.unwrap_or_else(|| format!("Browse articles about {title}."));
        Ok(BlogCategory { tag: tag.clone(), slug, title, description, image: data.image })
    }).collect()
}

/// Percent-encode a path segment so non-ASCII slugs are valid in sitemap
/// `<loc>`, canonical and Open Graph URLs. Browsers show them decoded and the
/// router decodes route segments, so `get_category` still sees the raw slug.
pub(super) fn encode_path_segment(segment: &str) -> String {
    let mut out = String::with_capacity(segment.len());
    for byte in segment.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

pub(super) fn validate_base_path(path: &str) -> Result<(), DocsKitError> {
    if !path.starts_with('/')
        || path.starts_with("//")
        || path[1..]
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || path
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, '?' | '#' | '\\' | '%' | ':'))
    {
        return Err(DocsKitError::BlogConfig(format!(
            "category base path {path:?} must be a root-relative path such as /blog/categories"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colliding_and_empty_slugs_require_overrides() {
        let tags = vec!["C++".into(), "C#".into()];
        assert!(build_categories(&tags, &HashMap::new()).is_err());
        assert!(build_categories(&["!!!".into()], &HashMap::new()).is_err());
        let metadata = HashMap::from([(
            "C++".into(),
            BlogCategoryMetadata {
                slug: Some("cpp".into()),
                ..Default::default()
            },
        )]);
        let categories = build_categories(&tags, &metadata).unwrap();
        assert_eq!(categories[0].slug, "cpp");
        assert_eq!(categories[1].slug, "c");
        let invalid = HashMap::from([(
            "C++".into(),
            BlogCategoryMetadata {
                slug: Some("bad/slug".into()),
                ..Default::default()
            },
        )]);
        assert!(build_categories(&tags, &invalid).is_err());
    }

    #[test]
    fn category_paths_must_be_root_relative_and_unambiguous() {
        for path in [
            "", "/", "relative", "//host", "/a//b", "/a/../b", "/a?b", "/a#b", "/a%20b", "/a b",
        ] {
            assert!(validate_base_path(path).is_err(), "{path}");
        }
        assert!(validate_base_path("/blog/categories").is_ok());
        assert!(validate_base_path("/topics").is_ok());
    }
}
