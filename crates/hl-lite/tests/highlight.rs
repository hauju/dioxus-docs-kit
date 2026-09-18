//! Golden-file and invariant tests.
//!
//! The `.tokens` files next to each fixture are the specification: they are
//! reviewed by eye, and any change to a lexer that moves them has to be
//! deliberate. Regenerate them with:
//!
//! ```sh
//! HL_LITE_UPDATE=1 cargo test -p hl-lite
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use hl_lite::{Kind, Lang, Span, highlight};

/// Every fixture, with the language it is lexed as.
const FIXTURES: &[(Lang, &str)] = &[
    (Lang::Bash, "bash.sh"),
    (Lang::Css, "css.css"),
    (Lang::Dockerfile, "dockerfile.dockerfile"),
    (Lang::Html, "html.html"),
    (Lang::JavaScript, "javascript.js"),
    (Lang::Json, "json.json"),
    (Lang::Markdown, "markdown.md"),
    (Lang::Python, "python.py"),
    (Lang::Rust, "rust.rs"),
    (Lang::Toml, "toml.toml"),
    (Lang::TypeScript, "typescript.ts"),
    (Lang::Yaml, "yaml.yaml"),
];

const ALL_LANGS: &[Lang] = &[
    Lang::Rust,
    Lang::Bash,
    Lang::Css,
    Lang::Dockerfile,
    Lang::Html,
    Lang::JavaScript,
    Lang::Json,
    Lang::Markdown,
    Lang::Python,
    Lang::Toml,
    Lang::TypeScript,
    Lang::Yaml,
];

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c => out.push(c),
        }
    }
    out
}

fn render(spans: &[Span<'_>]) -> String {
    let mut out = String::new();
    for span in spans {
        out.push_str(&format!("{:?}\t{}\n", span.kind, escape(span.text)));
    }
    out
}

#[test]
fn goldens_match() {
    let update = std::env::var_os("HL_LITE_UPDATE").is_some();
    let dir = fixture_dir();
    let mut stale = Vec::new();

    for &(lang, file) in FIXTURES {
        let source = fs::read_to_string(dir.join(file)).expect("fixture is readable");
        let actual = render(&highlight(lang, &source));
        let stem = file.split('.').next().expect("fixture has a stem");
        let golden = dir.join(format!("{stem}.tokens"));

        if update {
            fs::write(&golden, &actual).expect("golden is writable");
            continue;
        }
        match fs::read_to_string(&golden) {
            Ok(expected) if expected == actual => {}
            Ok(expected) => stale.push(format!(
                "{file}: golden differs\n{}",
                first_difference(&expected, &actual)
            )),
            Err(err) => stale.push(format!("{file}: {err}")),
        }
    }

    assert!(
        stale.is_empty(),
        "{}\n\nrerun with HL_LITE_UPDATE=1 to accept",
        stale.join("\n")
    );
}

fn first_difference(expected: &str, actual: &str) -> String {
    for (n, (e, a)) in expected.lines().zip(actual.lines()).enumerate() {
        if e != a {
            return format!("  line {}:\n  - {e}\n  + {a}", n + 1);
        }
    }
    format!(
        "  line count {} vs {}",
        expected.lines().count(),
        actual.lines().count()
    )
}

#[test]
fn spans_reconstruct_every_fixture() {
    let dir = fixture_dir();
    for &(lang, file) in FIXTURES {
        let source = fs::read_to_string(dir.join(file)).expect("fixture is readable");
        // Every language sees every fixture: a Dockerfile lexed as YAML must
        // still round-trip.
        for &other in ALL_LANGS {
            let joined: String = highlight(other, &source)
                .iter()
                .map(|s| s.text)
                .collect::<String>();
            assert_eq!(joined, source, "{file} as {:?} lost input", other.slug());
        }
        assert!(!highlight(lang, &source).is_empty());
    }
}

/// Inputs picked to break a scanner: unterminated everything, lone escapes,
/// non-ASCII, and the shortest form of each construct.
const ADVERSARIAL: &[&str] = &[
    "",
    "\n",
    "\n\n\n",
    " ",
    "\t\t",
    "\"",
    "\"unterminated",
    "\"\\",
    "'",
    "'unterminated",
    "`",
    "`template ${",
    "`${`${`",
    "/*",
    "/* /* nested",
    "//",
    "#",
    "#[",
    "#![derive(",
    "r#\"",
    "b'",
    "'a",
    "<<EOF\nbody without a terminator",
    "<<-'EOF'\n\tstill open",
    "<!--",
    "<div class=\"",
    "<script>a < b",
    "&amp",
    "0x",
    "1e",
    "1.",
    ".5",
    "-",
    "--",
    "---",
    "...",
    "|",
    ">",
    "@",
    "$",
    "${",
    "$(",
    "```",
    "```rust\nfn main() {}",
    "[text](",
    "**bold",
    "key:",
    "- ",
    "!!str",
    "&anchor",
    "héllo 🦀",
    "\"héllo 🦀\"",
    "// héllo 🦀 — em dash",
    "/* 🦀 */ let x = \"日本語\";",
    "ключ: значение",
    "🦀🦀🦀",
    "\u{feff}fn main() {}",
];

#[test]
fn adversarial_inputs_round_trip() {
    for &input in ADVERSARIAL {
        for &lang in ALL_LANGS {
            let spans = highlight(lang, input);
            let joined: String = spans.iter().map(|s| s.text).collect();
            assert_eq!(joined, input, "{:?} lost input on {input:?}", lang.slug());
            assert!(
                spans.iter().all(|s| !s.text.is_empty()),
                "{:?} emitted an empty span on {input:?}",
                lang.slug()
            );
        }
    }
}

#[test]
fn adjacent_spans_have_different_kinds() {
    let dir = fixture_dir();
    for &(lang, file) in FIXTURES {
        let source = fs::read_to_string(dir.join(file)).expect("fixture is readable");
        let spans = highlight(lang, &source);
        for pair in spans.windows(2) {
            assert_ne!(
                pair[0].kind, pair[1].kind,
                "{file}: adjacent spans of the same kind were not merged"
            );
        }
    }
}

#[test]
fn scales_linearly() {
    let dir = fixture_dir();
    for &(lang, file) in FIXTURES {
        let unit = fs::read_to_string(dir.join(file)).expect("fixture is readable");
        let reps = 1_000_000 / unit.len().max(1) + 1;
        let big = unit.repeat(reps);

        let start = Instant::now();
        let spans = highlight(lang, &big);
        let elapsed = start.elapsed();

        assert_eq!(
            spans.iter().map(|s| s.text.len()).sum::<usize>(),
            big.len(),
            "{file}: lost input at 1 MB"
        );
        // Generous: a release build does this in a few milliseconds, but debug
        // builds on a busy CI runner are ~30x slower.
        assert!(
            elapsed.as_secs() < 5,
            "{file}: {} bytes took {elapsed:?}",
            big.len()
        );
        println!("{file}: {} bytes in {elapsed:?}", big.len());
    }
}

#[test]
fn slug_aliases() {
    let table = [
        ("rust", Lang::Rust),
        ("rs", Lang::Rust),
        ("RS", Lang::Rust),
        ("bash", Lang::Bash),
        ("sh", Lang::Bash),
        ("shell", Lang::Bash),
        ("zsh", Lang::Bash),
        ("console", Lang::Bash),
        ("css", Lang::Css),
        ("CSS", Lang::Css),
        ("dockerfile", Lang::Dockerfile),
        ("Dockerfile", Lang::Dockerfile),
        ("docker", Lang::Dockerfile),
        ("html", Lang::Html),
        ("xml", Lang::Html),
        ("svg", Lang::Html),
        ("javascript", Lang::JavaScript),
        ("js", Lang::JavaScript),
        ("jsx", Lang::JavaScript),
        ("mjs", Lang::JavaScript),
        ("cjs", Lang::JavaScript),
        ("json", Lang::Json),
        ("jsonc", Lang::Json),
        ("json5", Lang::Json),
        ("markdown", Lang::Markdown),
        ("md", Lang::Markdown),
        ("mdx", Lang::Markdown),
        ("python", Lang::Python),
        ("py", Lang::Python),
        ("py3", Lang::Python),
        ("toml", Lang::Toml),
        ("typescript", Lang::TypeScript),
        ("ts", Lang::TypeScript),
        ("tsx", Lang::TypeScript),
        ("TSX", Lang::TypeScript),
        ("yaml", Lang::Yaml),
        ("yml", Lang::Yaml),
    ];
    for (slug, lang) in table {
        assert_eq!(Lang::from_slug(slug), Some(lang), "slug {slug}");
    }

    for unknown in ["", "ruby", "go", "c", "text", "plain", "rust-lang"] {
        assert_eq!(Lang::from_slug(unknown), None, "slug {unknown}");
    }

    // Canonical slugs round-trip.
    for &lang in ALL_LANGS {
        assert_eq!(Lang::from_slug(lang.slug()), Some(lang));
    }
}

#[test]
fn class_names_are_distinct() {
    let kinds = [
        Kind::Plain,
        Kind::Keyword,
        Kind::String,
        Kind::Comment,
        Kind::Number,
        Kind::Type,
        Kind::Function,
        Kind::Attribute,
        Kind::Property,
        Kind::Tag,
        Kind::Operator,
        Kind::Punctuation,
        Kind::Constant,
        Kind::Variable,
    ];
    let mut seen: Vec<&str> = kinds.iter().map(|k| k.class()).collect();
    seen.sort_unstable();
    let count = seen.len();
    seen.dedup();
    assert_eq!(seen.len(), count, "two kinds share a CSS class");
}
