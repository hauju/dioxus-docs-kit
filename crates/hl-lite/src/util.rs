//! Byte-level scanning helpers shared by the lexers.
//!
//! Every helper works on `&[u8]` and only ever *decides* on ASCII bytes, so an
//! index it returns is always a `char` boundary: the continuation bytes of a
//! multi-byte character are all `>= 0x80` and can never match an ASCII
//! delimiter.

#[inline]
pub(crate) fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

#[inline]
pub(crate) fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Number of bytes in the UTF-8 sequence that starts with `b`.
///
/// A stray continuation byte counts as 1 so that scanning always advances.
#[inline]
pub(crate) fn char_len(b: u8) -> usize {
    match b {
        0x00..=0xBF => 1,
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        _ => 4,
    }
}

/// Index of the `\n` ending the line containing `i`, or the input length.
pub(crate) fn line_end(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && b[i] != b'\n' {
        i += 1;
    }
    i
}

/// End of the identifier starting at `i`.
pub(crate) fn ident_end(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && is_ident(b[i]) {
        i += 1;
    }
    i
}

/// Skip spaces and tabs.
pub(crate) fn skip_blanks(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && (b[i] == b' ' || b[i] == b'\t') {
        i += 1;
    }
    i
}

/// Skip all whitespace, newlines included.
pub(crate) fn skip_ws(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

/// Scan a quoted literal that starts at `i` (the opening quote).
///
/// Returns the index just past the closing quote, or — when the literal is
/// unterminated — the end of the line (`multiline == false`) or of the input.
pub(crate) fn scan_quoted(b: &[u8], i: usize, quote: u8, escape: bool, multiline: bool) -> usize {
    let mut j = i + 1;
    while j < b.len() {
        let c = b[j];
        if escape && c == b'\\' {
            j += 2;
            continue;
        }
        if c == quote {
            return j + 1;
        }
        if !multiline && c == b'\n' {
            return j;
        }
        j += 1;
    }
    b.len()
}

/// Scan a numeric literal starting at `i`.
///
/// Handles `0x`/`0b`/`0o` prefixes, `_` separators, a fractional part (only
/// when a digit follows the `.`, so Rust's `1..n` and `1.max(2)` survive), an
/// exponent, and a trailing suffix — which doubles as Rust's `u32`, JS's `n`,
/// Python's `j` and CSS's `px`.
pub(crate) fn scan_number(b: &[u8], i: usize) -> usize {
    if i >= b.len() {
        return i;
    }
    let mut j = i;
    if b[j] == b'.' {
        j += 1;
    } else if b[j] == b'0' && matches!(b.get(j + 1).map(|c| c | 32), Some(b'x' | b'b' | b'o')) {
        j += 2;
        while j < b.len() && (is_ident(b[j])) {
            j += 1;
        }
        return j;
    }
    while j < b.len() && (b[j].is_ascii_digit() || b[j] == b'_') {
        j += 1;
    }
    if b.get(j) == Some(&b'.') && b.get(j + 1).is_some_and(u8::is_ascii_digit) {
        j += 1;
        while j < b.len() && (b[j].is_ascii_digit() || b[j] == b'_') {
            j += 1;
        }
    }
    let exp_digits = b.get(j + 1).is_some_and(u8::is_ascii_digit);
    let signed_exp =
        matches!(b.get(j + 1), Some(b'+' | b'-')) && b.get(j + 2).is_some_and(u8::is_ascii_digit);
    if b.get(j).is_some_and(|c| c | 32 == b'e') && (exp_digits || signed_exp) {
        j += if signed_exp { 2 } else { 1 };
        while j < b.len() && (b[j].is_ascii_digit() || b[j] == b'_') {
            j += 1;
        }
    }
    while j < b.len() && is_ident(b[j]) {
        j += 1;
    }
    j
}

/// True when `b[i..]` starts with `needle`.
pub(crate) fn starts_with(b: &[u8], i: usize, needle: &[u8]) -> bool {
    b.len() >= i + needle.len() && &b[i..i + needle.len()] == needle
}

/// Case-insensitive [`starts_with`] (ASCII only).
pub(crate) fn starts_with_ci(b: &[u8], i: usize, needle: &[u8]) -> bool {
    b.len() >= i + needle.len() && b[i..i + needle.len()].eq_ignore_ascii_case(needle)
}
