//! Shared search ranking and snippet extraction for the docs and blog
//! registries and their modals.
//!
//! Matching is case-insensitive substring: the query is split on whitespace and
//! an entry matches only when *every* term is found in at least one of its
//! fields (multi-term AND). Entries are scored by which field a term lands in
//! (title > heading > description > body), with a bonus for word-boundary /
//! prefix hits over mid-word ones and a mild earlier-position tiebreak. Equal
//! scores keep the index (navigation) order. There is no fuzzy/typo tolerance.

/// A field's tier, ordered highest-weight first.
#[derive(Clone, Copy)]
pub(crate) enum Tier {
    Title,
    Heading,
    Description,
    Body,
}

impl Tier {
    /// Base weight. The gap between tiers exceeds the maximum bonus/penalty
    /// swing, so a higher-tier hit always outranks a lower-tier one on a single
    /// term.
    fn weight(self) -> i32 {
        match self {
            Tier::Title => 4000,
            Tier::Heading => 3000,
            Tier::Description => 2000,
            Tier::Body => 1000,
        }
    }
}

/// A precomputed-lowercase field an entry exposes for matching.
pub(crate) struct Field<'a> {
    tier: Tier,
    lower: &'a str,
}

impl<'a> Field<'a> {
    pub(crate) fn title(lower: &'a str) -> Self {
        Self {
            tier: Tier::Title,
            lower,
        }
    }
    pub(crate) fn heading(lower: &'a str) -> Self {
        Self {
            tier: Tier::Heading,
            lower,
        }
    }
    pub(crate) fn description(lower: &'a str) -> Self {
        Self {
            tier: Tier::Description,
            lower,
        }
    }
    pub(crate) fn body(lower: &'a str) -> Self {
        Self {
            tier: Tier::Body,
            lower,
        }
    }
}

/// Extra score for a hit that starts a word (match at text start, or preceded
/// by a non-alphanumeric char) over a mid-word substring hit.
const BOUNDARY_BONUS: i32 = 250;
/// Cap on the earlier-position tiebreak so it never crosses tier boundaries.
const POSITION_CAP: i32 = 200;

/// Default snippet window in characters, centred on the first match.
pub(crate) const SNIPPET_WINDOW: usize = 100;

/// One char in, one char out, so char offsets stay aligned with the original
/// text (snippet highlight ranges rely on this).
fn lower_char(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

/// Lowercase a string for case-insensitive matching.
///
/// Rare multi-char lowercase expansions (e.g. `İ`) fall back to the original
/// char to keep the 1:1 char mapping; locale/typo folding is out of scope.
pub(crate) fn search_lower(s: &str) -> String {
    s.chars().map(lower_char).collect()
}

/// Split a query into lowercased, whitespace-delimited terms.
pub(crate) fn split_terms(query: &str) -> Vec<String> {
    query.split_whitespace().map(search_lower).collect()
}

/// Score a single term against a single field, or `None` if it does not occur.
fn score_term_in_field(term: &str, field: &Field<'_>) -> Option<i32> {
    let hay = field.lower;
    let pos = hay.find(term)?;
    // Word-boundary bonus: match at the start, or right after a non-alphanumeric.
    let boundary = hay[..pos]
        .chars()
        .next_back()
        .is_none_or(|c| !c.is_alphanumeric());
    // Earlier position is a mild tiebreak, capped so it stays within the tier.
    let penalty = (hay[..pos].chars().count() as i32).min(POSITION_CAP);
    let bonus = if boundary { BOUNDARY_BONUS } else { 0 };
    Some(field.tier.weight() + bonus - penalty)
}

/// Rank `index` entries against `query`.
///
/// `collect_fields` pushes an entry's matchable fields (any order) into the
/// reused buffer. Returns matching entries best-scored first; entries with an
/// equal score keep their index (navigation) order. Empty query → empty result.
pub(crate) fn rank<'a, T>(
    index: &'a [T],
    query: &str,
    mut collect_fields: impl FnMut(&'a T, &mut Vec<Field<'a>>),
) -> Vec<&'a T> {
    let terms = split_terms(query);
    if terms.is_empty() {
        return Vec::new();
    }

    // Reused across entries to keep per-query allocations to O(results).
    let mut fields: Vec<Field<'a>> = Vec::new();
    let mut scored: Vec<(i32, usize, &'a T)> = Vec::new();

    for (idx, entry) in index.iter().enumerate() {
        fields.clear();
        collect_fields(entry, &mut fields);

        let mut total = 0i32;
        let mut all_matched = true;
        for term in &terms {
            match fields
                .iter()
                .filter_map(|f| score_term_in_field(term, f))
                .max()
            {
                Some(score) => total += score,
                None => {
                    all_matched = false;
                    break;
                }
            }
        }

        if all_matched {
            scored.push((total, idx, entry));
        }
    }

    // Higher score first; equal scores keep index (navigation) order.
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    scored.into_iter().map(|(_, _, entry)| entry).collect()
}

/// A run of snippet text, flagged when it is (part of) a matched term so the UI
/// can highlight it.
#[derive(Clone, PartialEq)]
pub(crate) struct SnippetSegment {
    pub text: String,
    pub highlight: bool,
}

impl SnippetSegment {
    fn plain(chars: &[char]) -> Self {
        Self {
            text: chars.iter().collect(),
            highlight: false,
        }
    }

    fn ellipsis() -> Self {
        Self {
            text: "…".to_string(),
            highlight: false,
        }
    }

    fn highlighted(chars: &[char]) -> Self {
        Self {
            text: chars.iter().collect(),
            highlight: true,
        }
    }
}

/// Build a highlighted snippet from `text` around the first `terms` match.
///
/// Returns segments — plain runs interleaved with highlighted matched runs —
/// for a window of ~`window` chars centred on the earliest match, with leading
/// / trailing ellipses when the text is clipped. Returns an empty vec when no
/// term matches (callers render no snippet then). Matching uses the same
/// case-insensitive fold as ranking.
pub(crate) fn build_snippet(text: &str, terms: &[String], window: usize) -> Vec<SnippetSegment> {
    if text.is_empty() || terms.is_empty() {
        return Vec::new();
    }

    let chars: Vec<char> = text.chars().collect();
    let lower: Vec<char> = chars.iter().map(|&c| lower_char(c)).collect();

    // Collect all term match ranges (char indices).
    let mut matches: Vec<(usize, usize)> = Vec::new();
    for term in terms {
        let needle: Vec<char> = term.chars().collect();
        if needle.is_empty() {
            continue;
        }
        let mut start = 0;
        while start + needle.len() <= lower.len() {
            if lower[start..start + needle.len()] == needle[..] {
                matches.push((start, start + needle.len()));
                start += needle.len();
            } else {
                start += 1;
            }
        }
    }
    if matches.is_empty() {
        return Vec::new();
    }
    matches.sort_by_key(|&(s, _)| s);
    let first = matches[0].0;

    // Window centred on the first match, clamped to the text.
    let half = window / 2;
    let win_start = first.saturating_sub(half);
    let win_end = (win_start + window).min(chars.len());

    let mut segments: Vec<SnippetSegment> = Vec::new();
    if win_start > 0 {
        segments.push(SnippetSegment::ellipsis());
    }

    let mut cursor = win_start;
    for &(ms, me) in &matches {
        let ms = ms.max(win_start);
        let me = me.min(win_end);
        // Skip matches outside the window or already covered (overlaps).
        if me <= cursor || ms >= win_end {
            continue;
        }
        let ms = ms.max(cursor);
        if ms >= me {
            continue;
        }
        if ms > cursor {
            segments.push(SnippetSegment::plain(&chars[cursor..ms]));
        }
        segments.push(SnippetSegment::highlighted(&chars[ms..me]));
        cursor = me;
    }
    if cursor < win_end {
        segments.push(SnippetSegment::plain(&chars[cursor..win_end]));
    }
    if win_end < chars.len() {
        segments.push(SnippetSegment::ellipsis());
    }

    segments
}

/// Collapse a section's raw markdown into plain-ish text for matching and
/// snippet display: fenced code blocks are dropped, `[text](url)` links reduce
/// to their text, the noisiest inline markers are stripped, and whitespace is
/// collapsed to single spaces. Intentionally lightweight — not a full markdown
/// renderer.
pub(crate) fn clean_markdown(md: &str) -> String {
    // Drop fenced code blocks wholesale (noise, and their `##` lines are not
    // real headings).
    let mut no_code = String::with_capacity(md.len());
    let mut fence: Option<char> = None;
    for line in md.lines() {
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
        no_code.push_str(line);
        no_code.push('\n');
    }

    // Collapse `[text](url)` to `text`, strip the noisiest inline markers, and
    // squeeze runs of whitespace to a single space.
    let mut out = String::with_capacity(no_code.len());
    let mut prev_space = false;
    let mut chars = no_code.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '[' {
            let mut text = String::new();
            for tc in chars.by_ref() {
                if tc == ']' {
                    break;
                }
                text.push(tc);
            }
            if chars.peek() == Some(&'(') {
                chars.next();
                for uc in chars.by_ref() {
                    if uc == ')' {
                        break;
                    }
                }
            }
            for tc in text.chars() {
                push_clean(&mut out, tc, &mut prev_space);
            }
        } else {
            push_clean(&mut out, c, &mut prev_space);
        }
    }

    out.trim().to_string()
}

fn push_clean(out: &mut String, c: char, prev_space: &mut bool) {
    if matches!(c, '#' | '*' | '_' | '`' | '>' | '|' | '\\') {
        return;
    }
    if c.is_whitespace() {
        if !*prev_space {
            out.push(' ');
            *prev_space = true;
        }
    } else {
        out.push(c);
        *prev_space = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal entry for exercising the generic ranking engine.
    struct Entry {
        title: String,
        heading: String,
        description: String,
        body: String,
    }

    impl Entry {
        fn new(title: &str, heading: &str, description: &str, body: &str) -> Self {
            Self {
                title: search_lower(title),
                heading: search_lower(heading),
                description: search_lower(description),
                body: search_lower(body),
            }
        }
    }

    fn rank_entries<'a>(index: &'a [Entry], query: &str) -> Vec<&'a Entry> {
        rank(index, query, |e, buf| {
            buf.push(Field::title(&e.title));
            buf.push(Field::heading(&e.heading));
            buf.push(Field::description(&e.description));
            buf.push(Field::body(&e.body));
        })
    }

    #[test]
    fn search_lower_folds_case_one_char_per_char() {
        assert_eq!(search_lower("HELLO"), "hello");
        assert_eq!(search_lower("CafÉ"), "café");
        // Length preserved so highlight offsets stay aligned.
        assert_eq!(search_lower("AbC").chars().count(), 3);
    }

    #[test]
    fn split_terms_lowercases_and_splits_on_whitespace() {
        assert_eq!(split_terms("  Foo   BAR "), vec!["foo", "bar"]);
        assert!(split_terms("   ").is_empty());
    }

    #[test]
    fn tiers_rank_title_over_heading_over_description_over_body() {
        let index = vec![
            Entry::new("nope", "nope", "nope", "alpha here"), // body
            Entry::new("nope", "nope", "alpha here", "nope"), // description
            Entry::new("nope", "alpha here", "nope", "nope"), // heading
            Entry::new("alpha here", "nope", "nope", "nope"), // title
        ];
        let ranked = rank_entries(&index, "alpha");
        let titles: Vec<&str> = ranked.iter().map(|e| e.title.as_str()).collect();
        assert_eq!(titles, vec!["alpha here", "nope", "nope", "nope"]);
        // Confirm the ordering is title, heading, description, body by anchor.
        assert_eq!(ranked[1].heading, "alpha here");
        assert_eq!(ranked[2].description, "alpha here");
        assert_eq!(ranked[3].body, "alpha here");
    }

    #[test]
    fn multi_term_requires_every_term_to_match() {
        let index = vec![
            Entry::new("Rust guide", "", "", "async runtime details"), // both terms
            Entry::new("Rust guide", "", "", "no second word"),        // only "rust"
            Entry::new("Python", "", "", "async runtime"),             // only "async"
        ];
        let ranked = rank_entries(&index, "rust async");
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].title, "rust guide");
    }

    #[test]
    fn word_boundary_hit_outranks_mid_word_hit_in_same_tier() {
        let index = vec![
            Entry::new("t", "", "", "please locate the scatter plot"), // "cat" mid-word
            Entry::new("t", "", "", "the cat sat down"),               // "cat" at boundary
        ];
        let ranked = rank_entries(&index, "cat");
        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0].body, "the cat sat down");
    }

    #[test]
    fn earlier_position_breaks_ties_within_a_tier() {
        let index = vec![
            Entry::new("t", "", "", "a b c d e f g cat"), // later
            Entry::new("t", "", "", "cat first here"),    // earlier
        ];
        let ranked = rank_entries(&index, "cat");
        assert_eq!(ranked[0].body, "cat first here");
    }

    #[test]
    fn empty_query_returns_nothing() {
        let index = vec![Entry::new("alpha", "", "", "")];
        assert!(rank_entries(&index, "   ").is_empty());
    }

    #[test]
    fn snippet_returns_whole_short_text_without_ellipses() {
        let segs = build_snippet("the cat sat", &["cat".to_string()], SNIPPET_WINDOW);
        // No clipping → no ellipsis segments.
        assert!(!segs.iter().any(|s| s.text == "…"));
        let joined: String = segs.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(joined, "the cat sat");
        // The matched run is highlighted, the rest is not.
        let hl: Vec<&str> = segs
            .iter()
            .filter(|s| s.highlight)
            .map(|s| s.text.as_str())
            .collect();
        assert_eq!(hl, vec!["cat"]);
    }

    #[test]
    fn snippet_clips_and_adds_ellipses_around_a_deep_match() {
        let text: String = "lorem ".repeat(40) + "TARGET tail";
        let segs = build_snippet(&text, &["target".to_string()], 40);
        assert_eq!(segs.first().unwrap().text, "…");
        // The highlight preserves the original case from the source text.
        assert!(segs.iter().any(|s| s.highlight && s.text == "TARGET"));
        let joined: String = segs.iter().map(|s| s.text.as_str()).collect();
        assert!(joined.chars().count() <= 40 + 2, "window + two ellipses");
    }

    #[test]
    fn snippet_highlights_multiple_terms_in_order() {
        let segs = build_snippet(
            "alpha then beta then gamma",
            &["alpha".to_string(), "beta".to_string()],
            SNIPPET_WINDOW,
        );
        let hl: Vec<&str> = segs
            .iter()
            .filter(|s| s.highlight)
            .map(|s| s.text.as_str())
            .collect();
        assert_eq!(hl, vec!["alpha", "beta"]);
    }

    #[test]
    fn snippet_empty_when_no_term_matches() {
        assert!(build_snippet("nothing here", &["zzz".to_string()], SNIPPET_WINDOW).is_empty());
    }

    #[test]
    fn clean_markdown_drops_code_fences_and_strips_markers() {
        let md =
            "Intro **bold** text\n\n```rust\nlet x = 1;\n```\n\nSee [the docs](https://x.y) now";
        let cleaned = clean_markdown(md);
        assert!(!cleaned.contains("let x"), "code fence body dropped");
        assert!(!cleaned.contains('*'), "emphasis markers stripped");
        assert!(cleaned.contains("the docs"), "link text kept");
        assert!(!cleaned.contains("https://"), "link target dropped");
        assert!(!cleaned.contains('\n'), "whitespace collapsed");
    }
}
