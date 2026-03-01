//! Mermaid diagram rendering component.
//!
//! Renders fenced `mermaid` code blocks as actual diagrams by loading
//! mermaid.js from CDN on demand. Falls back to displaying the raw
//! mermaid source text if JavaScript is unavailable.

use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

/// Monotonic counter for unique mermaid element IDs.
static MERMAID_ID: AtomicU64 = AtomicU64::new(0);

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
/// jsDelivr, detects the current DaisyUI theme, and calls `mermaid.run()`
/// targeting only this element. A `MutationObserver` on `<html data-theme>`
/// re-renders the diagram when the user toggles themes.
#[component]
pub fn MermaidDiagram(props: MermaidDiagramProps) -> Element {
    let id = use_signal(|| format!("mermaid-{}", MERMAID_ID.fetch_add(1, Ordering::Relaxed)));

    #[allow(unused_variables)]
    let code = props.code.clone();

    #[cfg(target_arch = "wasm32")]
    {
        let element_id = id().clone();
        use_effect(move || {
            let element_id = element_id.clone();
            let code = code.clone();
            spawn(async move {
                let code_json = serde_json::to_string(&code).unwrap_or_default();
                let js = format!(
                    r#"
                    (async function() {{
                        const elId = {element_id_json};
                        const code = {code_json};

                        // Load mermaid.js from CDN once
                        if (!window.mermaid) {{
                            await new Promise((resolve, reject) => {{
                                if (document.querySelector('script[data-mermaid-cdn]')) {{
                                    // Another instance is already loading — wait for it
                                    const check = setInterval(() => {{
                                        if (window.mermaid) {{ clearInterval(check); resolve(); }}
                                    }}, 50);
                                    return;
                                }}
                                const s = document.createElement('script');
                                s.src = 'https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.min.js';
                                s.setAttribute('data-mermaid-cdn', '1');
                                s.onload = () => {{
                                    window.mermaid.initialize({{ startOnLoad: false }});
                                    resolve();
                                }};
                                s.onerror = reject;
                                document.head.appendChild(s);
                            }});
                        }}

                        // Detect DaisyUI theme → mermaid theme
                        function mermaidTheme() {{
                            const dt = document.documentElement.getAttribute('data-theme') || '';
                            return (dt === 'light') ? 'default' : 'dark';
                        }}

                        // Render helper
                        async function render() {{
                            const el = document.getElementById(elId);
                            if (!el) return;
                            // Reset element so mermaid re-parses it
                            el.removeAttribute('data-processed');
                            el.innerHTML = code;
                            try {{
                                await window.mermaid.run({{
                                    nodes: [el],
                                    suppressErrors: true,
                                }});
                            }} catch (_) {{}}
                        }}

                        // Initial render with the right theme
                        window.mermaid.initialize({{ startOnLoad: false, theme: mermaidTheme() }});
                        await render();

                        // Re-render on theme change
                        const observer = new MutationObserver(async () => {{
                            window.mermaid.initialize({{ startOnLoad: false, theme: mermaidTheme() }});
                            await render();
                        }});
                        observer.observe(document.documentElement, {{
                            attributes: true,
                            attributeFilter: ['data-theme'],
                        }});
                    }})();
                    "#,
                    element_id_json = serde_json::to_string(&element_id).unwrap_or_default(),
                    code_json = code_json,
                );
                let _ = document::eval(&js);
            });
        });
    }

    rsx! {
        div { class: "my-6 flex justify-center",
            pre {
                class: "mermaid",
                id: "{id()}",
                "{props.code}"
            }
        }
    }
}
