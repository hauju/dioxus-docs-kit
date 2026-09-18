# hl-lite

A tiny, dependency-free syntax highlighter for documentation sites.

`hl-lite` turns source text into a flat list of classified spans. Twelve
hand-written byte-level lexers, no regex engine, no C, **no dependencies at
all**, `#![forbid(unsafe_code)]` — about **48 KB of WebAssembly** (19 KB
brotli) for all twelve languages, against the ~6 MB a tree-sitter grammar set
costs, and a sub-second cold build instead of ~73 CPU-seconds.

It is deliberately *lexical*, not scope-aware: the fidelity of
[highlight.js](https://highlightjs.org) or Prism, not of a parser. That is
what a fenced code block in a documentation page needs.

## Usage

```rust
use hl_lite::{highlight, Lang};

let lang = Lang::from_slug("rs").expect("known fence slug");

let mut html = String::new();
for span in highlight(lang, "let x = 1; // hi") {
    html.push_str(&format!(
        "<span class=\"hl-{}\">{}</span>",
        span.kind.class(),
        span.text, // escape this in real code
    ));
}
```

The whole API:

```rust
pub enum Lang { Rust, Bash, Css, Dockerfile, Html, JavaScript, Json, Markdown,
                Python, Toml, TypeScript, Yaml }
pub enum Kind { Plain, Keyword, String, Comment, Number, Type, Function,
                Attribute, Property, Tag, Operator, Punctuation, Constant, Variable }
pub struct Span<'a> { pub kind: Kind, pub text: &'a str }

impl Lang { fn from_slug(slug: &str) -> Option<Lang>; fn slug(self) -> &'static str; }
impl Kind { fn class(self) -> &'static str; }          // Keyword -> "keyword"

pub fn highlight(lang: Lang, src: &str) -> Vec<Span<'_>>;
```

### Guarantees

- Concatenating every `Span::text` reproduces the input byte for byte, so a
  renderer can never drop or duplicate source.
- Spans never split a `char`; non-ASCII input cannot panic.
- Unterminated strings, comments, heredocs and templates run to the end of the
  input — no panic, no infinite loop.
- One forward pass per input: O(n).
- Adjacent spans of the same kind are merged.
- `Kind::Plain` covers whitespace and ordinary identifiers.

## Languages

`Lang::from_slug` is case-insensitive and accepts the usual fence aliases.

| Language | Slugs | Notable limitations |
|---|---|---|
| Rust | `rust`, `rs` | Any capitalised identifier is a `Type`, so `SCREAMING_SNAKE` constants are typed too. Lifetimes are `Constant`. Raw identifiers (`r#type`) are not special-cased. |
| Bash | `bash`, `sh`, `shell`, `zsh`, `console` | `${…}` and `$(…)` are single `Variable` spans — not lexed inside. Heredoc bodies are one `String`, interpolations included. Only a short list of commands is highlighted, and only in command position. A `$ ` prompt at the start of a line is handled; a `#` prompt is read as a comment, as in a real shell. |
| CSS | `css` | `url(…)` contents are not lexed. At-rule preludes are looser than declarations: a name before `:` is a `Property`, everything else plain. |
| Dockerfile | `dockerfile`, `docker` | The shell inside `RUN` is not lexed as Bash. `RUN <<EOF` heredocs are not recognised. |
| HTML / XML / SVG | `html`, `xml`, `svg` | `<script>` and `<style>` bodies stay `Plain` — no recursion into JS or CSS. That is also what keeps `a < b` inside a script from opening a tag. |
| JavaScript | `javascript`, `js`, `jsx`, `mjs`, `cjs` | `${…}` holes in template literals are `Plain`. Regex detection follows the previous significant token; a `/` that does not close on its line is division. JSX tags lex as operators and identifiers. Any capitalised identifier is a `Type`. |
| TypeScript | `typescript`, `ts`, `tsx` | As JavaScript, plus type-level keywords and the primitive type names. |
| JSON | `json`, `jsonc`, `json5` | Tolerates `//` and `/* */` comments and trailing commas; it highlights, it does not validate. |
| Markdown | `markdown`, `md`, `mdx` | Fenced blocks are one `String` and are never lexed as their language — the renderer splits fences out first. Tables, setext headings, reference links and indented code blocks stay plain. A `---` frontmatter fence reads as a thematic break and the frontmatter body is lexed as Markdown. |
| Python | `python`, `py`, `py3` | f-string holes stay inside the `String`. Decorators are recognised at the start of a line. Only `class`/`def` names become `Type`/`Function`; other capitalised names stay plain. The builtin list is short. |
| TOML | `toml` | Dotted keys are emitted as `Property` `.` `Property`. Date-times are recognised by shape and emitted as `Number`. |
| YAML | `yaml`, `yml` | Plain scalars stay `Plain` (which is correct, if sparse). A `,` inside a block-context scalar reads as punctuation. Block scalars (`\|`, `>`) take every following line indented deeper than the line that introduced them. A `*` opening a plain scalar reads as an alias. |

Anything else — `Lang::from_slug` returns `None`, and the caller renders the
block as plain text.

## Adding a language

1. Add a variant to `Lang`, its slugs to `SLUGS`, and an arm to `Lang::slug`
   and to `highlight` in `src/lib.rs`.
2. Write `src/<lang>.rs` with `pub(crate) fn lex(out: &mut Emit<'_>)`. Scan
   bytes, and report only what you recognise with `out.push(start, end, kind)`
   — the gaps become `Plain` automatically, which is what makes the
   round-trip guarantee hold. Reuse the scanners in `src/util.rs`, and only
   ever cut on ASCII bytes.
3. Add a `tests/fixtures/<lang>.<ext>` sample of 30–80 realistic lines, and an
   entry in `FIXTURES` and `ALL_LANGS` in `tests/highlight.rs`.
4. Generate the golden, then **read it** (see below).

Two rules keep the invariant tests passing: every loop iteration must advance
the cursor, and a scan that can fail must not be retried at the next byte
without a guard — see `no_regex_until` in `src/js.rs` for the pattern that
keeps a pathological line from turning into O(n²).

## Updating the goldens

`tests/fixtures/*.tokens` holds one line per span, `Kind<TAB>text`, with `\\`,
`\n`, `\t` and `\r` escaped. These files *are* the specification, so a diff in
them is a deliberate change to the output.

```sh
cargo test -p hl-lite                     # compare against the goldens
HL_LITE_UPDATE=1 cargo test -p hl-lite    # rewrite them, then read the diff
```

## License

MIT
