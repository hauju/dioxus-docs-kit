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

/// Extract an attribute value from tag content.
pub(super) fn extract_attr(tag_content: &str, attr_name: &str) -> Option<String> {
    let pattern = format!(r#"{}="([^"]*)""#, attr_name);
    let re = Regex::new(&pattern).ok()?;
    re.captures(tag_content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}
