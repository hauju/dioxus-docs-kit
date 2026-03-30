use dioxus::prelude::*;

use crate::blog::registry::BlogRegistry;

/// Author info display (avatar + name).
#[component]
pub fn AuthorInfo(author_id: String) -> Element {
    let registry = use_context::<&'static BlogRegistry>();

    let Some(author) = registry.get_author(&author_id) else {
        return rsx! { span { class: "text-sm text-base-content/50", "{author_id}" } };
    };

    let inner = rsx! {
        div { class: "flex items-center gap-2",
            if let Some(ref avatar) = author.avatar {
                img {
                    src: "{avatar}",
                    alt: "{author.name}",
                    class: "size-6 rounded-full",
                }
            }
            span { class: "font-medium", "{author.name}" }
        }
    };

    if let Some(ref url) = author.url {
        rsx! {
            a {
                href: "{url}",
                target: "_blank",
                rel: "noopener noreferrer",
                class: "hover:text-primary transition-colors",
                {inner}
            }
        }
    } else {
        rsx! { {inner} }
    }
}
