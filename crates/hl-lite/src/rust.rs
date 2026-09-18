//! Rust lexer.

use crate::Kind;
use crate::emit::Emit;
use crate::util::{char_len, ident_end, is_ident_start, line_end, scan_number, scan_quoted};

const KEYWORDS: &[&str] = &[
    "as",
    "async",
    "await",
    "break",
    "const",
    "continue",
    "crate",
    "dyn",
    "else",
    "enum",
    "extern",
    "fn",
    "for",
    "gen",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "macro_rules",
    "match",
    "mod",
    "move",
    "mut",
    "pub",
    "ref",
    "return",
    "self",
    "Self",
    "static",
    "struct",
    "super",
    "trait",
    "type",
    "union",
    "unsafe",
    "use",
    "where",
    "while",
    "yield",
];

const PRIMITIVES: &[&str] = &[
    "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize", "str", "u8", "u16",
    "u32", "u64", "u128", "usize",
];

pub(crate) fn lex(out: &mut Emit<'_>) {
    let b = out.bytes();
    let src = out.src();
    let n = b.len();
    let mut i = 0;

    while i < n {
        let c = b[i];
        match c {
            b'/' if b.get(i + 1) == Some(&b'/') => {
                let e = line_end(b, i);
                out.push(i, e, Kind::Comment);
                i = e;
            }
            b'/' if b.get(i + 1) == Some(&b'*') => {
                let e = block_comment(b, i);
                out.push(i, e, Kind::Comment);
                i = e;
            }
            b'#' if b.get(i + 1) == Some(&b'[')
                || (b.get(i + 1) == Some(&b'!') && b.get(i + 2) == Some(&b'[')) =>
            {
                let e = attribute(b, i);
                out.push(i, e, Kind::Attribute);
                i = e;
            }
            b'"' => {
                let e = scan_quoted(b, i, b'"', true, true);
                out.push(i, e, Kind::String);
                i = e;
            }
            b'\'' => {
                let (e, kind) = quote(b, i);
                out.push(i, e, kind);
                i = e;
            }
            b'0'..=b'9' => {
                let e = scan_number(b, i);
                out.push(i, e, Kind::Number);
                i = e;
            }
            b'b' | b'c' | b'r' => {
                if let Some(e) = prefixed_literal(b, i) {
                    out.push(i, e, Kind::String);
                    i = e;
                } else {
                    i = identifier(out, b, src, i);
                }
            }
            _ if is_ident_start(c) => {
                i = identifier(out, b, src, i);
            }
            b'{' | b'}' | b'(' | b')' | b'[' | b']' | b',' | b';' => {
                out.push(i, i + 1, Kind::Punctuation);
                i += 1;
            }
            _ if c.is_ascii_punctuation() => {
                out.push(i, i + 1, Kind::Operator);
                i += 1;
            }
            _ => i += char_len(c),
        }
    }
}

/// Classify the identifier at `i` and return the index just past it.
fn identifier(out: &mut Emit<'_>, b: &[u8], src: &str, i: usize) -> usize {
    let e = ident_end(b, i);
    let word = &src[i..e];

    if KEYWORDS.contains(&word) {
        out.push(i, e, Kind::Keyword);
        return e;
    }
    if word == "true" || word == "false" {
        out.push(i, e, Kind::Constant);
        return e;
    }
    // `name!` / `name!(` — a macro invocation, but not `name != x`.
    if b.get(e) == Some(&b'!') && b.get(e + 1) != Some(&b'=') {
        out.push(i, e + 1, Kind::Function);
        return e + 1;
    }
    let turbofish =
        b.get(e) == Some(&b':') && b.get(e + 1) == Some(&b':') && b.get(e + 2) == Some(&b'<');
    if b.get(e) == Some(&b'(') || turbofish {
        out.push(i, e, Kind::Function);
        return e;
    }
    if PRIMITIVES.contains(&word) || b[i].is_ascii_uppercase() {
        out.push(i, e, Kind::Type);
        return e;
    }
    e
}

/// `/* … */`, nesting as the language does.
fn block_comment(b: &[u8], i: usize) -> usize {
    let mut depth = 0usize;
    let mut j = i;
    while j < b.len() {
        if b[j] == b'/' && b.get(j + 1) == Some(&b'*') {
            depth += 1;
            j += 2;
        } else if b[j] == b'*' && b.get(j + 1) == Some(&b'/') {
            depth -= 1;
            j += 2;
            if depth == 0 {
                return j;
            }
        } else {
            j += 1;
        }
    }
    b.len()
}

/// `#[…]` / `#![…]`, balancing brackets and stepping over strings.
fn attribute(b: &[u8], i: usize) -> usize {
    let mut j = i;
    while j < b.len() && b[j] != b'[' {
        j += 1;
    }
    let mut depth = 0usize;
    while j < b.len() {
        match b[j] {
            b'[' => {
                depth += 1;
                j += 1;
            }
            b']' => {
                depth -= 1;
                j += 1;
                if depth == 0 {
                    return j;
                }
            }
            b'"' => j = scan_quoted(b, j, b'"', true, true),
            _ => j += 1,
        }
    }
    b.len()
}

/// A `'` opens either a char literal or a lifetime.
fn quote(b: &[u8], i: usize) -> (usize, Kind) {
    match b.get(i + 1) {
        Some(b'\\') => (scan_quoted(b, i, b'\'', true, false), Kind::String),
        Some(&c) if is_ident_start(c) => {
            let e = ident_end(b, i + 1);
            if b.get(e) == Some(&b'\'') {
                (e + 1, Kind::String) // 'a'
            } else {
                (e, Kind::Constant) // 'a — a lifetime
            }
        }
        Some(&c) => {
            let e = i + 1 + char_len(c);
            if b.get(e) == Some(&b'\'') {
                (e + 1, Kind::String)
            } else {
                (i + 1, Kind::Punctuation)
            }
        }
        None => (i + 1, Kind::Punctuation),
    }
}

/// `r"…"`, `r#"…"#`, `b"…"`, `b'x'`, `br#"…"#`, `c"…"` — or `None` when the
/// `b`/`c`/`r` just starts an ordinary identifier.
fn prefixed_literal(b: &[u8], i: usize) -> Option<usize> {
    let mut j = i;
    if b[j] == b'b' || b[j] == b'c' {
        j += 1;
    }
    let raw = b.get(j) == Some(&b'r');
    if raw {
        j += 1;
    }

    if !raw {
        if j == i {
            return None; // bare `r…` identifier
        }
        return match b.get(j) {
            Some(b'"') => Some(scan_quoted(b, j, b'"', true, true)),
            Some(b'\'') => Some(scan_quoted(b, j, b'\'', true, false)),
            _ => None,
        };
    }

    let mut hashes = 0usize;
    while b.get(j) == Some(&b'#') {
        hashes += 1;
        j += 1;
    }
    if b.get(j) != Some(&b'"') {
        return None;
    }
    j += 1;
    while j < b.len() {
        if b[j] == b'"' {
            let close = j + 1;
            let mut seen = 0usize;
            while seen < hashes && b.get(close + seen) == Some(&b'#') {
                seen += 1;
            }
            if seen == hashes {
                return Some(close + hashes);
            }
        }
        j += 1;
    }
    Some(b.len())
}
