//! Minimal YAML-subset parser for MDX and blog frontmatter.
//!
//! Frontmatter is a flat mapping of scalars and string sequences, so a full
//! YAML engine (and the deprecated `serde_yaml` / `unsafe-libyaml` stack it
//! drags into every wasm bundle) is more machinery than the job needs.
//!
//! # Supported subset
//!
//! - `key: value` scalars — plain, `'single-quoted'` (`''` escapes a quote), or
//!   `"double-quoted"` (with `\"`, `\\`, `\n` and `\t` escapes)
//! - unquoted `true` / `false` booleans (also `True`/`False`, `TRUE`/`FALSE`)
//! - block sequences: `key:` followed by `- item` lines
//! - flow sequences: `key: [a, "b"]`
//! - blank lines, `# comment` lines, and trailing ` # comment` after a value
//!
//! Everything else is a parse error: nested mappings, block scalars (`|`, `>`),
//! flow mappings, anchors/aliases/tags, multi-line values, and duplicate keys.

/// A frontmatter block used a shape outside the [supported subset](self).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlLiteError {
    /// Human-readable description, prefixed with the source line where known.
    pub message: String,
}

impl std::fmt::Display for YamlLiteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for YamlLiteError {}

fn err(line: usize, message: impl std::fmt::Display) -> YamlLiteError {
    YamlLiteError {
        message: format!("line {line}: {message}"),
    }
}

/// A value in a [`YamlMap`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YamlValue {
    /// `key:` with nothing after it (and no sequence items following).
    Null,
    /// A string scalar.
    Scalar(String),
    /// An unquoted `true` / `false`.
    Bool(bool),
    /// A block or flow sequence of scalars.
    Sequence(Vec<String>),
}

impl YamlValue {
    fn kind(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Scalar(_) => "a string",
            Self::Bool(_) => "a boolean",
            Self::Sequence(_) => "a sequence",
        }
    }
}

/// A flat frontmatter mapping, in source order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct YamlMap {
    entries: Vec<(String, YamlValue)>,
}

impl YamlMap {
    /// Look up a key.
    pub fn get(&self, key: &str) -> Option<&YamlValue> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// A string field, or `None` when the key is absent or null.
    pub fn optional_str(&self, key: &str) -> Result<Option<String>, YamlLiteError> {
        match self.get(key) {
            None | Some(YamlValue::Null) => Ok(None),
            Some(YamlValue::Scalar(s)) => Ok(Some(s.clone())),
            Some(other) => Err(type_error(key, "a string", other)),
        }
    }

    /// A string field that must be present and non-null.
    pub fn require_str(&self, key: &str) -> Result<String, YamlLiteError> {
        self.optional_str(key)?.ok_or_else(|| YamlLiteError {
            message: format!("missing field `{key}`"),
        })
    }

    /// A boolean field, defaulting to `false` when absent or null.
    pub fn optional_bool(&self, key: &str) -> Result<bool, YamlLiteError> {
        match self.get(key) {
            None | Some(YamlValue::Null) => Ok(false),
            Some(YamlValue::Bool(b)) => Ok(*b),
            Some(other) => Err(type_error(key, "a boolean", other)),
        }
    }

    /// A sequence of scalars, empty when the key is absent or null.
    pub fn optional_str_seq(&self, key: &str) -> Result<Vec<String>, YamlLiteError> {
        match self.get(key) {
            None | Some(YamlValue::Null) => Ok(Vec::new()),
            Some(YamlValue::Sequence(items)) => Ok(items.clone()),
            Some(other) => Err(type_error(key, "a sequence", other)),
        }
    }
}

fn type_error(key: &str, expected: &str, found: &YamlValue) -> YamlLiteError {
    YamlLiteError {
        message: format!("field `{key}`: expected {expected}, found {}", found.kind()),
    }
}

/// Parse a frontmatter block (delimiters already stripped) into a flat mapping.
///
/// See the [module docs](self) for the accepted subset; anything else is an error.
pub fn parse_yaml_lite(input: &str) -> Result<YamlMap, YamlLiteError> {
    let mut map = YamlMap::default();
    // Index of the entry that `- item` lines would extend.
    let mut pending: Option<usize> = None;

    for (idx, raw_line) in input.lines().enumerate() {
        let line = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed == "-" {
            return Err(err(line, "sequence items must be inline scalars"));
        }
        if let Some(item) = trimmed.strip_prefix("- ") {
            let Some(entry) = pending else {
                return Err(err(line, "sequence item outside a `key:` block"));
            };
            let value = parse_scalar(item.trim(), line)?;
            let slot = &mut map.entries[entry].1;
            match slot {
                YamlValue::Null => *slot = YamlValue::Sequence(vec![value]),
                YamlValue::Sequence(items) => items.push(value),
                _ => return Err(err(line, "sequence item cannot follow a scalar value")),
            }
            continue;
        }

        if raw_line.starts_with([' ', '\t']) {
            return Err(err(
                line,
                "nested mappings and multi-line values are not supported",
            ));
        }

        let Some((key, rest)) = trimmed.split_once(':') else {
            return Err(err(line, "expected `key: value`"));
        };
        if !rest.is_empty() && !rest.starts_with([' ', '\t']) {
            return Err(err(line, "expected a space after `key:`"));
        }
        let key = key.trim();
        if key.is_empty() {
            return Err(err(line, "empty key"));
        }
        if key.starts_with(['"', '\'']) {
            return Err(err(line, "quoted keys are not supported"));
        }
        if map.get(key).is_some() {
            return Err(err(line, format!("duplicate key `{key}`")));
        }

        let value = parse_value(rest.trim(), line)?;
        pending = matches!(value, YamlValue::Null).then_some(map.entries.len());
        map.entries.push((key.to_string(), value));
    }

    Ok(map)
}

/// Parse the text after `key:` into a value.
fn parse_value(raw: &str, line: usize) -> Result<YamlValue, YamlLiteError> {
    let Some(first) = raw.chars().next() else {
        return Ok(YamlValue::Null);
    };
    match first {
        '#' => Ok(YamlValue::Null),
        '[' => parse_flow_sequence(raw, line).map(YamlValue::Sequence),
        '{' => Err(err(line, "flow mappings are not supported")),
        '|' | '>' => Err(err(line, "block scalars (`|`, `>`) are not supported")),
        '&' | '*' | '!' => Err(err(
            line,
            "anchors, aliases and tags (`&`, `*`, `!`) are not supported",
        )),
        _ => {
            let quoted = first == '"' || first == '\'';
            let scalar = parse_scalar(raw, line)?;
            if !quoted {
                match scalar.as_str() {
                    "true" | "True" | "TRUE" => return Ok(YamlValue::Bool(true)),
                    "false" | "False" | "FALSE" => return Ok(YamlValue::Bool(false)),
                    "" => return Ok(YamlValue::Null),
                    _ => {}
                }
            }
            Ok(YamlValue::Scalar(scalar))
        }
    }
}

/// Parse a single scalar: plain, single-quoted or double-quoted.
fn parse_scalar(raw: &str, line: usize) -> Result<String, YamlLiteError> {
    let raw = raw.trim();
    if raw.starts_with(['"', '\'']) {
        return parse_quoted(raw, line);
    }
    // In a plain scalar, ` #` starts a trailing comment.
    let value = match raw.find(" #") {
        Some(i) => &raw[..i],
        None => raw,
    };
    Ok(value.trim_end().to_string())
}

fn parse_quoted(raw: &str, line: usize) -> Result<String, YamlLiteError> {
    let quote = if raw.starts_with('"') { '"' } else { '\'' };
    let body = &raw[1..];
    let mut out = String::new();
    let mut chars = body.char_indices();

    while let Some((i, ch)) = chars.next() {
        if quote == '"' && ch == '\\' {
            let Some((_, esc)) = chars.next() else {
                break;
            };
            out.push(match esc {
                '"' => '"',
                '\\' => '\\',
                'n' => '\n',
                't' => '\t',
                other => {
                    return Err(err(
                        line,
                        format!("unsupported escape `\\{other}` in a double-quoted string"),
                    ));
                }
            });
            continue;
        }
        if ch == quote {
            // YAML escapes a quote inside a single-quoted string by doubling it.
            if quote == '\'' && body[i + 1..].starts_with('\'') {
                out.push('\'');
                chars.next();
                continue;
            }
            let tail = body[i + 1..].trim();
            if !tail.is_empty() && !tail.starts_with('#') {
                return Err(err(line, "trailing content after a quoted string"));
            }
            return Ok(out);
        }
        out.push(ch);
    }

    Err(err(line, "unterminated quoted string"))
}

fn parse_flow_sequence(raw: &str, line: usize) -> Result<Vec<String>, YamlLiteError> {
    let close = raw
        .rfind(']')
        .ok_or_else(|| err(line, "unterminated flow sequence (expected a closing `]`)"))?;
    let tail = raw[close + 1..].trim();
    if !tail.is_empty() && !tail.starts_with('#') {
        return Err(err(line, "trailing content after a flow sequence"));
    }

    let inner = raw[1..close].trim();
    if inner.is_empty() {
        return Ok(Vec::new());
    }

    // Split on top-level commas, respecting quotes.
    let mut pieces: Vec<&str> = Vec::new();
    let mut start = 0usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (i, ch) in inner.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match quote {
            Some(q) => {
                if q == '"' && ch == '\\' {
                    escaped = true;
                } else if ch == q {
                    quote = None;
                }
            }
            None => match ch {
                '"' | '\'' => quote = Some(ch),
                '[' | '{' => {
                    return Err(err(line, "nested flow collections are not supported"));
                }
                ',' => {
                    pieces.push(&inner[start..i]);
                    start = i + 1;
                }
                _ => {}
            },
        }
    }
    if quote.is_some() {
        return Err(err(line, "unterminated quoted string in a flow sequence"));
    }
    pieces.push(&inner[start..]);

    pieces
        .into_iter()
        .map(|piece| {
            let piece = piece.trim();
            if piece.is_empty() {
                return Err(err(line, "empty item in a flow sequence"));
            }
            parse_scalar(piece, line)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> YamlMap {
        parse_yaml_lite(input).expect("expected the input to parse")
    }

    #[test]
    fn parses_plain_quoted_and_escaped_scalars() {
        let map = parse(
            "title: Plain Value\nsingle: 'it''s here'\ndouble: \"a \\\"quoted\\\" \\\\ path\"",
        );
        assert_eq!(
            map.optional_str("title").unwrap().as_deref(),
            Some("Plain Value")
        );
        assert_eq!(
            map.optional_str("single").unwrap().as_deref(),
            Some("it's here")
        );
        assert_eq!(
            map.optional_str("double").unwrap().as_deref(),
            Some("a \"quoted\" \\ path")
        );
    }

    #[test]
    fn skips_comment_lines_and_trailing_comments() {
        let map = parse("# leading comment\n\ntitle: Hello # trailing\nicon: \"book\" # also\n");
        assert_eq!(map.optional_str("title").unwrap().as_deref(), Some("Hello"));
        assert_eq!(map.optional_str("icon").unwrap().as_deref(), Some("book"));
    }

    #[test]
    fn hash_without_leading_space_stays_in_a_plain_scalar() {
        let map = parse("color: \"#ff0000\"\nanchor: intro#section");
        assert_eq!(
            map.optional_str("color").unwrap().as_deref(),
            Some("#ff0000")
        );
        assert_eq!(
            map.optional_str("anchor").unwrap().as_deref(),
            Some("intro#section")
        );
    }

    #[test]
    fn parses_block_and_flow_sequences() {
        let block = parse("tags:\n  - rust\n  - \"dioxus\"\ntitle: T");
        assert_eq!(block.optional_str_seq("tags").unwrap(), ["rust", "dioxus"]);
        assert_eq!(block.optional_str("title").unwrap().as_deref(), Some("T"));

        let flow = parse("tags: [\"announcement\", dioxus, 'x, y']");
        assert_eq!(
            flow.optional_str_seq("tags").unwrap(),
            ["announcement", "dioxus", "x, y"]
        );

        assert_eq!(
            parse("tags: []").optional_str_seq("tags").unwrap(),
            Vec::<String>::new()
        );
    }

    #[test]
    fn unindented_block_sequences_are_accepted() {
        let map = parse("tags:\n- rust\n- wasm");
        assert_eq!(map.optional_str_seq("tags").unwrap(), ["rust", "wasm"]);
    }

    #[test]
    fn parses_booleans_only_when_unquoted() {
        let map = parse("draft: true\nfeatured: False\nlabel: \"true\"");
        assert!(map.optional_bool("draft").unwrap());
        assert!(!map.optional_bool("featured").unwrap());
        assert_eq!(map.optional_str("label").unwrap().as_deref(), Some("true"));
        // A quoted "true" is a string, not a boolean.
        assert!(map.optional_bool("label").is_err());
        // Absent booleans default to false.
        assert!(!map.optional_bool("missing").unwrap());
    }

    #[test]
    fn required_and_typed_field_errors() {
        let map = parse("title: Hi\ntags: nope");
        let err = map.require_str("date").unwrap_err();
        assert!(err.to_string().contains("date"), "got: {err}");
        let err = map.optional_str_seq("tags").unwrap_err();
        assert!(
            err.to_string().contains("expected a sequence"),
            "got: {err}"
        );
    }

    #[test]
    fn empty_input_and_null_values() {
        let map = parse("");
        assert_eq!(map.optional_str("title").unwrap(), None);

        let map = parse("description:\ntitle: Hi");
        assert_eq!(map.get("description"), Some(&YamlValue::Null));
        assert_eq!(map.optional_str("description").unwrap(), None);
        assert_eq!(
            map.optional_str_seq("description").unwrap(),
            Vec::<String>::new()
        );
    }

    #[test]
    fn unsupported_shapes_are_errors() {
        for input in [
            "author:\n  name: Jane",           // nested mapping
            "body: |\n  line one\n  line two", // block scalar
            "body: >\n  folded",               // folded scalar
            "base: &anchor value",             // anchor
            "copy: *anchor",                   // alias
            "meta: { a: 1 }",                  // flow mapping
            "- a\n- b",                        // top-level sequence
            "Just a fenced paragraph.",        // not a mapping
            "title: A\ntitle: B",              // duplicate key
            "title: \"unterminated",           // unterminated quote
            "tags: [a, [b]]",                  // nested flow collection
            "tags: [a",                        // unterminated flow sequence
            "title:Hello",                     // missing space after the colon
        ] {
            assert!(
                parse_yaml_lite(input).is_err(),
                "expected an error for: {input:?}"
            );
        }
    }
}
