use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::LdX;

use crate::DocsContext;

use super::sidebar::DocsSidebar;

/// Slide-in sidebar drawer for mobile screens.
#[component]
pub fn MobileDrawer(mut open: Signal<bool>) -> Element {
    let ctx = use_context::<DocsContext>();

    // Auto-close on path change
    let current_path = ctx.current_path;
    use_effect(move || {
        let _ = current_path();
        open.set(false);
    });

    let is_open = open();

    let backdrop_class = if is_open {
        "fixed inset-0 z-[60] bg-black/50 lg:hidden transition-opacity duration-300 opacity-100"
    } else {
        "fixed inset-0 z-[60] bg-black/50 lg:hidden transition-opacity duration-300 opacity-0 pointer-events-none"
    };

    let panel_class = if is_open {
        "fixed left-0 top-0 bottom-0 z-[70] w-72 bg-base-200 lg:hidden transition-transform duration-300 translate-x-0 shadow-2xl"
    } else {
        "fixed left-0 top-0 bottom-0 z-[70] w-72 bg-base-200 lg:hidden transition-transform duration-300 -translate-x-full shadow-2xl"
    };

    rsx! {
        // Backdrop
        div {
            class: "{backdrop_class}",
            onclick: move |_| open.set(false),
        }

        // Drawer panel
        div {
            class: "{panel_class}",
            onclick: move |e| e.stop_propagation(),

            // Header
            div { class: "flex items-center justify-between px-4 py-3 border-b border-base-300",
                span { class: "font-semibold text-sm", "Navigation" }
                button {
                    class: "btn btn-ghost btn-xs btn-square",
                    onclick: move |_| open.set(false),
                    Icon { class: "size-4", icon: LdX }
                }
            }

            // Sidebar content
            div { class: "overflow-y-auto h-[calc(100%-3.5rem)] p-6",
                DocsSidebar {}
            }
        }
    }
}
