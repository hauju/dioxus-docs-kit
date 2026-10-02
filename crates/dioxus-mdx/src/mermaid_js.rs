//! The browser-side Mermaid loader, shared by every renderer of
//! `<pre class="mermaid">` blocks so theme handling cannot drift between them.

/// Client-side driver for every `<pre class="mermaid">` on the page.
///
/// Deliberately id-free: the server renders each page in a fresh request while
/// the wasm client keeps its own state, so any per-instance counter drifts
/// apart after the first navigation and lookups by id miss the hydrated
/// markup. Selecting on `pre.mermaid:not([data-processed])` cannot drift.
pub const MERMAID_JS: &str = r#"
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
