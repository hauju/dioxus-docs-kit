//! Syntax highlighting for code blocks using syntect.
//!
//! Generates HTML with inline styles for code syntax highlighting.

use std::sync::LazyLock;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

/// Lazily loaded syntax set with default syntaxes.
static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);

/// Lazily loaded theme set with default themes.
static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

/// Apply syntax highlighting to code.
///
/// Returns HTML string with inline styles for syntax highlighting.
/// Falls back to plain code wrapped in `<code>` if highlighting fails.
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

    // Use a dark theme suitable for dark mode
    // "base16-ocean.dark" is a good dark theme included in syntect
    let theme = THEME_SET
        .themes
        .get("base16-ocean.dark")
        .or_else(|| THEME_SET.themes.get("InspiredGitHub"))
        .unwrap_or_else(|| THEME_SET.themes.values().next().unwrap());

    // Generate highlighted HTML
    match highlighted_html_for_string(code, &SYNTAX_SET, syntax, theme) {
        Ok(html) => {
            // The output is wrapped in <pre style="..."><code>...</code></pre>
            // We want just the inner content since we have our own wrapper
            // Extract the content between <pre...> and </pre>
            if let Some(start) = html.find('>')
                && let Some(end) = html.rfind("</pre>")
            {
                // Trim leading/trailing whitespace from the extracted HTML
                // syntect adds a newline after <pre> which we don't want
                return html[start + 1..end].trim().to_string();
            }
            html
        }
        Err(_) => {
            // Fallback: escape HTML and return plain code
            escape_html(code)
        }
    }
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
        // Should contain syntax highlighting spans
        assert!(html.contains("<span"));
        assert!(html.contains("fn"));
    }

    #[test]
    fn test_highlight_javascript() {
        let code = "const x = 42;";
        let html = highlight_code(code, Some("js"));
        assert!(html.contains("<span"));
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
