//! Shared search ranking used by the docs and blog registries.

/// Rank `index` entries against `query` using case-insensitive substring
/// matching: title matches first, then description matches, then content
/// matches. Returns an empty list for an empty query.
///
/// `fields` extracts `(title, description, content)` from an entry.
pub(crate) fn rank_by_fields<'a, T>(
    index: &'a [T],
    query: &str,
    fields: impl Fn(&'a T) -> (&'a str, &'a str, &'a str),
) -> Vec<&'a T> {
    let query = query.trim();
    if query.is_empty() {
        return Vec::new();
    }
    let q = query.to_lowercase();

    let mut title_matches: Vec<&T> = Vec::new();
    let mut desc_matches: Vec<&T> = Vec::new();
    let mut content_matches: Vec<&T> = Vec::new();

    for entry in index {
        let (title, description, content) = fields(entry);
        if title.to_lowercase().contains(&q) {
            title_matches.push(entry);
        } else if description.to_lowercase().contains(&q) {
            desc_matches.push(entry);
        } else if content.to_lowercase().contains(&q) {
            content_matches.push(entry);
        }
    }

    title_matches.extend(desc_matches);
    title_matches.extend(content_matches);
    title_matches
}
