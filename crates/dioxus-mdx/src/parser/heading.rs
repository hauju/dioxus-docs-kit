//! Strip a duplicate top-of-document H1 heading from MDX bodies.
//!
//! Consumers of this crate render the page title (from frontmatter) inside their
//! own `<h1>` — e.g. `BlogPostView`'s header or `document::Title` in
//! `DocsPageContent`. If the MDX body *also* starts with `# Title`, the page
//! ends up with two `<h1>` elements, which trips SEO audits with "Multiple H1
//! elements detected". Frontmatter is the source of truth, so we drop the
//! leading body H1 instead of promoting it.

use regex::Regex;

/// Strip a single leading H1 (ATX `# Title` or setext `Title\n===`) from `body`.
///
/// Only the first heading is touched. Any H1 that appears later in the document
/// is preserved untouched. If the body's first non-blank line is not an H1,
/// the input is returned unchanged.
pub fn strip_leading_h1(body: &str) -> &str {
    // Skip leading *blank lines* only — preserve in-line indentation, since a
    // 4-space indent turns `    # Hello` into a code block, not a heading.
    let bytes = body.as_bytes();
    let mut cursor = 0usize;
    loop {
        let mut i = cursor;
        while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b'\r') {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'\n' {
            cursor = i + 1;
        } else {
            break;
        }
    }

    let rest = &body[cursor..];
    if rest.is_empty() {
        return body;
    }

    let atx = Regex::new(r"^ {0,3}#[ \t]+\S[^\n]*(?:\r?\n|$)").unwrap();
    if let Some(m) = atx.find(rest) {
        return &rest[m.end()..];
    }

    let setext = Regex::new(r"^[^\n]+\r?\n {0,3}=+[ \t]*(?:\r?\n|$)").unwrap();
    if let Some(m) = setext.find(rest) {
        return &rest[m.end()..];
    }

    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_leading_atx_h1() {
        let body = "# Hello\n\nbody text\n";
        // The blank line that separated the heading from the body remains and
        // is harmlessly trimmed by `parse_content` downstream.
        assert_eq!(strip_leading_h1(body), "\nbody text\n");
    }

    #[test]
    fn strips_leading_atx_h1_after_blank_lines() {
        let body = "\n\n# Hello\n\nbody text\n";
        assert_eq!(strip_leading_h1(body), "\nbody text\n");
    }

    #[test]
    fn strips_leading_atx_h1_without_trailing_newline() {
        let body = "# Hello";
        assert_eq!(strip_leading_h1(body), "");
    }

    #[test]
    fn strips_leading_setext_h1() {
        let body = "Hello\n=====\n\nbody text\n";
        assert_eq!(strip_leading_h1(body), "\nbody text\n");
    }

    #[test]
    fn preserves_body_without_leading_h1() {
        let body = "Just a paragraph\n\n## Subheading\n";
        assert_eq!(strip_leading_h1(body), body);
    }

    #[test]
    fn preserves_mid_document_h1() {
        let body = "intro paragraph\n\n# Later heading\n\nmore body\n";
        assert_eq!(strip_leading_h1(body), body);
    }

    #[test]
    fn preserves_h2_at_start() {
        let body = "## Subheading first\n\nbody\n";
        assert_eq!(strip_leading_h1(body), body);
    }

    #[test]
    fn preserves_atx_without_space() {
        // `#Hello` is not a valid ATX heading per CommonMark.
        let body = "#Hello\n\nbody\n";
        assert_eq!(strip_leading_h1(body), body);
    }

    #[test]
    fn preserves_deeply_indented_hash() {
        // 4+ spaces of indent makes this a code block, not a heading.
        let body = "    # Hello\n\nbody\n";
        assert_eq!(strip_leading_h1(body), body);
    }

    #[test]
    fn handles_empty_body() {
        assert_eq!(strip_leading_h1(""), "");
        assert_eq!(strip_leading_h1("\n\n"), "\n\n");
    }
}
