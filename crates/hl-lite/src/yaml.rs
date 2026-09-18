//! YAML lexer.
//!
//! Line-oriented, like the language itself: indentation decides structure, so
//! each line is classified (comment, document marker, list item, `key:`) and
//! whatever follows the colon goes through a small value scanner.

use crate::Kind;
use crate::emit::Emit;
use crate::util::{char_len, ident_end, is_ident, line_end, scan_number, scan_quoted, skip_blanks};

const CONSTANTS: &[&str] = &[
    "false", "False", "FALSE", "n", "no", "No", "NO", "null", "Null", "NULL", "off", "Off", "OFF",
    "on", "On", "ON", "true", "True", "TRUE", "y", "yes", "Yes", "YES", "~",
];

pub(crate) fn lex(out: &mut Emit<'_>) {
    let b = out.bytes();
    let src = out.src();
    let n = b.len();
    let mut i = 0;

    while i < n {
        let le = line_end(b, i);
        let next = if le < n { le + 1 } else { le };
        let cs = skip_blanks(b, i);
        let indent = cs - i;

        if cs >= le {
            i = next;
            continue;
        }
        if b[cs] == b'#' {
            out.push(cs, le, Kind::Comment);
            i = next;
            continue;
        }
        // Document markers.
        if (b[cs] == b'-' || b[cs] == b'.')
            && cs + 3 <= le
            && b[cs..cs + 3].iter().all(|&c| c == b[cs])
        {
            out.push(cs, cs + 3, Kind::Punctuation);
            let (end, block) = value(out, b, src, cs + 3, le, indent);
            i = if block > end { block } else { next };
            continue;
        }

        // Nested list markers: `- - item`.
        let mut j = cs;
        while b.get(j) == Some(&b'-') && matches!(b.get(j + 1), Some(b' ' | b'\t') | None) {
            out.push(j, j + 1, Kind::Punctuation);
            j = skip_blanks(b, j + 1);
        }
        if j >= le {
            i = next;
            continue;
        }

        // `key:` — plain or quoted.
        if let Some((key_end, colon)) = key(b, j, le) {
            out.push(j, key_end, Kind::Property);
            out.push(colon, colon + 1, Kind::Punctuation);
            j = colon + 1;
        }

        let (_, block) = value(out, b, src, j, le, indent);
        i = if block > le { block } else { next };
    }
}

/// A `key:` at `i` — returns `(end of key, index of the colon)`.
fn key(b: &[u8], i: usize, le: usize) -> Option<(usize, usize)> {
    let mut j = i;
    if matches!(b.get(j), Some(b'"' | b'\'')) {
        let e = scan_quoted(b, j, b[j], b[j] == b'"', false);
        let c = skip_blanks(b, e);
        return (b.get(c) == Some(&b':')).then_some((e, c));
    }
    // A plain scalar key ends at the first `: ` (or a `:` at end of line).
    while j < le {
        match b[j] {
            b':' if j + 1 >= le || matches!(b[j + 1], b' ' | b'\t') => {
                return (j > i).then_some((j, j));
            }
            b'#' if j > i && matches!(b[j - 1], b' ' | b'\t') => return None,
            b'{' | b'[' | b',' => return None,
            _ => j += char_len(b[j]),
        }
    }
    None
}

/// Scan the value part of a line.
///
/// Returns `(end of the scanned range, end of a block scalar)`; the second is
/// only greater than the first when a `|` or `>` swallowed following lines.
fn value(
    out: &mut Emit<'_>,
    b: &[u8],
    src: &str,
    from: usize,
    le: usize,
    indent: usize,
) -> (usize, usize) {
    let mut j = skip_blanks(b, from);
    while j < le {
        match b[j] {
            b'#' if j == from || matches!(b[j - 1], b' ' | b'\t') => {
                out.push(j, le, Kind::Comment);
                return (le, 0);
            }
            c @ (b'"' | b'\'') => {
                let e = scan_quoted(b, j, c, c == b'"', false);
                let kind = if b.get(skip_blanks(b, e)) == Some(&b':') {
                    Kind::Property
                } else {
                    Kind::String
                };
                out.push(j, e, kind);
                j = e;
            }
            b'&' | b'*' => {
                let e = ident_end(b, j + 1).max(j + 1);
                out.push(j, e, Kind::Variable);
                j = e;
            }
            b'!' => {
                let mut e = j + 1;
                while e < le && (is_ident(b[e]) || matches!(b[e], b'!' | b'-' | b'/' | b':')) {
                    e += 1;
                }
                out.push(j, e, Kind::Type);
                j = e;
            }
            b'|' | b'>' => {
                let mut e = j + 1;
                while matches!(b.get(e), Some(b'-' | b'+'))
                    || b.get(e).is_some_and(u8::is_ascii_digit)
                {
                    e += 1;
                }
                out.push(j, e, Kind::Operator);
                let rest = skip_blanks(b, e);
                if rest >= le || b[rest] == b'#' {
                    if rest < le {
                        out.push(rest, le, Kind::Comment);
                    }
                    let body = block_scalar(b, le, indent);
                    out.push(le, body, Kind::String);
                    return (le, body);
                }
                j = e;
            }
            b'{' | b'}' | b'[' | b']' | b',' | b':' => {
                out.push(j, j + 1, Kind::Punctuation);
                j += 1;
            }
            b' ' | b'\t' => j += 1,
            _ => {
                let e = scalar_end(b, j, le);
                if e == j {
                    j += char_len(b[j]);
                    continue;
                }
                let word = &src[j..e];
                let kind = if b.get(e) == Some(&b':') {
                    Kind::Property
                } else if CONSTANTS.contains(&word) {
                    Kind::Constant
                } else if is_number(b, j, e) {
                    Kind::Number
                } else {
                    Kind::Plain
                };
                out.push(j, e, kind);
                j = e;
            }
        }
    }
    (le, 0)
}

/// End of a plain scalar: it stops at a flow indicator, a `: ` or a comment.
fn scalar_end(b: &[u8], i: usize, le: usize) -> usize {
    let mut j = i;
    while j < le {
        match b[j] {
            b',' | b'{' | b'}' | b'[' | b']' => break,
            b':' if j + 1 >= le || matches!(b[j + 1], b' ' | b'\t' | b',' | b'}' | b']') => break,
            b'#' if j > i && matches!(b[j - 1], b' ' | b'\t') => break,
            _ => j += char_len(b[j]),
        }
    }
    // Trailing blanks belong to nobody.
    while j > i && matches!(b[j - 1], b' ' | b'\t') {
        j -= 1;
    }
    j
}

fn is_number(b: &[u8], i: usize, e: usize) -> bool {
    let start = if matches!(b[i], b'-' | b'+') {
        i + 1
    } else {
        i
    };
    start < e && b[start].is_ascii_digit() && scan_number(b, start) >= e
}

/// Lines belonging to a `|`/`>` block scalar: everything indented deeper than
/// the line that introduced it.
fn block_scalar(b: &[u8], le: usize, indent: usize) -> usize {
    let n = b.len();
    let mut i = if le < n { le + 1 } else { n };
    let mut end = le;
    while i < n {
        let line_end_ = line_end(b, i);
        let cs = skip_blanks(b, i);
        let blank = cs >= line_end_;
        if !blank && cs - i <= indent {
            break;
        }
        end = line_end_;
        i = if line_end_ < n { line_end_ + 1 } else { n };
    }
    end
}
