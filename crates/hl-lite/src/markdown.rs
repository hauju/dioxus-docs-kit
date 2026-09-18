//! Markdown lexer.
//!
//! Line-oriented: block structure first, then a small inline pass over what is
//! left. Fenced code is emitted as one string and is never lexed as its
//! language — the renderer already splits fences out before it gets here.

use crate::Kind;
use crate::emit::Emit;
use crate::util::{char_len, line_end, skip_blanks};

pub(crate) fn lex(out: &mut Emit<'_>) {
    let b = out.bytes();
    let n = b.len();
    let mut i = 0;

    while i < n {
        let le = line_end(b, i);
        let next = if le < n { le + 1 } else { le };
        let cs = skip_blanks(b, i);

        // Fenced code block: everything through the closing fence.
        if let Some((fence, len)) = fence(b, cs, le) {
            let e = fence_block(b, next, fence, len);
            out.push(i, e, Kind::String);
            i = e;
            continue;
        }

        // ATX heading.
        if b.get(cs) == Some(&b'#') {
            let mut h = cs;
            while h < le && b[h] == b'#' && h - cs < 6 {
                h += 1;
            }
            if h > cs && (h >= le || b[h] == b' ') {
                out.push(i, le, Kind::Keyword);
                i = next;
                continue;
            }
        }

        // Thematic break.
        if is_thematic_break(b, cs, le) {
            out.push(cs, le, Kind::Punctuation);
            i = next;
            continue;
        }

        // Block quote and list markers, which can nest.
        let mut j = cs;
        loop {
            let m = skip_blanks(b, j);
            if b.get(m) == Some(&b'>') {
                out.push(m, m + 1, Kind::Punctuation);
                j = m + 1;
                continue;
            }
            if let Some(e) = list_marker(b, m, le) {
                out.push(m, e, Kind::Punctuation);
                j = e;
                continue;
            }
            j = m;
            break;
        }

        inline(out, b, j, le);
        i = next;
    }
}

/// An opening ``` / ~~~ fence, as `(char, run length)`.
fn fence(b: &[u8], cs: usize, le: usize) -> Option<(u8, usize)> {
    let c = *b.get(cs)?;
    if c != b'`' && c != b'~' {
        return None;
    }
    let mut j = cs;
    while j < le && b[j] == c {
        j += 1;
    }
    (j - cs >= 3).then_some((c, j - cs))
}

/// End of a fenced block that opened on the line before `from`.
fn fence_block(b: &[u8], from: usize, fence_char: u8, len: usize) -> usize {
    let n = b.len();
    let mut i = from;
    while i < n {
        let le = line_end(b, i);
        let next = if le < n { le + 1 } else { le };
        let cs = skip_blanks(b, i);
        if let Some((c, l)) = fence(b, cs, le)
            && c == fence_char
            && l >= len
            && skip_blanks(b, cs + l) >= le
        {
            return next;
        }
        i = next;
    }
    n
}

fn is_thematic_break(b: &[u8], cs: usize, le: usize) -> bool {
    let c = match b.get(cs) {
        Some(&c @ (b'-' | b'*' | b'_')) => c,
        _ => return false,
    };
    let mut count = 0;
    for &x in &b[cs..le] {
        if x == c {
            count += 1;
        } else if x != b' ' && x != b'\t' {
            return false;
        }
    }
    count >= 3
}

/// `- `, `* `, `+ `, `1. `, `2) ` — returns the index just past the marker.
fn list_marker(b: &[u8], i: usize, le: usize) -> Option<usize> {
    match b.get(i) {
        Some(b'-' | b'*' | b'+') if b.get(i + 1) == Some(&b' ') => Some(i + 1),
        Some(c) if c.is_ascii_digit() => {
            let mut j = i;
            while j < le && b[j].is_ascii_digit() {
                j += 1;
            }
            if matches!(b.get(j), Some(b'.' | b')')) && b.get(j + 1) == Some(&b' ') {
                Some(j + 1)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Inline spans on one line.
fn inline(out: &mut Emit<'_>, b: &[u8], from: usize, le: usize) {
    let mut j = from;
    // A failed scan means nothing else of that shape can close on this line
    // either; without these the worst case would be quadratic.
    let mut no_code = from;
    let mut no_emph = from;
    let mut no_link = from;

    while j < le {
        match b[j] {
            b'\\' => j += 2,
            b'`' if j >= no_code => {
                let mut run = j;
                while run < le && b[run] == b'`' {
                    run += 1;
                }
                match find_run(b, run, le, b'`', run - j) {
                    Some(e) => {
                        out.push(j, e, Kind::String);
                        j = e;
                    }
                    None => {
                        no_code = le;
                        j = run;
                    }
                }
            }
            c @ (b'*' | b'_') if j >= no_emph => {
                let mut run = j;
                while run < le && b[run] == c && run - j < 3 {
                    run += 1;
                }
                match find_run(b, run, le, c, run - j) {
                    Some(e) => {
                        out.push(j, e, Kind::Operator);
                        j = e;
                    }
                    None => {
                        no_emph = le;
                        j = run;
                    }
                }
            }
            b'[' | b'!'
                if j >= no_link && b.get(if b[j] == b'!' { j + 1 } else { j }) == Some(&b'[') =>
            {
                let open = if b[j] == b'!' { j + 1 } else { j };
                match link(b, open, le) {
                    Some((text_end, url_end)) => {
                        out.push(j, open + 1, Kind::Punctuation);
                        out.push(text_end, text_end + 2, Kind::Punctuation); // `](`
                        out.push(text_end + 2, url_end, Kind::Constant);
                        out.push(url_end, url_end + 1, Kind::Punctuation);
                        j = url_end + 1;
                    }
                    None => {
                        no_link = le;
                        j = open + 1;
                    }
                }
            }
            c => j += char_len(c),
        }
    }
}

/// Index just past a run of `len` copies of `c`, searched in `from..le`.
fn find_run(b: &[u8], from: usize, le: usize, c: u8, len: usize) -> Option<usize> {
    let mut j = from;
    while j < le {
        if b[j] == b'\\' {
            j += 2;
            continue;
        }
        if b[j] == c {
            let mut k = j;
            while k < le && b[k] == c {
                k += 1;
            }
            if k - j >= len {
                return Some(j + len);
            }
            j = k;
            continue;
        }
        j += 1;
    }
    None
}

/// `[text](url)` starting at the `[` — returns `(index of ']', index of ')')`.
fn link(b: &[u8], open: usize, le: usize) -> Option<(usize, usize)> {
    let mut j = open + 1;
    while j < le && b[j] != b']' {
        if b[j] == b'\\' {
            j += 1;
        }
        j += 1;
    }
    if b.get(j) != Some(&b']') || b.get(j + 1) != Some(&b'(') {
        return None;
    }
    let text_end = j;
    let mut k = j + 2;
    while k < le && b[k] != b')' {
        k += 1;
    }
    (b.get(k) == Some(&b')')).then_some((text_end, k))
}
