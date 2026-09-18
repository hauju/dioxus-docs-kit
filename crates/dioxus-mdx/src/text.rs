//! Heading/anchor helpers shared by the renderer, the table of contents and
//! the build-time bundle generator.
//!
//! Deliberately free of both `dioxus` and any regex engine so the build crate
//! (`dioxus-docs-kit-build`) can call the *same* code the browser does instead
//! of keeping a hand-synced copy of it.

/// Extract headers from markdown content for a table of contents.
///
/// Returns `(anchor id, title, level)` for every h2–h4 ATX heading. Fenced code
/// blocks are skipped so a `## Setup` inside a sample does not become a TOC
/// entry linking to an anchor the renderer never emits.
pub fn extract_headers(content: &str) -> Vec<(String, String, u8)> {
    let mut headers = Vec::new();
    let mut fence: Option<char> = None;

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            let marker = if trimmed.starts_with("```") { '`' } else { '~' };
            match fence {
                None => fence = Some(marker),
                Some(open) if open == marker => fence = None,
                Some(_) => {} // the other marker inside a fence is literal content
            }
            continue;
        }
        if fence.is_some() {
            continue;
        }

        if let Some((level, title)) = parse_atx_heading(line) {
            headers.push((slugify(title), title.to_string(), level));
        }
    }

    headers
}

/// Return `(level, text)` for an h2–h4 ATX heading line, or `None`.
///
/// Matches `^#{2,4}\s+\S` — 2–4 leading `#`, at least one space/tab, then
/// non-empty text. h1/h5/h6 are ignored (h1 is the page title, rendered by the
/// layout; h5/h6 are too deep to be worth an anchor).
pub fn parse_atx_heading(line: &str) -> Option<(u8, &str)> {
    let hashes = line.bytes().take_while(|&b| b == b'#').count();
    if !(2..=4).contains(&hashes) {
        return None;
    }
    let rest = &line[hashes..];
    if !rest.starts_with([' ', '\t']) {
        return None;
    }
    let text = rest.trim();
    if text.is_empty() {
        return None;
    }
    Some((hashes as u8, text))
}

/// Convert a title to a URL-friendly slug.
///
/// Standard HTML entities are decoded and markdown link syntax is reduced to
/// its text first, so the slug is identical whether the input is raw markdown
/// heading text (TOC, search index, build-time anchor checks) or the
/// HTML-escaped, tag-stripped heading the renderer injects ids from.
pub fn slugify(text: &str) -> String {
    let text = text
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&");
    let text = strip_markdown_links(&text);
    text.to_lowercase()
        .chars()
        .filter_map(|c| {
            if c.is_alphanumeric() {
                Some(c)
            } else if c.is_whitespace() || c == '-' || c == '_' || c == '.' {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Reduce markdown links/images `[text](url)` to their text. The renderer
/// slugs from HTML where the `<a>` tag is already stripped, so raw heading
/// text must shed the link syntax to produce the same slug.
pub fn strip_markdown_links(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(open) = rest.find('[') {
        if let Some(mid) = rest[open..].find("](") {
            let mid = open + mid;
            if let Some(close) = rest[mid..].find(')') {
                out.push_str(&rest[..open]);
                out.push_str(&rest[open + 1..mid]);
                rest = &rest[mid + close + 1..];
                continue;
            }
        }
        out.push_str(&rest[..=open]);
        rest = &rest[open + 1..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_normalizes_entities_and_links() {
        assert_eq!(slugify("Tips & Tricks"), "tips-tricks");
        assert_eq!(slugify("Tips &amp; Tricks"), "tips-tricks");
        assert_eq!(slugify("Q&A"), "qa");
        assert_eq!(slugify("Q&amp;A"), "qa");
        assert_eq!(slugify("a < b"), "a-b");
        assert_eq!(slugify("a &lt; b"), "a-b");
        assert_eq!(slugify("See [the docs](https://x.y/z)"), "see-the-docs");
        assert_eq!(slugify("See the docs"), "see-the-docs");
        assert_eq!(slugify("Use `cargo build`"), "use-cargo-build");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Getting Started!"), "getting-started");
        assert_eq!(slugify("API v1.0"), "api-v1-0");
    }

    #[test]
    fn test_extract_headers() {
        let content = r#"
## Introduction

Some text.

### Getting Started

More text.

## Configuration

### Advanced Options
"#;

        let headers = extract_headers(content);
        assert_eq!(headers.len(), 4);
        assert_eq!(
            headers[0],
            ("introduction".to_string(), "Introduction".to_string(), 2)
        );
        assert_eq!(
            headers[1],
            (
                "getting-started".to_string(),
                "Getting Started".to_string(),
                3
            )
        );
        assert_eq!(
            headers[2],
            ("configuration".to_string(), "Configuration".to_string(), 2)
        );
        assert_eq!(
            headers[3],
            (
                "advanced-options".to_string(),
                "Advanced Options".to_string(),
                3
            )
        );
    }

    #[test]
    fn extract_headers_covers_h2_to_h4_only() {
        let md = "# Title\n## Section One\n### Sub Section\n##### Too Deep\ntext\n";
        let titles: Vec<String> = extract_headers(md).into_iter().map(|(id, ..)| id).collect();
        assert_eq!(titles, vec!["section-one", "sub-section"]);
    }

    #[test]
    fn extract_headers_skips_headings_inside_code_fences() {
        let content = "## Real One\n\n```md\n## Fake Heading\n```\n\n### Real Two\n";
        let headers = extract_headers(content);
        let titles: Vec<&str> = headers.iter().map(|(_, t, _)| t.as_str()).collect();
        assert_eq!(titles, vec!["Real One", "Real Two"]);
    }

    #[test]
    fn extract_headers_skips_headings_inside_tilde_fences() {
        let content = "## Real\n\n~~~\n## Fake\n~~~\n";
        let headers = extract_headers(content);
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].1, "Real");
    }

    #[test]
    fn extract_headers_treats_other_marker_inside_fence_as_content() {
        // A ``` line inside a ~~~ fence is literal text, not a fence toggle.
        let content = "~~~\n```\n## Fake\n~~~\n\n## Real\n";
        let headers = extract_headers(content);
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].1, "Real");
    }
}
