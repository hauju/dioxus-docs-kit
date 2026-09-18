//! HTML / XML / SVG lexer.
//!
//! The bodies of `<script>` and `<style>` are skipped as plain text rather
//! than lexed as JavaScript or CSS: recursion would double the code size for
//! a construct that barely appears in documentation snippets, and skipping
//! them also keeps a `<` inside JavaScript from opening a bogus tag.

use crate::Kind;
use crate::emit::Emit;
use crate::util::{
    char_len, is_ident, is_ident_start, scan_quoted, skip_ws, starts_with, starts_with_ci,
};

pub(crate) fn lex(out: &mut Emit<'_>) {
    let b = out.bytes();
    let src = out.src();
    let n = b.len();
    let mut i = 0;

    while i < n {
        match b[i] {
            b'<' if starts_with(b, i, b"<!--") => {
                let e = find(b, i + 4, b"-->").map_or(n, |p| p + 3);
                out.push(i, e, Kind::Comment);
                i = e;
            }
            b'<' if starts_with(b, i, b"<![CDATA[") => {
                let e = find(b, i + 9, b"]]>").map_or(n, |p| p + 3);
                out.push(i, e, Kind::String);
                i = e;
            }
            b'<' if starts_with_ci(b, i, b"<!doctype") => {
                let e = find(b, i, b">").map_or(n, |p| p + 1);
                out.push(i, e, Kind::Keyword);
                i = e;
            }
            b'<' if starts_with(b, i, b"<?") => {
                let e = find(b, i + 2, b"?>").map_or(n, |p| p + 2);
                out.push(i, e, Kind::Keyword);
                i = e;
            }
            b'<' if b
                .get(i + 1)
                .is_some_and(|&c| c == b'/' || is_ident_start(c)) =>
            {
                i = tag(out, b, src, i);
            }
            b'&' => {
                let e = entity(b, i);
                if e > i + 1 {
                    out.push(i, e, Kind::Constant);
                }
                i = e;
            }
            c => i += char_len(c),
        }
    }
}

fn tag(out: &mut Emit<'_>, b: &[u8], src: &str, i: usize) -> usize {
    let n = b.len();
    let mut j = i + 1;
    let closing = b[j] == b'/';
    if closing {
        j += 1;
    }
    out.push(i, j, Kind::Punctuation);

    let name_end = name_end(b, j);
    out.push(j, name_end, Kind::Tag);
    let name = &src[j..name_end];
    j = name_end;

    let mut after_eq = false;
    while j < n {
        j = skip_ws(b, j);
        if j >= n {
            break;
        }
        match b[j] {
            b'>' => {
                out.push(j, j + 1, Kind::Punctuation);
                j += 1;
                break;
            }
            b'/' if b.get(j + 1) == Some(&b'>') => {
                out.push(j, j + 2, Kind::Punctuation);
                j += 2;
                break;
            }
            b'=' => {
                out.push(j, j + 1, Kind::Operator);
                after_eq = true;
                j += 1;
            }
            c @ (b'"' | b'\'') => {
                let e = scan_quoted(b, j, c, false, true);
                out.push(j, e, Kind::String);
                after_eq = false;
                j = e;
            }
            c if is_attr_char(c) => {
                let e = attr_end(b, j);
                out.push(
                    j,
                    e,
                    if after_eq {
                        Kind::String
                    } else {
                        Kind::Attribute
                    },
                );
                after_eq = false;
                j = e;
            }
            _ => {
                out.push(j, j + 1, Kind::Operator);
                after_eq = false;
                j += 1;
            }
        }
    }

    if !closing && (name.eq_ignore_ascii_case("script") || name.eq_ignore_ascii_case("style")) {
        let close = if name.eq_ignore_ascii_case("script") {
            b"</script".as_slice()
        } else {
            b"</style".as_slice()
        };
        j = find_ci(b, j, close).unwrap_or(n);
    }
    j
}

fn name_end(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && (is_ident(b[i]) || matches!(b[i], b'-' | b':' | b'.')) {
        i += 1;
    }
    i
}

fn is_attr_char(c: u8) -> bool {
    is_ident(c)
        || matches!(
            c,
            b'-' | b':' | b'.' | b'@' | b'#' | b'{' | b'}' | b'%' | b'/'
        )
}

fn attr_end(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && is_attr_char(b[i]) {
        i += 1;
    }
    i
}

/// `&amp;`, `&#10;` — returns `i + 1` when this is a bare `&`.
fn entity(b: &[u8], i: usize) -> usize {
    let mut j = i + 1;
    if b.get(j) == Some(&b'#') {
        j += 1;
    }
    let start = j;
    while j < b.len() && j - start < 10 && is_ident(b[j]) {
        j += 1;
    }
    if j > start && b.get(j) == Some(&b';') {
        j + 1
    } else {
        i + 1
    }
}

fn find(b: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    (from..b.len().saturating_sub(needle.len() - 1)).find(|&j| starts_with(b, j, needle))
}

fn find_ci(b: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    (from..b.len().saturating_sub(needle.len() - 1)).find(|&j| starts_with_ci(b, j, needle))
}
