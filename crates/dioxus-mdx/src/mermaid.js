(function() {
    // Shared by Topcoat pages and Dioxus route effects. Keep loading and
    // observation page-wide, while tracking each block without SSR/client IDs.
    window.__dkPeleScan ??= (() => {
        let library;
        let nextId = 0;
        const states = new WeakMap();
        const observed = new Set();
        const load = url => library ??= import(url)
            .catch(error => { library = undefined; throw error; });

        async function render(el) {
            const state = states.get(el);
            state.pending = true;
            try {
                const pele = await load(state.url);
                const { svg } = await pele.renderAsync(state.source, {
                    fontFamily: getComputedStyle(el.parentElement).fontFamily,
                    idPrefix: 'dk-diagram-' + ++nextId,
                });
                // A route may replace the source or detach the node while
                // the renderer or its diagram-type chunk is downloading.
                if (!current(el, state)) return;
                // Dioxus owns the source text node. Keep it intact so later
                // prop updates reach the live DOM, even after hydration.
                const output = document.createElement('div');
                output.className = 'dk-diagram';
                output.innerHTML = svg;
                el.after(output);
                el.hidden = true;
                state.output = output;
            } catch (error) {
                if (!current(el, state)) return;
                // Both the original source and errors stay plain text.
                const message = document.createElement('span');
                message.className = 'dk-diagram-error';
                message.setAttribute('role', 'status');
                message.textContent = 'Diagram could not be rendered: ' + error.message;
                el.after(message);
                state.output = message;
            } finally {
                state.pending = false;
            }
        }

        function current(el, state) {
            if (!el.isConnected || states.get(el) !== state) return false;
            if (el.textContent === state.source) return true;
            // Also handle source changes during a download, before the next
            // framework effect has had a chance to scan the page.
            scan();
            return false;
        }

        const observer = new IntersectionObserver(entries => {
            for (const { target, isIntersecting } of entries) {
                if (!isIntersecting) continue;
                observer.unobserve(target);
                observed.delete(target);
                render(target);
            }
        }, { rootMargin: '200px' });

        function scan() {
            for (const el of observed) {
                if (!el.isConnected) {
                    observer.unobserve(el);
                    observed.delete(el);
                }
            }
            for (const el of document.querySelectorAll('pre.mermaid')) {
                const state = states.get(el);
                const source = el.textContent;
                const url = el.dataset.peleUrl;
                if (state?.source === source && state.url === url) {
                    if (state.output && el.nextElementSibling === state.output) continue;
                    if (state.pending) continue;
                }
                state?.output?.remove();
                el.hidden = false;
                states.set(el, { source, url });
                observer.observe(el);
                observed.add(el);
            }
        }
        return scan;
    })();
    window.__dkPeleScan();
})();
