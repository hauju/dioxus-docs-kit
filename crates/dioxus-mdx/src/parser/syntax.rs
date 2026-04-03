//! Syntax highlighting for code blocks using syntect.
//!
//! Generates HTML with CSS classes for code syntax highlighting.
//! Token colors are defined via CSS custom properties so they adapt
//! to both light and dark DaisyUI themes.

use std::sync::LazyLock;
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// Lazily loaded syntax set with default syntaxes.
static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);

/// Apply syntax highlighting to code.
///
/// Returns HTML string with CSS classes for syntax highlighting.
/// Token spans use classes like `sy-keyword`, `sy-string`, etc.
/// that are styled via CSS custom properties (see `syntax_highlight_css()`).
/// Falls back to plain escaped code if highlighting fails.
pub fn highlight_code(code: &str, language: Option<&str>) -> String {
    let lang = language.unwrap_or("txt");

    // Map common language aliases
    let syntax_name = map_language(lang);

    // Find syntax definition
    let syntax = SYNTAX_SET
        .find_syntax_by_extension(syntax_name)
        .or_else(|| SYNTAX_SET.find_syntax_by_name(syntax_name))
        .or_else(|| SYNTAX_SET.find_syntax_by_extension(lang))
        .or_else(|| SYNTAX_SET.find_syntax_by_name(lang))
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    // Use ClassedHTMLGenerator to emit CSS classes instead of inline styles.
    // This lets us control colors via CSS custom properties that adapt to the
    // active DaisyUI theme.
    let mut generator = ClassedHTMLGenerator::new_with_class_style(
        syntax,
        &SYNTAX_SET,
        ClassStyle::SpacedPrefixed { prefix: "sy-" },
    );

    for line in LinesWithEndings::from(code) {
        if generator
            .parse_html_for_line_which_includes_newline(line)
            .is_err()
        {
            return escape_html(code);
        }
    }

    generator.finalize()
}

/// Returns CSS content (without `<style>` tags) for syntax highlighting.
///
/// Defines CSS custom properties for token colors under two selectors:
/// - `[data-theme="dark"]` — colors for dark backgrounds
/// - `[data-theme="light"]` — colors for light backgrounds
///
/// Inject this once via `document::Style` in your layout component.
pub fn syntax_highlight_css() -> &'static str {
    include_str!("syntax_highlight.css")
}

/// Map common language aliases to syntect syntax names.
/// Returns a static string if there's a known mapping, otherwise returns the original.
fn map_language(lang: &str) -> &str {
    // Use case-insensitive matching via eq_ignore_ascii_case
    // JavaScript variants
    if lang.eq_ignore_ascii_case("js") || lang.eq_ignore_ascii_case("javascript") {
        return "JavaScript";
    }
    if lang.eq_ignore_ascii_case("jsx") {
        return "JavaScript (JSX)";
    }
    if lang.eq_ignore_ascii_case("ts") || lang.eq_ignore_ascii_case("typescript") {
        return "TypeScript";
    }
    if lang.eq_ignore_ascii_case("tsx") {
        return "TypeScript (TSX)";
    }

    // Shell variants
    if lang.eq_ignore_ascii_case("sh")
        || lang.eq_ignore_ascii_case("bash")
        || lang.eq_ignore_ascii_case("shell")
        || lang.eq_ignore_ascii_case("zsh")
    {
        return "Bash";
    }

    // Rust
    if lang.eq_ignore_ascii_case("rs") || lang.eq_ignore_ascii_case("rust") {
        return "Rust";
    }

    // Python
    if lang.eq_ignore_ascii_case("py") || lang.eq_ignore_ascii_case("python") {
        return "Python";
    }

    // Ruby
    if lang.eq_ignore_ascii_case("rb") || lang.eq_ignore_ascii_case("ruby") {
        return "Ruby";
    }

    // Go
    if lang.eq_ignore_ascii_case("go") || lang.eq_ignore_ascii_case("golang") {
        return "Go";
    }

    // JSON
    if lang.eq_ignore_ascii_case("json") || lang.eq_ignore_ascii_case("jsonc") {
        return "JSON";
    }

    // YAML
    if lang.eq_ignore_ascii_case("yml") || lang.eq_ignore_ascii_case("yaml") {
        return "YAML";
    }

    // HTML/CSS
    if lang.eq_ignore_ascii_case("html") || lang.eq_ignore_ascii_case("htm") {
        return "HTML";
    }
    if lang.eq_ignore_ascii_case("css") {
        return "CSS";
    }
    if lang.eq_ignore_ascii_case("scss") {
        return "SCSS";
    }
    if lang.eq_ignore_ascii_case("sass") {
        return "Sass";
    }

    // Config files
    if lang.eq_ignore_ascii_case("toml") {
        return "TOML";
    }
    if lang.eq_ignore_ascii_case("ini") {
        return "INI";
    }
    if lang.eq_ignore_ascii_case("env") {
        return "Bourne Again Shell (bash)";
    }

    // Markdown
    if lang.eq_ignore_ascii_case("md") || lang.eq_ignore_ascii_case("markdown") {
        return "Markdown";
    }

    // SQL
    if lang.eq_ignore_ascii_case("sql") {
        return "SQL";
    }

    // C/C++
    if lang.eq_ignore_ascii_case("c") || lang.eq_ignore_ascii_case("h") {
        return "C";
    }
    if lang.eq_ignore_ascii_case("cpp")
        || lang.eq_ignore_ascii_case("cc")
        || lang.eq_ignore_ascii_case("cxx")
        || lang.eq_ignore_ascii_case("hpp")
    {
        return "C++";
    }

    // Java
    if lang.eq_ignore_ascii_case("java") {
        return "Java";
    }

    // C#
    if lang.eq_ignore_ascii_case("cs") || lang.eq_ignore_ascii_case("csharp") {
        return "C#";
    }

    // PHP
    if lang.eq_ignore_ascii_case("php") {
        return "PHP";
    }

    // Swift
    if lang.eq_ignore_ascii_case("swift") {
        return "Swift";
    }

    // Kotlin
    if lang.eq_ignore_ascii_case("kt") || lang.eq_ignore_ascii_case("kotlin") {
        return "Kotlin";
    }

    // Dockerfile
    if lang.eq_ignore_ascii_case("dockerfile") || lang.eq_ignore_ascii_case("docker") {
        return "Dockerfile";
    }

    // Plain text
    if lang.eq_ignore_ascii_case("txt") || lang.eq_ignore_ascii_case("text") {
        return "Plain Text";
    }

    // Default: return the original language string
    lang
}

/// Escape HTML special characters.
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_rust() {
        let code = r#"fn main() {
    println!("Hello, world!");
}"#;
        let html = highlight_code(code, Some("rust"));
        // Should contain syntax highlighting spans with class attributes
        assert!(html.contains("<span"));
        assert!(html.contains("sy-"));
        assert!(html.contains("fn"));
    }

    #[test]
    fn test_highlight_javascript() {
        let code = "const x = 42;";
        let html = highlight_code(code, Some("js"));
        assert!(html.contains("<span"));
    }

    #[test]
    fn test_highlight_no_inline_styles() {
        let code = "let x = 42;";
        let html = highlight_code(code, Some("rust"));
        // Should NOT contain inline style attributes
        assert!(!html.contains("style="));
    }

    #[test]
    fn test_highlight_unknown_language() {
        let code = "some text";
        let html = highlight_code(code, Some("unknown_lang_xyz"));
        // Should still return something
        assert!(!html.is_empty());
    }

    #[test]
    fn test_highlight_no_language() {
        let code = "plain text";
        let html = highlight_code(code, None);
        assert!(!html.is_empty());
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<div>"), "&lt;div&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
    }
}
