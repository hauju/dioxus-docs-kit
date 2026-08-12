//! Shared utility functions for MDX component parsing.

use std::sync::LazyLock;

use regex::Regex;

/// Find the closing tag, handling nested tags of the same type.
pub(super) fn find_closing_tag(content: &str, tag_name: &str) -> Option<usize> {
    let open_tag = format!("<{}", tag_name);
    let close_tag = format!("</{}>", tag_name);

    let mut depth = 1;
    let mut pos = 0;

    while depth > 0 && pos < content.len() {
        let next_open = content[pos..].find(&open_tag).map(|i| i + pos);
        let next_close = content[pos..].find(&close_tag).map(|i| i + pos);

        match (next_open, next_close) {
            (Some(o), Some(c)) if o < c => {
                depth += 1;
                pos = o + open_tag.len();
            }
            (_, Some(c)) => {
                depth -= 1;
                if depth == 0 {
                    return Some(c);
                }
                pos = c + close_tag.len();
            }
            _ => return None,
        }
    }

    None
}

/// Advance past the first character, then seek the next occurrence of `tag`.
///
/// Skipping a whole character keeps the slice on a UTF-8 boundary, and always
/// advancing at least one character guarantees the caller's scan loop makes
/// progress even when the tag it lands on fails to parse.
pub(super) fn skip_to_next_tag<'a>(content: &'a str, tag: &str) -> Option<&'a str> {
    let skip = content.chars().next()?.len_utf8();
    content[skip..].find(tag).map(|idx| &content[skip + idx..])
}

/// Extract an attribute value from tag content.
///
/// Tokenizes the whole tag once against a cached regex rather than building a
/// per-attribute pattern. Besides avoiding a regex compile on every lookup,
/// this matches attribute *names*: the old `format!("{attr}=\"...\"")` had no
/// left boundary, so asking for `title` on `<Card subtitle="Sub" title="Real">`
/// matched the tail of `subtitle` and returned "Sub".
pub(super) fn extract_attr(tag_content: &str, attr_name: &str) -> Option<String> {
    static ATTR_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"([A-Za-z_:][-A-Za-z0-9_:.]*)="([^"]*)""#).unwrap());

    ATTR_RE
        .captures_iter(tag_content)
        .find(|caps| &caps[1] == attr_name)
        .map(|caps| caps[2].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_to_next_tag_handles_multibyte_leading_char() {
        // Slicing a fixed byte offset here would split 'Ü' mid-codepoint.
        assert_eq!(
            skip_to_next_tag("Überblick<Tab title=\"A\">", "<Tab"),
            Some("<Tab title=\"A\">")
        );
    }

    #[test]
    fn skip_to_next_tag_always_advances() {
        // A tag at offset 0 must not be returned unchanged, or the caller's
        // scan loop never terminates.
        assert_eq!(skip_to_next_tag("<Card a<Card b", "<Card"), Some("<Card b"));
        assert_eq!(skip_to_next_tag("<Card only", "<Card"), None);
    }

    #[test]
    fn skip_to_next_tag_handles_empty_input() {
        assert_eq!(skip_to_next_tag("", "<Card"), None);
    }

    #[test]
    fn extract_attr_matches_whole_attribute_names() {
        // `title="..."` also occurs as the tail of `subtitle="..."`; the lookup
        // must not match a suffix of a longer attribute name.
        let tag = r#" subtitle="Sub" title="Real""#;
        assert_eq!(extract_attr(tag, "title").as_deref(), Some("Real"));
        assert_eq!(extract_attr(tag, "subtitle").as_deref(), Some("Sub"));
    }

    #[test]
    fn extract_attr_returns_none_for_absent_attribute() {
        assert_eq!(extract_attr(r#" title="A""#, "icon"), None);
        assert_eq!(extract_attr("", "title"), None);
    }

    #[test]
    fn extract_attr_reads_each_attribute_of_a_tag() {
        let tag = r#" title="A Card" icon="star" href="/link""#;
        assert_eq!(extract_attr(tag, "title").as_deref(), Some("A Card"));
        assert_eq!(extract_attr(tag, "icon").as_deref(), Some("star"));
        assert_eq!(extract_attr(tag, "href").as_deref(), Some("/link"));
    }
}
