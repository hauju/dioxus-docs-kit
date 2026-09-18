//! Mermaid diagram rendering component.
//!
//! Renders fenced `mermaid` code blocks as actual diagrams by loading
//! mermaid.js from CDN on demand. Falls back to displaying the raw
//! mermaid source text if JavaScript is unavailable.

use dioxus::prelude::*;

/// Client-side driver for every `<pre class="mermaid">` on the page.
///
/// Deliberately id-free: the server renders each page in a fresh request while
/// the wasm client keeps its own state, so any per-instance counter drifts
/// apart after the first navigation and lookups by id miss the hydrated
/// markup. Selecting on `pre.mermaid:not([data-processed])` cannot drift.
#[cfg(target_arch = "wasm32")]
const MERMAID_JS: &str = r#"
(async function() {
    // Load mermaid.js from CDN once.
    if (!window.mermaid) {
        await new Promise((resolve, reject) => {
            if (document.querySelector('script[data-mermaid-cdn]')) {
                // Another instance is already loading — wait for it.
                const check = setInterval(() => {
                    if (window.mermaid) { clearInterval(check); resolve(); }
                }, 50);
                return;
            }
            const s = document.createElement('script');
            s.src = 'https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.min.js';
            s.setAttribute('data-mermaid-cdn', '1');
            s.onload = () => {
                window.mermaid.initialize({ startOnLoad: false });
                resolve();
            };
            s.onerror = reject;
            document.head.appendChild(s);
        });
    }

    // Detect DaisyUI theme → mermaid theme
    function mermaidTheme() {
        const dt = document.documentElement.getAttribute('data-theme') || '';
        return (dt === 'light') ? 'default' : 'dark';
    }

    function blocks() {
        return [...document.querySelectorAll('pre.mermaid')];
    }

    async function render() {
        for (const el of blocks()) {
            // Dioxus owns the text node and rewrites it when the route
            // changes, which silently drops mermaid's SVG while leaving the
            // marker behind. Drop the marker so the block renders again.
            if (el.hasAttribute('data-processed') && !el.querySelector('svg')) {
                el.removeAttribute('data-processed');
                delete el.dataset.mermaidSrc;
            }
        }
        const nodes = blocks().filter(el => !el.hasAttribute('data-processed'));
        if (nodes.length === 0) return;
        // Stash the source before mermaid replaces it, so a theme switch can
        // restore and re-render the diagram.
        for (const el of nodes) {
            el.dataset.mermaidSrc ??= el.textContent;
        }
        try {
            await window.mermaid.run({ nodes, suppressErrors: true });
        } catch (_) {}
    }

    window.mermaid.initialize({ startOnLoad: false, theme: mermaidTheme() });
    await render();

    // One observer for the whole document — every block re-renders together.
    if (!window.__mermaidThemeObserver) {
        window.__mermaidThemeObserver = new MutationObserver(async () => {
            for (const el of blocks()) {
                if (el.dataset.mermaidSrc === undefined) continue;
                el.textContent = el.dataset.mermaidSrc;
                el.removeAttribute('data-processed');
            }
            window.mermaid.initialize({ startOnLoad: false, theme: mermaidTheme() });
            await render();
        });
        window.__mermaidThemeObserver.observe(document.documentElement, {
            attributes: true,
            attributeFilter: ['data-theme'],
        });
    }
})();
"#;

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
            let _ = document::eval(MERMAID_JS);
        });
    }));

    rsx! {
        div { class: "my-6 flex justify-center",
            pre { class: "mermaid", "{props.code}" }
        }
    }
}
