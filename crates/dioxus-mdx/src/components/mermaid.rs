//! Mermaid diagram rendering component.
//!
//! Renders fenced `mermaid` code blocks as actual diagrams by loading
//! bundled Pele modules on demand. Falls back to displaying the raw
//! mermaid source text if JavaScript is unavailable.

use dioxus::prelude::*;

const PELE: Asset = asset!("/assets/pele", AssetOptions::folder());

/// Props for MermaidDiagram component.
#[derive(Props, Clone, PartialEq)]
pub struct MermaidDiagramProps {
    /// Raw mermaid diagram source code.
    pub code: String,
}

/// Renders a mermaid diagram.
///
/// The component outputs a `<pre class="mermaid">` element. A `use_effect`
/// hook (WASM-only) registers it with the shared lazy Pele loader. Diagram
/// colors follow the docs theme through CSS variables without re-rendering.
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
            pre {
                class: "mermaid",
                "data-pele-url": format!("{PELE}/lazy.js"),
                "{props.code}"
            }
        }
    }
}
