//! CSS lexer.

use crate::Kind;
use crate::emit::Emit;
use crate::util::{char_len, ident_end, is_ident, is_ident_start, scan_number, scan_quoted};

#[derive(Clone, Copy, PartialEq, Eq)]
enum St {
    /// Between rules: selectors.
    Sel,
    /// After `@name`, up to the `{` or `;`.
    Prelude,
    /// Inside a declaration block, expecting a property.
    Prop,
    /// After the `:` of a declaration.
    Value,
}

/// At-rules whose block contains nested rules rather than declarations.
const NESTED_AT: &[&str] = &[
    "media",
    "supports",
    "container",
    "layer",
    "scope",
    "keyframes",
    "document",
    "-webkit-keyframes",
];

pub(crate) fn lex(out: &mut Emit<'_>) {
    let b = out.bytes();
    let src = out.src();
    let n = b.len();
    let mut i = 0;
    let mut state = St::Sel;
    let mut nested_at = false;
    let mut in_attr_sel = false;
    let mut stack: Vec<St> = Vec::new();

    while i < n {
        let c = b[i];
        match c {
            b'/' if b.get(i + 1) == Some(&b'*') => {
                let e = block_comment(b, i);
                out.push(i, e, Kind::Comment);
                i = e;
            }
            b'"' | b'\'' => {
                let e = scan_quoted(b, i, c, true, false);
                out.push(i, e, Kind::String);
                i = e;
            }
            b'@' if b
                .get(i + 1)
                .is_some_and(|&c| is_ident_start(c) || c == b'-') =>
            {
                let e = dashed_ident_end(b, i + 1);
                out.push(i, e, Kind::Keyword);
                let name = &src[i + 1..e];
                nested_at = NESTED_AT.iter().any(|at| at.eq_ignore_ascii_case(name));
                state = St::Prelude;
                i = e;
            }
            b'{' => {
                out.push(i, i + 1, Kind::Punctuation);
                stack.push(match state {
                    St::Prop | St::Value => St::Prop,
                    _ => St::Sel,
                });
                state = if state == St::Prelude && nested_at {
                    St::Sel
                } else {
                    St::Prop
                };
                nested_at = false;
                i += 1;
            }
            b'}' => {
                out.push(i, i + 1, Kind::Punctuation);
                state = stack.pop().unwrap_or(St::Sel);
                i += 1;
            }
            b';' => {
                out.push(i, i + 1, Kind::Punctuation);
                state = match state {
                    St::Prop | St::Value => St::Prop,
                    _ => St::Sel,
                };
                i += 1;
            }
            b':' if state == St::Prop => {
                out.push(i, i + 1, Kind::Punctuation);
                state = St::Value;
                i += 1;
            }
            b'!' if state == St::Value && b.get(i + 1).is_some_and(|&c| is_ident_start(c)) => {
                let e = ident_end(b, i + 1);
                out.push(i, e, Kind::Keyword); // !important
                i = e;
            }
            b'#' if b.get(i + 1).is_some_and(|&c| is_ident(c) || c == b'-') => {
                let e = dashed_ident_end(b, i + 1);
                let kind = if state == St::Value {
                    Kind::Constant // #rrggbb
                } else {
                    Kind::Tag // #id
                };
                out.push(i, e, kind);
                i = e;
            }
            b'.' if state == St::Sel && b.get(i + 1).is_some_and(|&c| is_ident_start(c)) => {
                let e = dashed_ident_end(b, i + 1);
                out.push(i, e, Kind::Tag);
                i = e;
            }
            b':' if state == St::Sel
                && b.get(i + 1)
                    .is_some_and(|&c| c == b':' || is_ident_start(c)) =>
            {
                let start = if b[i + 1] == b':' { i + 2 } else { i + 1 };
                let e = dashed_ident_end(b, start);
                out.push(i, e, Kind::Tag);
                i = e;
            }
            b'[' if state == St::Sel => {
                out.push(i, i + 1, Kind::Punctuation);
                in_attr_sel = true;
                i += 1;
            }
            b']' => {
                out.push(i, i + 1, Kind::Punctuation);
                in_attr_sel = false;
                i += 1;
            }
            b'0'..=b'9' => {
                let e = number(b, i);
                out.push(i, e, Kind::Number);
                i = e;
            }
            b'.' if b.get(i + 1).is_some_and(u8::is_ascii_digit) => {
                let e = number(b, i);
                out.push(i, e, Kind::Number);
                i = e;
            }
            b'-' if b
                .get(i + 1)
                .is_some_and(|&c| c.is_ascii_digit() || c == b'.') =>
            {
                let e = number(b, i + 1);
                out.push(i, e, Kind::Number);
                i = e;
            }
            b'-' if b.get(i + 1) == Some(&b'-') => {
                // A custom property: declared in Prop state, read in Value state.
                let e = dashed_ident_end(b, i);
                let kind = if state == St::Value {
                    Kind::Variable
                } else {
                    Kind::Property
                };
                out.push(i, e, kind);
                i = e;
            }
            _ if is_ident_start(c) || (c == b'-' && b.get(i + 1).is_some_and(|&c| is_ident(c))) => {
                let e = dashed_ident_end(b, i);
                let word = &src[i..e];
                let kind = if in_attr_sel {
                    Kind::Attribute
                } else {
                    match state {
                        St::Sel => Kind::Tag,
                        St::Prop => Kind::Property,
                        St::Prelude => {
                            if b.get(e) == Some(&b'(') {
                                Kind::Function
                            } else if b.get(e) == Some(&b':') {
                                Kind::Property
                            } else if matches!(word, "and" | "not" | "only" | "or") {
                                Kind::Keyword
                            } else {
                                Kind::Plain
                            }
                        }
                        St::Value => {
                            if b.get(e) == Some(&b'(') {
                                Kind::Function
                            } else {
                                Kind::Plain
                            }
                        }
                    }
                };
                out.push(i, e, kind);
                i = e;
            }
            b'(' | b')' | b',' | b':' => {
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

/// An identifier that may contain `-`, as CSS names do.
fn dashed_ident_end(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && (is_ident(b[i]) || b[i] == b'-') {
        i += 1;
    }
    i
}

/// A number plus its unit, `%` included.
fn number(b: &[u8], i: usize) -> usize {
    let e = scan_number(b, i);
    if b.get(e) == Some(&b'%') { e + 1 } else { e }
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
