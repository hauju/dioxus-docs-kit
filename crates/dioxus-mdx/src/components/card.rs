//! Card and CardGroup components for documentation.

use dioxus::prelude::*;

use crate::components::MdxIcon;
use crate::parser::{CardGroupNode, CardNode};

/// Props for DocCardGroup component.
#[derive(Props, Clone, PartialEq)]
pub struct DocCardGroupProps {
    /// Card group data.
    pub group: CardGroupNode,
    /// Optional callback for internal link clicks.
    /// If provided, internal links call this instead of using `<a>`.
    #[props(optional)]
    pub on_link: Option<EventHandler<String>>,
    /// Base path for doc links (e.g., "/docs").
    #[props(default = "/docs".to_string())]
    pub doc_base_path: String,
}

/// Grid of cards component.
#[component]
pub fn DocCardGroup(props: DocCardGroupProps) -> Element {
    let grid_class = match props.group.cols {
        1 => "grid grid-cols-1 gap-4",
        2 => "grid grid-cols-1 md:grid-cols-2 gap-4",
        3 => "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
        _ => "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4",
    };

    rsx! {
        div { class: "my-6 {grid_class}",
            for (i, card) in props.group.cards.iter().enumerate() {
                DocCard {
                    key: "{i}",
                    card: card.clone(),
                    on_link: props.on_link,
                    doc_base_path: props.doc_base_path.clone(),
                }
            }
        }
    }
}

/// Props for DocCard component.
#[derive(Props, Clone, PartialEq)]
pub struct DocCardProps {
    /// Card data.
    pub card: CardNode,
    /// Optional callback for internal link clicks.
    #[props(optional)]
    pub on_link: Option<EventHandler<String>>,
    /// Base path for doc links.
    #[props(default = "/docs".to_string())]
    pub doc_base_path: String,
}

/// Individual card component.
#[component]
pub fn DocCard(props: DocCardProps) -> Element {
    // Render markdown content
    let html = if !props.card.content.is_empty() {
        markdown::to_html_with_options(&props.card.content, &markdown::Options::gfm())
            .unwrap_or_else(|_| props.card.content.clone())
    } else {
        String::new()
    };

    let card_content = rsx! {
        div { class: "bg-base-300 hover:border-primary/50 transition-colors duration-150 border border-base-content/10 rounded-lg h-full",
            div { class: "p-6",
                // Icon on top
                if let Some(icon) = &props.card.icon {
                    div { class: "text-primary mb-5",
                        MdxIcon { name: icon.clone(), class: "size-6".to_string() }
                    }
                }
                // Title - no underline
                h3 { class: "font-semibold text-base-content mb-2 no-underline",
                    "{props.card.title}"
                }
                // Content/Description - no underlines, plain text color
                if !html.is_empty() {
                    div {
                        class: "text-sm text-base-content/60 leading-relaxed [&>p]:my-0 [&_a]:no-underline [&_a]:text-base-content/60",
                        dangerous_inner_html: html,
                    }
                }
            }
        }
    };

    // Wrap in link if href is present
    if let Some(href) = &props.card.href {
        // Handle internal vs external links
        if href.starts_with("http://") || href.starts_with("https://") {
            rsx! {
                a {
                    href: "{href}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "block no-underline hover:no-underline not-prose",
                    {card_content}
                }
            }
        } else {
            // Convert Mintlify-style paths to internal routing
            let internal_href = convert_doc_href(href, &props.doc_base_path);

            if let Some(on_link) = &props.on_link {
                let href_for_click = internal_href.clone();
                let on_link = *on_link;
                rsx! {
                    button {
                        class: "block no-underline text-left w-full not-prose",
                        onclick: move |_| on_link.call(href_for_click.clone()),
                        {card_content}
                    }
                }
            } else {
                rsx! {
                    a {
                        href: "{internal_href}",
                        class: "block no-underline hover:no-underline not-prose",
                        {card_content}
                    }
                }
            }
        }
    } else {
        card_content
    }
}

/// Convert Mintlify-style doc paths to internal routing.
fn convert_doc_href(href: &str, base_path: &str) -> String {
    // If the href already starts with the base path, return as-is
    if href.starts_with(base_path) {
        return href.to_string();
    }
    // Remove leading slash if present
    let path = href.trim_start_matches('/');
    format!("{}/{}", base_path, path)
}
