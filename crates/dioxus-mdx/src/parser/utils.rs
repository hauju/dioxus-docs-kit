//! Shared utility functions for MDX component parsing.

use std::sync::LazyLock;

use crate::re::Regex;

/// A fenced code block located by [`find_fenced_blocks`].
pub(super) struct FencedBlock<'a> {
    /// Byte offset of the start of the opening fence line.
    pub(super) start: usize,
    /// Byte offset past the closing fence line's newline (or end of input).
    pub(super) end: usize,
    pub(super) language: Option<&'a str>,
    pub(super) filename: Option<&'a str>,
    /// Raw body between the fences (callers trim it).
    pub(super) code: &'a str,
    /// Leading `[ \t]` count of the opening fence line. A fence nested in a
    /// component (`<Step>`, `<Tab>`, ...) is usually indented with it, and per
    /// CommonMark that much indentation is stripped from every body line.
    pub(super) indent: usize,
}

impl FencedBlock<'_> {
    /// The body with up to [`indent`](Self::indent) leading blanks removed from
    /// each line, so an indented fence renders like one at column zero.
    pub(super) fn dedented_code(&self) -> String {
        if self.indent == 0 {
            return self.code.to_string();
        }
        self.code
            .split_inclusive('\n')
            .map(|line| {
                let strip = line
                    .bytes()
                    .take(self.indent)
                    .take_while(|b| *b == b' ' || *b == b'\t')
                    .count();
                &line[strip..]
            })
            .collect()
    }
}

/// Find all fenced code blocks (``` or ~~~) in `content`, line by line.
///
/// Fence tracking matches `toc.rs`/`registry.rs`: only a bare fence line with
/// the SAME marker closes a block, so the other marker inside a fence is
/// literal content, not a toggle. Everything else keeps the semantics of the
/// backtick-only regex this replaces: the language is `\w+`, the filename
/// separator is `[ \t]+` (NOT `\s+`, which would match across the newline and
/// swallow the first code line as the filename), CRLF never leaks a `\r` into
/// captures, a closing fence on the very next line does not close the block
/// (zero-line blocks are not blocks), and an unclosed fence yields no block.
pub(super) fn find_fenced_blocks(content: &str) -> Vec<FencedBlock<'_>> {
    struct OpenFence<'a> {
        marker: char,
        start: usize,
        body_start: usize,
        indent: usize,
        language: Option<&'a str>,
        filename: Option<&'a str>,
    }

    let mut blocks = Vec::new();
    let mut open: Option<OpenFence> = None;
    let mut line_start = 0;

    for line in content.split_inclusive('\n') {
        let line_end = line_start + line.len();
        let text = line.strip_suffix('\n').unwrap_or(line);
        let text = text.strip_suffix('\r').unwrap_or(text);

        match &open {
            Some(fence) => {
                // The line right after the opener cannot be the closer: the
                // old regex required a newline between them.
                if line_start > fence.body_start && is_closing_fence(text, fence.marker) {
                    blocks.push(FencedBlock {
                        start: fence.start,
                        end: line_end,
                        language: fence.language,
                        filename: fence.filename,
                        code: &content[fence.body_start..line_start],
                        indent: fence.indent,
                    });
                    open = None;
                }
            }
            None => {
                if let Some((marker, language, filename)) = parse_opening_fence(text) {
                    open = Some(OpenFence {
                        marker,
                        start: line_start,
                        body_start: line_end,
                        indent: text.len() - text.trim_start_matches([' ', '\t']).len(),
                        language,
                        filename,
                    });
                }
            }
        }
        line_start = line_end;
    }

    blocks
}

/// Split a fence line into its marker char and the text after it, if it is one.
fn split_fence_marker(text: &str) -> Option<(char, &str)> {
    let rest = text.trim_start_matches([' ', '\t']);
    if let Some(info) = rest.strip_prefix("```") {
        Some(('`', info))
    } else if let Some(info) = rest.strip_prefix("~~~") {
        Some(('~', info))
    } else {
        None
    }
}

/// Parse an opening fence line into `(marker, language, filename)`.
///
/// Returns `None` for lines that cannot open a fence: after the `\w+` language
/// only `[ \t]` may follow, so a marker followed by "rust,ignore" is ordinary
/// content, same as under the old regex.
fn parse_opening_fence(text: &str) -> Option<(char, Option<&str>, Option<&str>)> {
    let (marker, info) = split_fence_marker(text)?;

    let lang_end = info
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(info.len());
    let (language, rest) = info.split_at(lang_end);
    if !rest.is_empty() && !rest.starts_with([' ', '\t']) {
        return None;
    }
    let filename = rest.trim_start_matches([' ', '\t']);

    Some((
        marker,
        (!language.is_empty()).then_some(language),
        (!filename.is_empty()).then_some(filename),
    ))
}

/// A closing fence is the same marker on its own line, `[ \t]` padding aside.
fn is_closing_fence(text: &str, marker: char) -> bool {
    match split_fence_marker(text) {
        Some((m, rest)) => m == marker && rest.chars().all(|c| c == ' ' || c == '\t'),
        None => false,
    }
}

/// Strip the common leading `[ \t]` indentation from every line of `text`.
///
/// A component body extracted from between its tags keeps whatever indentation
/// nested it in the source (`<Step>` bodies in `docs/` are indented by four).
/// Trimming the body as a whole only unindents its *first* line, so the
/// paragraph after the first blank line still starts in column four and
/// CommonMark renders it as an indented code block. The prefix is measured over
/// non-blank lines only, so a blank line never forces it back to zero, and a
/// fenced block inside the body stays correct because
/// [`FencedBlock::dedented_code`] measures against its own opening fence.
pub(super) fn dedent(text: &str) -> String {
    let indent = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start_matches([' ', '\t']).len())
        .min()
        .unwrap_or(0);

    if indent == 0 {
        return text.to_string();
    }

    text.split_inclusive('\n')
        .map(|line| {
            let strip = line
                .bytes()
                .take(indent)
                .take_while(|b| *b == b' ' || *b == b'\t')
                .count();
            &line[strip..]
        })
        .collect()
}

/// Find the `>` that ends the opening tag starting at `from`.
///
/// A plain `find('>')` ends the tag at the first `>` anywhere, so a quoted
/// attribute value holding one — `type="Option<String>"` — cuts the tag in
/// half: the type is truncated, a bare `required` after it is lost, and the
/// tag's tail leaks into the body as literal text. Quoted regions are skipped;
/// `"` and `'` both quote, and the other kind inside one is ordinary text.
pub(super) fn find_tag_end(content: &str, from: usize) -> Option<usize> {
    let mut quote: Option<u8> = None;

    for (offset, byte) in content[from..].bytes().enumerate() {
        match quote {
            Some(open) if byte == open => quote = None,
            Some(_) => {}
            None => match byte {
                b'"' | b'\'' => quote = Some(byte),
                b'>' => return Some(from + offset),
                _ => {}
            },
        }
    }

    None
}

/// Byte ranges of the inline code spans (`` `code` ``) in `content`.
///
/// A component tag between backticks is prose *about* the component, not a use
/// of it, so the tag scanners have to look past it. Runs follow CommonMark — an
/// opening run of N backticks closes at the next run of exactly N — but only
/// within a single line: that covers every inline span in practice and stops a
/// stray backtick from swallowing the rest of the document. Lines inside a
/// fenced block are skipped, since their content is code already.
pub(super) fn find_inline_code_spans(content: &str) -> Vec<(usize, usize)> {
    let fences: Vec<(usize, usize)> = find_fenced_blocks(content)
        .iter()
        .map(|block| (block.start, block.end))
        .collect();

    let mut spans = Vec::new();
    let mut line_start = 0;

    for line in content.split_inclusive('\n') {
        if !in_ranges(&fences, line_start) {
            push_line_code_spans(line, line_start, &mut spans);
        }
        line_start += line.len();
    }

    spans
}

/// Collect one line's backtick spans, offset into the enclosing document.
fn push_line_code_spans(line: &str, offset: usize, spans: &mut Vec<(usize, usize)>) {
    let bytes = line.as_bytes();
    let run_end = |from: usize| {
        let mut end = from;
        while end < bytes.len() && bytes[end] == b'`' {
            end += 1;
        }
        end
    };

    let mut pos = 0;
    while pos < bytes.len() {
        if bytes[pos] != b'`' {
            pos += 1;
            continue;
        }

        let open_start = pos;
        pos = run_end(pos);
        let width = pos - open_start;

        // Seek a closing run of exactly `width`; a longer or shorter one is
        // ordinary text, and an unclosed run leaves `pos` past the opener so
        // the next run gets its turn as the opener.
        let mut probe = pos;
        while probe < bytes.len() {
            if bytes[probe] != b'`' {
                probe += 1;
                continue;
            }
            let close_start = probe;
            probe = run_end(probe);
            if probe - close_start == width {
                spans.push((offset + open_start, offset + probe));
                pos = probe;
                break;
            }
        }
    }
}

/// Whether `idx` falls inside any of the half-open `ranges`.
fn in_ranges(ranges: &[(usize, usize)], idx: usize) -> bool {
    ranges.iter().any(|&(start, end)| idx >= start && idx < end)
}

/// Find the closing tag, handling nested tags of the same type.
pub(super) fn find_closing_tag(content: &str, tag_name: &str) -> Option<usize> {
    let open_tag = format!("<{}", tag_name);
    let close_tag = format!("</{}>", tag_name);
    let code = find_inline_code_spans(content);

    let mut depth = 1;
    let mut pos = 0;

    while depth > 0 && pos < content.len() {
        let next_open = find_open_tag(content, pos, &open_tag, &code);
        let next_close = find_outside_code(content, pos, &close_tag, &code);

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

/// Find the next `<Tag` opening at or after `from`, requiring a name boundary.
///
/// A plain substring search counts `<Tabs` as an opening `<Tab` and `<CardGroup`
/// as an opening `<Card`, inflating the nesting depth — while the closing tags
/// (`</Tabs>`, `</CardGroup>`) never match `</Tab>` / `</Card>`, so the depth
/// never balances again and the outer element is silently dropped.
fn find_open_tag(
    content: &str,
    from: usize,
    open_tag: &str,
    code: &[(usize, usize)],
) -> Option<usize> {
    let mut search = from;

    while let Some(rel) = content[search..].find(open_tag) {
        let idx = search + rel;
        let after = idx + open_tag.len();
        let ends_here = content[after..]
            .chars()
            .next()
            .is_none_or(|ch| ch.is_whitespace() || ch == '>' || ch == '/');
        if ends_here && !in_ranges(code, idx) {
            return Some(idx);
        }
        search = after;
    }

    None
}

/// Find the next `needle` at or after `from` that is not inside an inline code
/// span, so a `` `</Card>` `` written in prose does not close the real element.
fn find_outside_code(
    content: &str,
    from: usize,
    needle: &str,
    code: &[(usize, usize)],
) -> Option<usize> {
    let mut search = from;

    while let Some(rel) = content[search..].find(needle) {
        let idx = search + rel;
        if !in_ranges(code, idx) {
            return Some(idx);
        }
        search = idx + needle.len();
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

    #[test]
    fn find_closing_tag_ignores_longer_tag_names() {
        // `<Tabs` must not count as an opening `<Tab`.
        let inner = "outer<Tabs><Tab>x</Tab></Tabs></Tab>rest";
        let close = find_closing_tag(inner, "Tab").expect("closing </Tab>");
        assert_eq!(&inner[close..], "</Tab>rest");
    }

    #[test]
    fn find_closing_tag_still_counts_real_nesting() {
        let inner = "<Tab>inner</Tab></Tab>rest";
        let close = find_closing_tag(inner, "Tab").expect("closing </Tab>");
        assert_eq!(&inner[close..], "</Tab>rest");
    }

    #[test]
    fn find_closing_tag_ignores_tags_in_inline_code() {
        // A `</Card>` written in prose must not close the real element, and a
        // `<Card>` in prose must not push the nesting depth up.
        let inner = "a `</Card>` in prose</Card>rest";
        let close = find_closing_tag(inner, "Card").expect("closing </Card>");
        assert_eq!(&inner[close..], "</Card>rest");

        let nested = "a `<Card>` in prose</Card>rest";
        let close = find_closing_tag(nested, "Card").expect("closing </Card>");
        assert_eq!(&nested[close..], "</Card>rest");
    }

    #[test]
    fn dedent_strips_the_common_prefix_of_non_blank_lines() {
        assert_eq!(dedent("    one\n\n    two\n"), "one\n\ntwo\n");
        // The deeper line keeps its relative indentation.
        assert_eq!(dedent("  one\n      two"), "one\n    two");
    }

    #[test]
    fn dedent_leaves_unindented_and_empty_text_alone() {
        assert_eq!(dedent("one\n    two"), "one\n    two");
        assert_eq!(dedent(""), "");
        assert_eq!(dedent("\n  \n"), "\n  \n");
    }

    #[test]
    fn find_tag_end_skips_quoted_angle_brackets() {
        let tag = r#"<ParamField path="x" type="Option<String>" required>Body"#;
        let end = find_tag_end(tag, 0).expect("tag end");
        assert_eq!(
            &tag[..=end],
            r#"<ParamField path="x" type="Option<String>" required>"#
        );

        let single = "<Card title='a>b'>body";
        assert_eq!(find_tag_end(single, 0), Some("<Card title='a>b'".len()));
    }

    #[test]
    fn find_tag_end_reports_no_end_for_an_unterminated_tag() {
        assert_eq!(find_tag_end("<Card title=\"a", 0), None);
    }

    #[test]
    fn find_inline_code_spans_matches_runs_of_equal_width() {
        let md = "Use `<OpenAPI>` and ``a ` b`` here.";
        let spans = find_inline_code_spans(md);
        let found: Vec<&str> = spans.iter().map(|&(s, e)| &md[s..e]).collect();
        assert_eq!(found, vec!["`<OpenAPI>`", "``a ` b``"]);
    }

    #[test]
    fn find_inline_code_spans_skips_fences_and_unclosed_runs() {
        // The fence body is code already, and a lone backtick closes nothing.
        let md = "```rust\nlet a = `x`;\n```\nprose ` with one tick\n";
        assert!(find_inline_code_spans(md).is_empty());
    }
}
