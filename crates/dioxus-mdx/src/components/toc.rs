//! Table of contents component for documentation pages.
//!
//! Features:
//! - Displays page headers in a sidebar navigation
//! - Tracks scroll position and highlights the current section
//! - Uses IntersectionObserver for performant scroll tracking

use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::ld_icons::LdList};

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
        use_effect(move || {
            let ids = header_ids_for_effect.clone();
            if ids.is_empty() {
                return;
            }

            // Set up IntersectionObserver and scroll listener via JavaScript
            // Uses data-toc-link attributes to find and update TOC links
            let js = format!(
                r#"
                (function() {{
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
                    }};
                }})();
                "#,
                serde_json::to_string(&ids).unwrap_or_default()
            );

            // Run the JavaScript
            spawn(async move {
                let _ = document::eval(&js);
            });
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
pub fn extract_headers(content: &str) -> Vec<(String, String, u8)> {
    let mut headers = Vec::new();
    let heading_re = regex::Regex::new(r"(?m)^(#{2,4})\s+(.+)$").unwrap();

    for caps in heading_re.captures_iter(content) {
        let level = caps[1].len() as u8;
        let title = caps[2].trim().to_string();
        let id = slugify(&title);
        headers.push((id, title, level));
    }

    headers
}

/// Convert a title to a URL-friendly slug.
pub fn slugify(text: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
