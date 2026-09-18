//! JSON lexer, tolerant of JSONC comments and trailing commas.

use crate::Kind;
use crate::emit::Emit;
use crate::util::{
    char_len, ident_end, is_ident_start, line_end, scan_number, scan_quoted, skip_ws,
};

pub(crate) fn lex(out: &mut Emit<'_>) {
    let b = out.bytes();
    let src = out.src();
    let n = b.len();
    let mut i = 0;

    while i < n {
        let c = b[i];
        match c {
            b'"' => {
                let e = scan_quoted(b, i, b'"', true, false);
                // A string that a `:` follows is a key.
                let kind = if b.get(skip_ws(b, e)) == Some(&b':') {
                    Kind::Property
                } else {
                    Kind::String
                };
                out.push(i, e, kind);
                i = e;
            }
            b'/' if b.get(i + 1) == Some(&b'/') => {
                let e = line_end(b, i);
                out.push(i, e, Kind::Comment);
                i = e;
            }
            b'/' if b.get(i + 1) == Some(&b'*') => {
                let mut e = i + 2;
                while e < n && !(b[e] == b'*' && b.get(e + 1) == Some(&b'/')) {
                    e += 1;
                }
                let e = if e < n { e + 2 } else { n };
                out.push(i, e, Kind::Comment);
                i = e;
            }
            b'-' | b'+' if b.get(i + 1).is_some_and(u8::is_ascii_digit) => {
                let e = scan_number(b, i + 1);
                out.push(i, e, Kind::Number);
                i = e;
            }
            b'0'..=b'9' => {
                let e = scan_number(b, i);
                out.push(i, e, Kind::Number);
                i = e;
            }
            b'{' | b'}' | b'[' | b']' | b',' | b':' => {
                out.push(i, i + 1, Kind::Punctuation);
                i += 1;
            }
            _ if is_ident_start(c) => {
                let e = ident_end(b, i);
                if matches!(&src[i..e], "true" | "false" | "null") {
                    out.push(i, e, Kind::Constant);
                }
                i = e;
            }
            _ => i += char_len(c),
        }
    }
}
