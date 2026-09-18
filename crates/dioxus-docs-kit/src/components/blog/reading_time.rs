use dioxus::prelude::*;
use dioxus_mdx::lucide::{Icon, LdClock};

/// Reading time badge component.
#[component]
pub fn ReadingTimeBadge(minutes: u32) -> Element {
    rsx! {
        div { class: "flex items-center gap-1 text-sm text-base-content/60",
            Icon { class: "size-3.5", icon: LdClock }
            span { "{minutes} min read" }
        }
    }
}
