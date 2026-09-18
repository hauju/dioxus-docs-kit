//! Dockerfile lexer.

use crate::Kind;
use crate::emit::Emit;
use crate::util::{char_len, ident_end, is_ident, is_ident_start, line_end, scan_quoted};

const INSTRUCTIONS: &[&str] = &[
    "ADD",
    "ARG",
    "CMD",
    "COPY",
    "ENTRYPOINT",
    "ENV",
    "EXPOSE",
    "FROM",
    "HEALTHCHECK",
    "LABEL",
    "MAINTAINER",
    "ONBUILD",
    "RUN",
    "SHELL",
    "STOPSIGNAL",
    "USER",
    "VOLUME",
    "WORKDIR",
];

pub(crate) fn lex(out: &mut Emit<'_>) {
    let b = out.bytes();
    let src = out.src();
    let n = b.len();
    let mut i = 0;
    // A `\` at the end of the previous line keeps the same logical line going,
    // so no instruction is expected at the start of this one.
    let mut expect_instruction = true;

    while i < n {
        let c = b[i];
        match c {
            b'\n' => {
                expect_instruction = !continued(b, i);
                i += 1;
            }
            b' ' | b'\t' | b'\r' => i += 1,
            b'#' => {
                let e = line_end(b, i);
                out.push(i, e, Kind::Comment);
                i = e;
            }
            b'"' | b'\'' => {
                let e = scan_quoted(b, i, c, true, false);
                out.push(i, e, Kind::String);
                i = e;
            }
            b'$' => {
                let e = match b.get(i + 1) {
                    Some(b'{') => brace_end(b, i + 1),
                    Some(&c) if is_ident_start(c) => ident_end(b, i + 1),
                    _ => i + 1,
                };
                out.push(i, e, Kind::Variable);
                i = e;
            }
            b'-' if (i == 0 || b[i - 1].is_ascii_whitespace())
                && b.get(i + 1)
                    .is_some_and(|&c| c.is_ascii_alphanumeric() || c == b'-') =>
            {
                let mut e = i + 1;
                while e < n && (is_ident(b[e]) || b[e] == b'-') {
                    e += 1;
                }
                out.push(i, e, Kind::Attribute); // --from=builder, -y
                i = e;
            }
            b'\\' => {
                out.push(i, i + 1, Kind::Operator);
                i += 1;
            }
            _ if is_word(c) => {
                let e = word_end(b, i);
                if c.is_ascii_digit() {
                    out.push(i, e, Kind::Number); // 8080, 1.96.0, 30s
                } else {
                    let word = &src[i..ident_end(b, i)];
                    if expect_instruction
                        && e == i + word.len()
                        && INSTRUCTIONS.iter().any(|k| k.eq_ignore_ascii_case(word))
                    {
                        out.push(i, e, Kind::Keyword);
                        expect_instruction = false;
                    } else if word.eq_ignore_ascii_case("as") && e == i + 2 {
                        out.push(i, e, Kind::Keyword);
                    }
                }
                i = e;
            }
            _ if c.is_ascii_punctuation() => {
                out.push(i, i + 1, Kind::Operator);
                i += 1;
            }
            _ => i += char_len(c),
        }
    }
}

/// Shell words: paths, hyphenated package names and versions stay in one
/// piece rather than being chopped up by `/`, `-` and `.`.
fn is_word(c: u8) -> bool {
    is_ident(c) || matches!(c, b'-' | b'.' | b'/' | b'+' | b'~' | b'*')
}

fn word_end(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && is_word(b[i]) {
        i += 1;
    }
    i
}

/// Does the line ending at the `\n` at `i` end with a continuation?
fn continued(b: &[u8], i: usize) -> bool {
    let mut j = i;
    while j > 0 && matches!(b[j - 1], b'\r' | b' ' | b'\t') {
        j -= 1;
    }
    j > 0 && b[j - 1] == b'\\'
}

fn brace_end(b: &[u8], i: usize) -> usize {
    let mut j = i;
    while j < b.len() {
        if b[j] == b'}' {
            return j + 1;
        }
        if b[j] == b'\n' {
            return j;
        }
        j += 1;
    }
    b.len()
}
