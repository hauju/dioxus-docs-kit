//! Shared utility functions for MDX component parsing.

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
pub(super) fn extract_attr(tag_content: &str, attr_name: &str) -> Option<String> {
    let pattern = format!(r#"{}="([^"]*)""#, attr_name);
    let re = Regex::new(&pattern).ok()?;
    re.captures(tag_content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
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
}
