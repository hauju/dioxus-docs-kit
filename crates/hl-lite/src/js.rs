//! JavaScript and TypeScript lexer.

use crate::Kind;
use crate::emit::Emit;
use crate::util::{
    char_len, ident_end, is_ident, is_ident_start, line_end, scan_number, scan_quoted,
};

const KEYWORDS: &[&str] = &[
    "as",
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "export",
    "extends",
    "finally",
    "for",
    "from",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "let",
    "new",
    "of",
    "return",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

const TS_KEYWORDS: &[&str] = &[
    "abstract",
    "asserts",
    "declare",
    "enum",
    "implements",
    "infer",
    "interface",
    "is",
    "keyof",
    "module",
    "namespace",
    "override",
    "private",
    "protected",
    "public",
    "readonly",
    "satisfies",
    "type",
];

const TS_TYPES: &[&str] = &[
    "any", "bigint", "boolean", "never", "number", "object", "string", "symbol", "unknown", "void",
];

const CONSTANTS: &[&str] = &["Infinity", "NaN", "false", "null", "true", "undefined"];

pub(crate) fn lex(out: &mut Emit<'_>, ts: bool) {
    let b = out.bytes();
    let src = out.src();
    let n = b.len();
    let mut i = 0;
    // What came before, so a `/` can be told apart from a regex literal.
    let mut prev = Prev::Start;
    // A regex scan already failed on the line ending here; don't retry, or a
    // line of stray slashes would cost O(n²).
    let mut no_regex_until = 0usize;

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
            b'/' if prev.allows_regex() && i >= no_regex_until => match regex(b, i) {
                Some(e) => {
                    out.push(i, e, Kind::String);
                    i = e;
                    prev = Prev::Value;
                }
                None => {
                    no_regex_until = line_end(b, i);
                    out.push(i, i + 1, Kind::Operator);
                    i += 1;
                    prev = Prev::Operator;
                }
            },
            b'"' | b'\'' => {
                let e = scan_quoted(b, i, c, true, false);
                out.push(i, e, Kind::String);
                i = e;
                prev = Prev::Value;
            }
            b'`' => {
                i = template(out, b, i);
                prev = Prev::Value;
            }
            b'@' if b.get(i + 1).is_some_and(|&c| is_ident_start(c)) => {
                let e = ident_end(b, i + 1);
                out.push(i, e, Kind::Attribute);
                i = e;
                prev = Prev::Value;
            }
            b'0'..=b'9' => {
                let e = scan_number(b, i);
                out.push(i, e, Kind::Number);
                i = e;
                prev = Prev::Value;
            }
            b'.' if b.get(i + 1).is_some_and(u8::is_ascii_digit) && !prev.is_value() => {
                let e = scan_number(b, i);
                out.push(i, e, Kind::Number);
                i = e;
                prev = Prev::Value;
            }
            b'#' | b'$' | b'_' => {
                // `#private` fields, `$` identifiers.
                let e = ident_end(b, i + 1).max(i + 1);
                i = ident(out, b, src, i, e, ts);
                prev = Prev::Value;
            }
            _ if is_ident_start(c) => {
                let e = ident_end(b, i);
                let word = &src[i..e];
                prev = if is_keyword(word, ts) {
                    Prev::Keyword(keyword_allows_regex(word))
                } else {
                    Prev::Value
                };
                i = ident(out, b, src, i, e, ts);
            }
            b'{' | b'}' | b'(' | b')' | b'[' | b']' | b',' | b';' | b':' => {
                out.push(i, i + 1, Kind::Punctuation);
                i += 1;
                prev = if matches!(c, b')' | b']') {
                    Prev::Value
                } else {
                    Prev::Operator
                };
            }
            _ if c.is_ascii_punctuation() => {
                out.push(i, i + 1, Kind::Operator);
                i += 1;
                prev = Prev::Operator;
            }
            _ => i += char_len(c),
        }
    }
}

/// The previous significant token, for regex disambiguation.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Prev {
    Start,
    /// An operator, `(`, `,`, `{`, `;`, `:` … — a regex may follow.
    Operator,
    /// An identifier, literal, `)` or `]` — a `/` here is division.
    Value,
    /// A keyword; the flag says whether a regex may follow it.
    Keyword(bool),
}

impl Prev {
    fn allows_regex(self) -> bool {
        match self {
            Prev::Start | Prev::Operator => true,
            Prev::Value => false,
            Prev::Keyword(ok) => ok,
        }
    }
    fn is_value(self) -> bool {
        self == Prev::Value
    }
}

fn keyword_allows_regex(word: &str) -> bool {
    !matches!(word, "this" | "super")
}

fn is_keyword(word: &str, ts: bool) -> bool {
    KEYWORDS.contains(&word) || (ts && TS_KEYWORDS.contains(&word))
}

fn ident(out: &mut Emit<'_>, b: &[u8], src: &str, i: usize, e: usize, ts: bool) -> usize {
    let word = &src[i..e];
    let kind = if is_keyword(word, ts) {
        Kind::Keyword
    } else if CONSTANTS.contains(&word) {
        Kind::Constant
    } else if ts && TS_TYPES.contains(&word) {
        Kind::Type
    } else if b.get(e) == Some(&b'(') {
        Kind::Function
    } else if b[i].is_ascii_uppercase() {
        Kind::Type
    } else {
        Kind::Plain
    };
    out.push(i, e, kind);
    e
}

fn block_comment(b: &[u8], i: usize) -> usize {
    let mut j = i + 2;
    while j < b.len() {
        if b[j] == b'*' && b.get(j + 1) == Some(&b'/') {
            return j + 2;
        }
        j += 1;
    }
    b.len()
}

/// A template literal. The `${…}` holes are left plain; their contents are not
/// lexed recursively.
fn template(out: &mut Emit<'_>, b: &[u8], i: usize) -> usize {
    let n = b.len();
    let mut j = i + 1;
    let mut seg = i;
    while j < n {
        match b[j] {
            b'\\' => j += 2,
            b'`' => {
                j += 1;
                break;
            }
            b'$' if b.get(j + 1) == Some(&b'{') => {
                out.push(seg, j + 2, Kind::String);
                let hole = hole_end(b, j + 2);
                out.push(j + 2, hole, Kind::Plain);
                j = hole;
                seg = j;
            }
            _ => j += 1,
        }
    }
    let j = j.min(n);
    out.push(seg, j, Kind::String);
    j
}

/// End of a `${…}` hole: the matching `}`, skipping nested braces and strings.
fn hole_end(b: &[u8], i: usize) -> usize {
    let mut depth = 1usize;
    let mut j = i;
    while j < b.len() {
        match b[j] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return j;
                }
            }
            c @ (b'"' | b'\'' | b'`') => {
                j = scan_quoted(b, j, c, true, c == b'`');
                continue;
            }
            _ => {}
        }
        j += 1;
    }
    b.len()
}

/// A regex literal, or `None` when it does not close on this line.
fn regex(b: &[u8], i: usize) -> Option<usize> {
    let mut j = i + 1;
    let mut in_class = false;
    while j < b.len() {
        match b[j] {
            b'\\' => j += 2,
            b'\n' => return None,
            b'[' => {
                in_class = true;
                j += 1;
            }
            b']' => {
                in_class = false;
                j += 1;
            }
            b'/' if !in_class => {
                j += 1;
                while j < b.len() && is_ident(b[j]) {
                    j += 1;
                }
                return Some(j);
            }
            _ => j += 1,
        }
    }
    None
}
