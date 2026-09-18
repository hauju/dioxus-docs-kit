//! Span accumulator shared by every lexer.

use crate::{Kind, Span};

/// Collects classified byte ranges and turns them into [`Span`]s.
///
/// A lexer only reports the ranges it recognises; everything in between is
/// filled in as [`Kind::Plain`], which is what makes "the spans concatenate
/// back to the input" true by construction rather than by discipline.
pub(crate) struct Emit<'a> {
    src: &'a str,
    /// `(end offset, kind)`; each range starts where the previous one ended.
    marks: Vec<(usize, Kind)>,
    /// End of the last mark, i.e. the first byte not yet accounted for.
    pos: usize,
}

impl<'a> Emit<'a> {
    pub(crate) fn new(src: &'a str) -> Self {
        Emit {
            src,
            marks: Vec::new(),
            pos: 0,
        }
    }

    /// The input being lexed.
    pub(crate) fn src(&self) -> &'a str {
        self.src
    }

    /// The input as bytes — every lexer scans bytes, never chars.
    pub(crate) fn bytes(&self) -> &'a [u8] {
        self.src.as_bytes()
    }

    /// Classify `src[start..end]` as `kind`.
    ///
    /// Anything between the previous range and `start` becomes `Plain`.
    /// Offsets are clamped into range and snapped back to a `char` boundary,
    /// so a lexer bug can degrade the colouring but can never panic or lose
    /// input.
    pub(crate) fn push(&mut self, start: usize, end: usize, kind: Kind) {
        let start = self.snap(start).max(self.pos);
        let end = self.snap(end).max(start);
        self.mark(start, Kind::Plain);
        self.mark(end, kind);
    }

    fn mark(&mut self, end: usize, kind: Kind) {
        if end <= self.pos {
            return;
        }
        match self.marks.last_mut() {
            Some((last_end, last_kind)) if *last_kind == kind => *last_end = end,
            _ => self.marks.push((end, kind)),
        }
        self.pos = end;
    }

    /// Largest `char` boundary at or below `i`.
    fn snap(&self, i: usize) -> usize {
        let mut i = i.min(self.src.len());
        while !self.src.is_char_boundary(i) {
            i -= 1;
        }
        i
    }

    /// Finish the trailing `Plain` run and materialise the spans.
    pub(crate) fn finish(mut self) -> Vec<Span<'a>> {
        let len = self.src.len();
        self.mark(len, Kind::Plain);
        let mut spans = Vec::with_capacity(self.marks.len());
        let mut start = 0;
        for (end, kind) in self.marks {
            spans.push(Span {
                kind,
                text: &self.src[start..end],
            });
            start = end;
        }
        spans
    }
}
