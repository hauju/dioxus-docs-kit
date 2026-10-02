//! Mermaid diagram rendering component.
//!
//! Renders fenced `mermaid` code blocks as actual diagrams by loading
//! mermaid.js from CDN on demand. Falls back to displaying the raw
//! mermaid source text if JavaScript is unavailable.

use dioxus::prelude::*;

/// Props for MermaidDiagram component.
#[derive(Props, Clone, PartialEq)]
pub struct MermaidDiagramProps {
    /// Raw mermaid diagram source code.
    pub code: String,
}

/// Renders a mermaid diagram.
///
/// The component outputs a `<pre class="mermaid">` element that mermaid.js
/// recognises. A `use_effect` hook (WASM-only) lazily loads mermaid from
/// jsDelivr, detects the current DaisyUI theme, and calls `mermaid.run()` over
/// every block the page still has to draw. A single `MutationObserver` on
/// `<html data-theme>` re-renders them all when the user toggles themes.
#[component]
pub fn MermaidDiagram(props: MermaidDiagramProps) -> Element {
    #[allow(unused_variables)]
    let code = props.code.clone();

    #[cfg(target_arch = "wasm32")]
    use_effect(use_reactive!(|code| {
        let _ = &code;
        spawn(async move {
            let _ = document::eval(crate::MERMAID_JS);
        });
    }));

    rsx! {
        div { class: "not-prose my-6 flex justify-center",
            pre { class: "mermaid", "{props.code}" }
        }
    }
}
