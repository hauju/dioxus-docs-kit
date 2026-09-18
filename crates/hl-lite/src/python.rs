//! Python lexer.

use crate::Kind;
use crate::emit::Emit;
use crate::util::{char_len, ident_end, is_ident_start, line_end, scan_number, scan_quoted};

const KEYWORDS: &[&str] = &[
    "and", "as", "assert", "async", "await", "break", "case", "class", "continue", "def", "del",
    "elif", "else", "except", "finally", "for", "from", "global", "if", "import", "in", "is",
    "lambda", "match", "nonlocal", "not", "or", "pass", "raise", "return", "try", "while", "with",
    "yield",
];

const CONSTANTS: &[&str] = &[
    "Ellipsis",
    "False",
    "None",
    "NotImplemented",
    "True",
    "__name__",
    "cls",
    "self",
];

const BUILTINS: &[&str] = &[
    "abs",
    "all",
    "any",
    "bool",
    "bytes",
    "dict",
    "enumerate",
    "filter",
    "float",
    "format",
    "getattr",
    "hasattr",
    "input",
    "int",
    "isinstance",
    "len",
    "list",
    "map",
    "max",
    "min",
    "next",
    "open",
    "print",
    "range",
    "repr",
    "reversed",
    "round",
    "set",
    "setattr",
    "sorted",
    "str",
    "sum",
    "super",
    "tuple",
    "type",
    "zip",
];

pub(crate) fn lex(out: &mut Emit<'_>) {
    let b = out.bytes();
    let src = out.src();
    let n = b.len();
    let mut i = 0;
    // `def`/`class` name the identifier that follows them.
    let mut pending: Option<Kind> = None;

    while i < n {
        let c = b[i];
        match c {
            b'#' => {
                let e = line_end(b, i);
                out.push(i, e, Kind::Comment);
                i = e;
            }
            b'"' | b'\'' => {
                let e = string(b, i);
                out.push(i, e, Kind::String);
                i = e;
            }
            b'@' if at_line_start(b, i) && b.get(i + 1).is_some_and(|&c| is_ident_start(c)) => {
                let mut e = ident_end(b, i + 1);
                while b.get(e) == Some(&b'.') && b.get(e + 1).is_some_and(|&c| is_ident_start(c)) {
                    e = ident_end(b, e + 1);
                }
                out.push(i, e, Kind::Attribute);
                i = e;
            }
            b'0'..=b'9' => {
                let e = scan_number(b, i);
                out.push(i, e, Kind::Number);
                i = e;
            }
            _ if is_ident_start(c) => {
                // A string prefix (`f"…"`, `rb'…'`) rather than a name?
                if let Some(q) = string_prefix(b, i) {
                    let e = string(b, q);
                    out.push(i, e, Kind::String);
                    i = e;
                    continue;
                }
                let e = ident_end(b, i);
                let word = &src[i..e];
                let kind = if let Some(k) = pending.take() {
                    k
                } else if KEYWORDS.contains(&word) {
                    if word == "def" {
                        pending = Some(Kind::Function);
                    } else if word == "class" {
                        pending = Some(Kind::Type);
                    }
                    Kind::Keyword
                } else if CONSTANTS.contains(&word) {
                    Kind::Constant
                } else if BUILTINS.contains(&word) || b.get(e) == Some(&b'(') {
                    Kind::Function
                } else {
                    Kind::Plain
                };
                out.push(i, e, kind);
                i = e;
            }
            b'{' | b'}' | b'(' | b')' | b'[' | b']' | b',' | b':' | b';' => {
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

fn at_line_start(b: &[u8], i: usize) -> bool {
    let mut j = i;
    while j > 0 && (b[j - 1] == b' ' || b[j - 1] == b'\t') {
        j -= 1;
    }
    j == 0 || b[j - 1] == b'\n'
}

/// Index of the quote opening a prefixed string (`r`, `b`, `u`, `f`, `rb`, …),
/// or `None` when this is an ordinary identifier.
fn string_prefix(b: &[u8], i: usize) -> Option<usize> {
    let mut j = i;
    while j - i < 2
        && b.get(j)
            .is_some_and(|&c| matches!(c | 32, b'r' | b'b' | b'u' | b'f'))
    {
        j += 1;
    }
    (j > i && matches!(b.get(j), Some(b'"' | b'\''))).then_some(j)
}

/// A string literal starting at the quote, triple-quoted or not.
fn string(b: &[u8], i: usize) -> usize {
    let q = b[i];
    if b.get(i + 1) == Some(&q) && b.get(i + 2) == Some(&q) {
        let mut j = i + 3;
        while j < b.len() {
            if b[j] == b'\\' {
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
    scan_quoted(b, i, q, true, false)
}
