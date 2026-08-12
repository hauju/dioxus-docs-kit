//! Table of contents component for documentation pages.
//!
//! Features:
//! - Displays page headers in a sidebar navigation
//! - Tracks scroll position and highlights the current section
//! - Uses IntersectionObserver for performant scroll tracking

use std::sync::LazyLock;

use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::ld_icons::LdList};

static HEADING_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?m)^(#{2,4})\s+(.+)$").unwrap());

/// Props for DocTableOfContents component.
#[derive(Props, Clone, PartialEq)]
pub struct DocTableOfContentsProps {
    /// List of headers: (id, title, level).
    pub headers: Vec<(String, String, u8)>,
}

/// Table of contents sidebar component with scroll tracking.
///
/// Scroll tracking is handled client-side via JavaScript for performance.
/// The component uses data attributes and CSS for active state styling.
#[component]
pub fn DocTableOfContents(props: DocTableOfContentsProps) -> Element {
    // Extract header IDs for the observer
    #[allow(unused_variables)]
    let header_ids: Vec<String> = props.headers.iter().map(|(id, _, _)| id.clone()).collect();

    // Set up IntersectionObserver to track visible sections (client-side only)
    #[cfg(target_arch = "wasm32")]
    {
        let header_ids_for_effect = header_ids.clone();
        use_effect(use_reactive!(|header_ids_for_effect| {
            let ids = header_ids_for_effect.clone();
            if ids.is_empty() {
                return;
            }

            // Set up IntersectionObserver and scroll listener via JavaScript
            // Uses data-toc-link attributes to find and update TOC links
            let js = format!(
                r#"
                (function() {{
                    // Remove the previous page's scroll listener before adding a
                    // new one, so navigation doesn't accumulate handlers.
                    if (window.tocCleanup) {{ window.tocCleanup(); }}

                    const ids = {};

                    // Update active TOC item
                    function setActiveTocItem(activeId) {{
                        // Remove active class from all TOC links
                        document.querySelectorAll('[data-toc-link]').forEach(link => {{
                            link.classList.remove('toc-active');
                            link.classList.add('toc-inactive');
                        }});

                        // Add active class to the current link
                        if (activeId) {{
                            const activeLink = document.querySelector(`[data-toc-link="${{activeId}}"]`);
                            if (activeLink) {{
                                activeLink.classList.remove('toc-inactive');
                                activeLink.classList.add('toc-active');
                            }}
                        }}
                    }}

                    // Find the currently active heading based on scroll position
                    function updateActiveHeading() {{
                        let activeId = null;
                        const scrollPos = window.scrollY + 100; // Offset for fixed header

                        for (const id of ids) {{
                            const el = document.getElementById(id);
                            if (el) {{
                                const rect = el.getBoundingClientRect();
                                const absoluteTop = rect.top + window.scrollY;
                                if (absoluteTop <= scrollPos) {{
                                    activeId = id;
                                }}
                            }}
                        }}

                        setActiveTocItem(activeId);
                    }}

                    // Debounce scroll handler
                    let scrollTimeout;
                    function handleScroll() {{
                        clearTimeout(scrollTimeout);
                        scrollTimeout = setTimeout(updateActiveHeading, 10);
                    }}

                    // Set up scroll listener
                    window.addEventListener('scroll', handleScroll, {{ passive: true }});

                    // Initial update
                    setTimeout(updateActiveHeading, 100);

                    // Store cleanup function
                    window.tocCleanup = () => {{
                        window.removeEventListener('scroll', handleScroll);
                        clearTimeout(scrollTimeout);
                        window.tocCleanup = null;
                    }};
                }})();
                "#,
                serde_json::to_string(&ids).unwrap_or_default()
            );

            // Run the JavaScript
            spawn(async move {
                let _ = document::eval(&js);
            });
        }));

        // Remove the scroll listener when the TOC unmounts (leaving docs pages).
        use_drop(|| {
            let _ = document::eval("if (window.tocCleanup) { window.tocCleanup(); }");
        });
    }

    if props.headers.is_empty() {
        return rsx! {};
    }

    rsx! {
        nav { class: "text-sm",
            h4 { class: "font-semibold text-base-content mb-4 text-xs uppercase tracking-wider flex items-center gap-1.5",
                Icon { class: "size-3.5", icon: LdList }
                "On this page"
            }
            ul { class: "space-y-2.5",
                for (i, (id, title, level)) in props.headers.iter().enumerate() {
                    TocItem {
                        key: "{i}",
                        id: id.clone(),
                        title: title.clone(),
                        level: *level,
                    }
                }
            }
        }
        // CSS for active/inactive states (injected once)
        style {
            r#"
            .toc-active {{
                color: oklch(var(--p)) !important;
                font-weight: 500;
            }}
            .toc-active::before {{
                content: '';
                position: absolute;
                left: -14px;
                top: 50%;
                transform: translateY(-50%);
                width: 3px;
                height: 18px;
                background: oklch(var(--p));
                border-radius: 9999px;
                transition: all 0.15s ease-out;
            }}
            .toc-inactive {{
                color: oklch(var(--bc) / 0.55);
                transition: color 0.15s ease-out;
            }}
            .toc-inactive:hover {{
                color: oklch(var(--bc) / 0.9);
            }}
            "#
        }
    }
}

/// Props for TocItem.
#[derive(Props, Clone, PartialEq)]
struct TocItemProps {
    id: String,
    title: String,
    level: u8,
}

/// Individual TOC item.
#[component]
fn TocItem(props: TocItemProps) -> Element {
    let (indent_class, text_class) = match props.level {
        2 => ("", ""),
        3 => ("ml-4", "text-[13px]"),
        _ => ("ml-6", "text-xs"),
    };

    rsx! {
        li {
            class: "{indent_class} relative",
            a {
                href: "#{props.id}",
                class: "toc-inactive block py-0.5 {text_class}",
                "data-toc-link": "{props.id}",
                onclick: move |evt| {
                    evt.prevent_default();
                    // Smooth scroll to the heading (client-side only)
                    #[cfg(target_arch = "wasm32")]
                    {
                        let id = props.id.clone();
                        spawn(async move {
                            let js = format!(
                                r#"
                                const el = document.getElementById({});
                                if (el) {{
                                    el.scrollIntoView({{ behavior: 'smooth', block: 'start' }});
                                    // Update URL hash without jumping
                                    history.pushState(null, '', '#' + {});
                                }}
                                "#,
                                serde_json::to_string(&id).unwrap_or_default(),
                                serde_json::to_string(&id).unwrap_or_default()
                            );
                            let _ = document::eval(&js);
                        });
                    }
                },
                "{props.title}"
            }
        }
    }
}

/// Extract headers from markdown content for table of contents.
///
/// Fenced code blocks are skipped so a `## Setup` inside a sample does not
/// become a TOC entry linking to an anchor the renderer never emits. Mirrors
/// the fence handling in the docs-kit search splitter and build-time anchor
/// validator.
pub fn extract_headers(content: &str) -> Vec<(String, String, u8)> {
    let mut headers = Vec::new();
    let mut fence: Option<char> = None;

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            let marker = if trimmed.starts_with("```") { '`' } else { '~' };
            match fence {
                None => fence = Some(marker),
                Some(open) if open == marker => fence = None,
                Some(_) => {} // the other marker inside a fence is literal content
            }
            continue;
        }
        if fence.is_some() {
            continue;
        }

        if let Some(caps) = HEADING_RE.captures(line) {
            let level = caps[1].len() as u8;
            let title = caps[2].trim().to_string();
            let id = slugify(&title);
            headers.push((id, title, level));
        }
    }

    headers
}

/// Convert a title to a URL-friendly slug.
///
/// Standard HTML entities are decoded and markdown link syntax is reduced to
/// its text first, so the slug is identical whether the input is raw markdown
/// heading text (TOC, search index, build-time anchor checks) or the
/// HTML-escaped, tag-stripped heading the renderer injects ids from.
pub fn slugify(text: &str) -> String {
    let text = text
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&");
    let text = strip_markdown_links(&text);
    text.to_lowercase()
        .chars()
        .filter_map(|c| {
            if c.is_alphanumeric() {
                Some(c)
            } else if c.is_whitespace() || c == '-' || c == '_' || c == '.' {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Reduce markdown links/images `[text](url)` to their text. The renderer
/// slugs from HTML where the `<a>` tag is already stripped, so raw heading
/// text must shed the link syntax to produce the same slug.
fn strip_markdown_links(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(open) = rest.find('[') {
        if let Some(mid) = rest[open..].find("](") {
            let mid = open + mid;
            if let Some(close) = rest[mid..].find(')') {
                out.push_str(&rest[..open]);
                out.push_str(&rest[open + 1..mid]);
                rest = &rest[mid + close + 1..];
                continue;
            }
        }
        out.push_str(&rest[..=open]);
        rest = &rest[open + 1..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_normalizes_entities_and_links() {
        assert_eq!(slugify("Tips & Tricks"), "tips-tricks");
        assert_eq!(slugify("Tips &amp; Tricks"), "tips-tricks");
        assert_eq!(slugify("Q&A"), "qa");
        assert_eq!(slugify("Q&amp;A"), "qa");
        assert_eq!(slugify("a < b"), "a-b");
        assert_eq!(slugify("a &lt; b"), "a-b");
        assert_eq!(slugify("See [the docs](https://x.y/z)"), "see-the-docs");
        assert_eq!(slugify("See the docs"), "see-the-docs");
        assert_eq!(slugify("Use `cargo build`"), "use-cargo-build");
    }

    #[test]
    fn test_extract_headers() {
        let content = r#"
## Introduction

Some text.

### Getting Started

More text.

## Configuration

### Advanced Options
"#;

        let headers = extract_headers(content);
        assert_eq!(headers.len(), 4);
        assert_eq!(
            headers[0],
            ("introduction".to_string(), "Introduction".to_string(), 2)
        );
        assert_eq!(
            headers[1],
            (
                "getting-started".to_string(),
                "Getting Started".to_string(),
                3
            )
        );
        assert_eq!(
            headers[2],
            ("configuration".to_string(), "Configuration".to_string(), 2)
        );
        assert_eq!(
            headers[3],
            (
                "advanced-options".to_string(),
                "Advanced Options".to_string(),
                3
            )
        );
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Getting Started!"), "getting-started");
        assert_eq!(slugify("API v1.0"), "api-v1-0");
    }

    #[test]
    fn extract_headers_skips_headings_inside_code_fences() {
        let content = "## Real One\n\n```md\n## Fake Heading\n```\n\n### Real Two\n";
        let headers = extract_headers(content);
        let titles: Vec<&str> = headers.iter().map(|(_, t, _)| t.as_str()).collect();
        assert_eq!(titles, vec!["Real One", "Real Two"]);
    }

    #[test]
    fn extract_headers_skips_headings_inside_tilde_fences() {
        let content = "## Real\n\n~~~\n## Fake\n~~~\n";
        let headers = extract_headers(content);
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].1, "Real");
    }

    #[test]
    fn extract_headers_treats_other_marker_inside_fence_as_content() {
        // A ``` line inside a ~~~ fence is literal text, not a fence toggle.
        let content = "~~~\n```\n## Fake\n~~~\n\n## Real\n";
        let headers = extract_headers(content);
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].1, "Real");
    }
}
