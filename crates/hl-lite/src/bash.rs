//! Bash / shell lexer. Also used for `console` transcripts.

use crate::Kind;
use crate::emit::Emit;
use crate::util::{
    char_len, ident_end, is_ident, is_ident_start, line_end, scan_quoted, skip_blanks,
};

const KEYWORDS: &[&str] = &[
    "if", "then", "elif", "else", "fi", "for", "while", "until", "do", "done", "case", "esac",
    "in", "function", "select", "coproc", "time", "return", "break", "continue",
];

/// Builtins and everyday commands, highlighted only in command position.
/// Deliberately short: an unknown command stays plain rather than guessing.
const COMMANDS: &[&str] = &[
    "alias", "apt", "apt-get", "awk", "brew", "bun", "bunx", "cargo", "cat", "cd", "chmod",
    "chown", "cp", "curl", "date", "declare", "docker", "dx", "echo", "env", "eval", "exec",
    "exit", "export", "find", "git", "grep", "head", "jq", "just", "kill", "ln", "local", "ls",
    "make", "mkdir", "mv", "node", "npm", "npx", "pip", "pip3", "printf", "pwd", "python",
    "python3", "read", "readonly", "rm", "rmdir", "rustc", "rustup", "scp", "sed", "set", "shift",
    "sleep", "sort", "source", "ssh", "sudo", "tail", "tar", "test", "touch", "trap", "uniq",
    "unset", "wc", "wget", "which", "xargs", "yarn",
];

pub(crate) fn lex(out: &mut Emit<'_>) {
    let b = out.bytes();
    let src = out.src();
    let n = b.len();
    let mut i = 0;
    // Next word would be a command name.
    let mut cmd_pos = true;
    // Heredoc delimiters opened on the current line, in order.
    let mut heredocs: Vec<(&str, bool)> = Vec::new();

    while i < n {
        let c = b[i];
        match c {
            b'\n' => {
                i += 1;
                let pending = std::mem::take(&mut heredocs);
                for (delim, strip) in pending {
                    let e = heredoc_body(b, src, i, delim, strip);
                    out.push(i, e, Kind::String);
                    i = e;
                }
                cmd_pos = true;
            }
            b' ' | b'\t' | b'\r' => i += 1,
            b'#' if token_start(b, i) => {
                let e = line_end(b, i);
                out.push(i, e, Kind::Comment);
                i = e;
            }
            // A `$ ` prompt in a pasted shell session.
            b'$' if b.get(i + 1) == Some(&b' ') && at_line_start(b, i) => {
                out.push(i, i + 1, Kind::Punctuation);
                i += 1;
                cmd_pos = true;
            }
            b'$' => {
                i = variable(out, b, i);
                cmd_pos = false;
            }
            b'\'' => {
                let e = scan_quoted(b, i, b'\'', false, true);
                out.push(i, e, Kind::String);
                i = e;
                cmd_pos = false;
            }
            b'"' => {
                i = double_quoted(out, b, i);
                cmd_pos = false;
            }
            b'`' => {
                let e = scan_quoted(b, i, b'`', true, true);
                out.push(i, e, Kind::String);
                i = e;
                cmd_pos = false;
            }
            b'<' if b.get(i + 1) == Some(&b'<') && b.get(i + 2) != Some(&b'<') => {
                i = heredoc_open(out, b, src, i, &mut heredocs);
                cmd_pos = false;
            }
            b'-' if token_start(b, i)
                && b.get(i + 1)
                    .is_some_and(|&c| c.is_ascii_alphanumeric() || c == b'-') =>
            {
                let e = word_end(b, i);
                out.push(i, e, Kind::Attribute);
                i = e;
                cmd_pos = false;
            }
            b'0'..=b'9' if token_start(b, i) => {
                let e = word_end(b, i);
                let kind = if b[i..e].iter().all(u8::is_ascii_digit) {
                    Kind::Number
                } else {
                    Kind::Plain
                };
                out.push(i, e, kind);
                i = e;
                cmd_pos = false;
            }
            _ if is_ident_start(c) => {
                let e = word_end(b, i);
                let word = &src[i..e];
                if KEYWORDS.contains(&word) {
                    out.push(i, e, Kind::Keyword);
                    // `then`, `do`, `else`, … are followed by a command.
                    cmd_pos = true;
                } else if b.get(e) == Some(&b'=')
                    && b.get(e + 1) != Some(&b'=')
                    && is_bare_name(b, i, e)
                {
                    out.push(i, e, Kind::Variable); // NAME=value assignment
                } else if cmd_pos && COMMANDS.contains(&word) {
                    out.push(i, e, Kind::Function);
                    cmd_pos = false;
                } else {
                    cmd_pos = false;
                }
                i = e;
            }
            b';' | b'(' | b')' | b'{' | b'}' | b'[' | b']' => {
                out.push(i, i + 1, Kind::Punctuation);
                i += 1;
                cmd_pos = matches!(c, b';' | b'(' | b'{');
            }
            b'|' | b'&' => {
                out.push(i, i + 1, Kind::Operator);
                i += 1;
                cmd_pos = true;
            }
            _ if c.is_ascii_punctuation() => {
                out.push(i, i + 1, Kind::Operator);
                i += 1;
            }
            _ => i += char_len(c),
        }
    }
}

/// Word characters, which in a shell include path and flag punctuation.
fn is_word(c: u8) -> bool {
    is_ident(c)
        || matches!(
            c,
            b'-' | b'.' | b'/' | b'+' | b':' | b'@' | b'~' | b'%' | b','
        )
}

fn word_end(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && is_word(b[i]) {
        i += 1;
    }
    i
}

/// A plain shell name — what may appear left of `=` in an assignment.
fn is_bare_name(b: &[u8], start: usize, end: usize) -> bool {
    b[start..end].iter().all(|&c| is_ident(c))
}

/// True when `i` starts a new word (so `#` is a comment and `-x` is a flag,
/// but the `#` of `$#` and the `-` of `a-b` are not).
fn token_start(b: &[u8], i: usize) -> bool {
    i == 0
        || matches!(
            b[i - 1],
            b' ' | b'\t' | b'\n' | b'\r' | b';' | b'|' | b'&' | b'(' | b')' | b'{' | b'}'
        )
}

/// True when only blanks separate `i` from the start of its line.
fn at_line_start(b: &[u8], i: usize) -> bool {
    let mut j = i;
    while j > 0 && (b[j - 1] == b' ' || b[j - 1] == b'\t') {
        j -= 1;
    }
    j == 0 || b[j - 1] == b'\n'
}

/// `$name`, `${…}`, `$(…)`, `$1`, `$?`, …
fn variable(out: &mut Emit<'_>, b: &[u8], i: usize) -> usize {
    let e = match b.get(i + 1) {
        Some(b'{') => balanced(b, i + 1, b'{', b'}'),
        Some(b'(') => balanced(b, i + 1, b'(', b')'),
        Some(&c) if is_ident_start(c) => ident_end(b, i + 1),
        Some(&c) if c.is_ascii_digit() => i + 2,
        Some(b'?' | b'#' | b'@' | b'*' | b'!' | b'$' | b'-' | b'_') => i + 2,
        _ => {
            out.push(i, i + 1, Kind::Operator);
            return i + 1;
        }
    };
    out.push(i, e, Kind::Variable);
    e
}

/// Index just past the `close` matching the `open` at `i`.
fn balanced(b: &[u8], i: usize, open: u8, close: u8) -> usize {
    let mut depth = 0usize;
    let mut j = i;
    while j < b.len() {
        let c = b[j];
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                return j + 1;
            }
        } else if c == b'\'' || c == b'"' {
            j = scan_quoted(b, j, c, c == b'"', true);
            continue;
        }
        j += 1;
    }
    b.len()
}

/// A double-quoted string, with `$…` interpolations picked out.
fn double_quoted(out: &mut Emit<'_>, b: &[u8], i: usize) -> usize {
    let mut j = i + 1;
    let mut seg = i;
    while j < b.len() {
        match b[j] {
            b'\\' => j += 2,
            b'"' => {
                j += 1;
                break;
            }
            b'$' if b.get(j + 1).is_some_and(|&c| {
                matches!(c, b'{' | b'(' | b'*' | b'@' | b'?' | b'#' | b'!')
                    || is_ident_start(c)
                    || c.is_ascii_digit()
            }) =>
            {
                out.push(seg, j, Kind::String);
                j = variable(out, b, j);
                seg = j;
            }
            _ => j += 1,
        }
    }
    let j = j.min(b.len());
    out.push(seg, j, Kind::String);
    j
}

/// `<<EOF`, `<<-EOF`, `<<'EOF'` — records the delimiter for the next newline.
fn heredoc_open<'a>(
    out: &mut Emit<'a>,
    b: &'a [u8],
    src: &'a str,
    i: usize,
    heredocs: &mut Vec<(&'a str, bool)>,
) -> usize {
    let mut j = i + 2;
    let strip = b.get(j) == Some(&b'-');
    if strip {
        j += 1;
    }
    out.push(i, j, Kind::Operator);

    let k = skip_blanks(b, j);
    let (delim_start, delim_end, after) = match b.get(k) {
        Some(&q @ (b'\'' | b'"')) => {
            let e = scan_quoted(b, k, q, false, false);
            let inner_end = if b.get(e.wrapping_sub(1)) == Some(&q) {
                e - 1
            } else {
                e
            };
            (k + 1, inner_end.max(k + 1), e)
        }
        _ => {
            let e = ident_end(b, k);
            (k, e, e)
        }
    };
    if delim_end > delim_start {
        out.push(k, after, Kind::String);
        heredocs.push((&src[delim_start..delim_end], strip));
        after
    } else {
        j
    }
}

/// Everything from `i` through the terminator line, as one string.
fn heredoc_body(b: &[u8], src: &str, start: usize, delim: &str, strip: bool) -> usize {
    let mut i = start;
    while i < b.len() {
        let le = line_end(b, i);
        let mut cs = i;
        if strip {
            while cs < le && b[cs] == b'\t' {
                cs += 1;
            }
        }
        let mut ce = le;
        while ce > cs && matches!(b[ce - 1], b'\r' | b' ' | b'\t') {
            ce -= 1;
        }
        let next = if le < b.len() { le + 1 } else { le };
        if &src[cs..ce] == delim {
            return next;
        }
        if next == i {
            break;
        }
        i = next;
    }
    b.len()
}
