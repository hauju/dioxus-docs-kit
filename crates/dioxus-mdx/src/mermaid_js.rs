//! The browser-side Mermaid loader, shared by every renderer of
//! `<pre class="mermaid">` blocks so theme handling cannot drift between them.

/// Client-side driver for every `<pre class="mermaid">` on the page.
///
/// Loads mermaid lazily: nothing is fetched until a diagram comes within 200px
/// of the viewport, and then the code-split ESM build pulls in only the core
/// plus the diagram types the page uses (~270 KB compressed for a flowchart
/// and a sequence diagram, against ~935 KB for the all-in-one bundle).
///
/// Safe to run repeatedly (the Dioxus kit runs it on every route change):
/// the loader and both observers are page-wide singletons.
///
/// Deliberately id-free: the server renders each page in a fresh request while
/// the wasm client keeps its own state, so any per-instance counter drifts
/// apart after the first navigation and lookups by id miss the hydrated
/// markup. Selecting on `pre.mermaid` and data attributes cannot drift.
pub const MERMAID_JS: &str = r#"
(function() {
    // The ESM entry point, imported once per page.
    function load() {
        return window.__dkMermaid ??= import('https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs')
            .then(m => m.default);
    }

    // Detect DaisyUI theme → mermaid theme
    function mermaidTheme() {
        const dt = document.documentElement.getAttribute('data-theme') || '';
        return (dt === 'light') ? 'default' : 'dark';
    }

    function blocks() {
        return [...document.querySelectorAll('pre.mermaid')];
    }

    async function render(nodes) {
        if (nodes.length === 0) return;
        // Stash the source before mermaid replaces it, so a theme switch can
        // restore and re-render the diagram. Set before loading, so a block
        // already on its way is not queued a second time.
        for (const el of nodes) {
            el.dataset.mermaidSrc ??= el.textContent;
        }
        const mermaid = await load();
        mermaid.initialize({ startOnLoad: false, theme: mermaidTheme() });
        try {
            await mermaid.run({ nodes, suppressErrors: true });
        } catch (_) {}
    }

    // Render each diagram when it nears the viewport; pages whose diagrams the
    // reader never scrolls to never download mermaid.
    window.__dkMermaidView ??= new IntersectionObserver((entries, observer) => {
        const nodes = entries.filter(e => e.isIntersecting).map(e => e.target);
        for (const el of nodes) observer.unobserve(el);
        render(nodes);
    }, { rootMargin: '200px' });

    for (const el of blocks()) {
        // Dioxus owns the text node and rewrites it when the route changes,
        // which silently drops mermaid's SVG while leaving the marker behind.
        // Drop the marker so the block renders again.
        if (el.hasAttribute('data-processed') && !el.querySelector('svg')) {
            el.removeAttribute('data-processed');
            delete el.dataset.mermaidSrc;
        }
        // Not rendered and not already on its way.
        if (el.dataset.mermaidSrc === undefined) {
            window.__dkMermaidView.observe(el);
        }
    }

    // One observer for the whole document: rendered blocks re-render with the
    // new theme; blocks not rendered yet pick it up when they do.
    if (!window.__mermaidThemeObserver) {
        window.__mermaidThemeObserver = new MutationObserver(() => {
            const rendered = blocks().filter(el => el.hasAttribute('data-processed') && el.dataset.mermaidSrc !== undefined);
            for (const el of rendered) {
                el.textContent = el.dataset.mermaidSrc;
                el.removeAttribute('data-processed');
            }
            render(rendered);
        });
        window.__mermaidThemeObserver.observe(document.documentElement, {
            attributes: true,
            attributeFilter: ['data-theme'],
        });
    }
})();
"#;
