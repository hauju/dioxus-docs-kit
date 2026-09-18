//! TOML lexer.

use crate::Kind;
use crate::emit::Emit;
use crate::util::{
    char_len, is_ident, is_ident_start, line_end, scan_number, scan_quoted, skip_blanks,
};

pub(crate) fn lex(out: &mut Emit<'_>) {
    let b = out.bytes();
    let src = out.src();
    let n = b.len();
    let mut i = 0;
    // Before the `=`, a bare word is a key; after it, a value.
    let mut in_value = false;
    // Open `[` / `{` in a value, which let it span lines.
    let mut depth = 0usize;

    while i < n {
        let c = b[i];
        match c {
            b'\n' => {
                if depth == 0 {
                    in_value = false;
                }
                i += 1;
            }
            b' ' | b'\t' | b'\r' => i += 1,
            b'#' => {
                let e = line_end(b, i);
                out.push(i, e, Kind::Comment);
                i = e;
            }
            b'[' if !in_value => {
                let e = table_end(b, i);
                out.push(i, e, Kind::Tag);
                i = e;
            }
            b'"' | b'\'' => {
                let e = string(b, i);
                out.push(
                    i,
                    e,
                    if in_value {
                        Kind::String
                    } else {
                        Kind::Property
                    },
                );
                i = e;
            }
            b'=' => {
                out.push(i, i + 1, Kind::Operator);
                // A `=` inside an inline table does not end the outer value.
                in_value |= depth == 0;
                i += 1;
            }
            b'[' | b'{' => {
                out.push(i, i + 1, Kind::Punctuation);
                depth += 1;
                i += 1;
            }
            b']' | b'}' => {
                out.push(i, i + 1, Kind::Punctuation);
                depth = depth.saturating_sub(1);
                i += 1;
            }
            b',' | b'.' => {
                out.push(i, i + 1, Kind::Punctuation);
                i += 1;
            }
            _ if in_value => match c {
                b'0'..=b'9' => {
                    let e = date(b, i).unwrap_or_else(|| scan_number(b, i));
                    out.push(i, e, Kind::Number);
                    i = e.max(i + 1);
                }
                b'-' | b'+' if b.get(i + 1).is_some_and(u8::is_ascii_digit) => {
                    let e = scan_number(b, i + 1);
                    out.push(i, e, Kind::Number);
                    i = e;
                }
                _ if is_ident_start(c) => {
                    let e = bare_key_end(b, i);
                    let word = &src[i..e];
                    // Inline tables carry keys of their own: `{ path = "…" }`.
                    if b.get(skip_blanks(b, e)) == Some(&b'=') {
                        out.push(i, e, Kind::Property);
                    } else if matches!(word, "true" | "false" | "inf" | "nan") {
                        out.push(i, e, Kind::Constant);
                    }
                    i = e;
                }
                _ => i += char_len(c),
            },
            // Key position.
            _ if is_ident(c) || c == b'-' => {
                let e = bare_key_end(b, i);
                out.push(i, e, Kind::Property);
                i = e;
            }
            _ => i += char_len(c),
        }
    }
}

/// A bare key: letters, digits, `_` and `-`.
fn bare_key_end(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && (is_ident(b[i]) || b[i] == b'-') {
        i += 1;
    }
    i
}

/// `[table]` / `[[array of tables]]`, stepping over quoted key segments.
fn table_end(b: &[u8], i: usize) -> usize {
    let double = b.get(i + 1) == Some(&b'[');
    let mut j = i + if double { 2 } else { 1 };
    while j < b.len() {
        match b[j] {
            c @ (b'"' | b'\'') => j = scan_quoted(b, j, c, c == b'"', false),
            b']' => {
                return if double && b.get(j + 1) == Some(&b']') {
                    j + 2
                } else {
                    j + 1
                };
            }
            b'\n' => return j,
            _ => j += 1,
        }
    }
    b.len()
}

/// Basic, literal and multi-line strings.
fn string(b: &[u8], i: usize) -> usize {
    let q = b[i];
    if b.get(i + 1) == Some(&q) && b.get(i + 2) == Some(&q) {
        let mut j = i + 3;
        while j < b.len() {
            if q == b'"' && b[j] == b'\\' {
                j += 2;
                continue;
            }
            if b[j] == q && b.get(j + 1) == Some(&q) && b.get(j + 2) == Some(&q) {
                return j + 3;
            }
            j += 1;
        }
        return b.len();
    }
    scan_quoted(b, i, q, q == b'"', false)
}

/// An offset date-time, local date or local time.
fn date(b: &[u8], i: usize) -> Option<usize> {
    if digits(b, i, 4)
        && b.get(i + 4) == Some(&b'-')
        && digits(b, i + 5, 2)
        && b.get(i + 7) == Some(&b'-')
        && digits(b, i + 8, 2)
    {
        let mut j = i + 10;
        let sep = b.get(j).copied();
        if matches!(sep, Some(b'T' | b't')) || (sep == Some(b' ') && digits(b, j + 1, 2)) {
            j = time(b, j + 1);
        }
        return Some(j);
    }
    if digits(b, i, 2) && b.get(i + 2) == Some(&b':') && digits(b, i + 3, 2) {
        return Some(time(b, i));
    }
    None
}

fn time(b: &[u8], i: usize) -> usize {
    if !digits(b, i, 2) || b.get(i + 2) != Some(&b':') || !digits(b, i + 3, 2) {
        return i;
    }
    let mut j = i + 5;
    if b.get(j) == Some(&b':') && digits(b, j + 1, 2) {
        j += 3;
    }
    if b.get(j) == Some(&b'.') && digits(b, j + 1, 1) {
        j += 1;
        while b.get(j).is_some_and(u8::is_ascii_digit) {
            j += 1;
        }
    }
    match b.get(j) {
        Some(b'Z' | b'z') => j + 1,
        Some(b'+' | b'-')
            if digits(b, j + 1, 2) && b.get(j + 3) == Some(&b':') && digits(b, j + 4, 2) =>
        {
            j + 6
        }
        _ => j,
    }
}

fn digits(b: &[u8], i: usize, n: usize) -> bool {
    b.len() >= i + n && b[i..i + n].iter().all(u8::is_ascii_digit)
}
