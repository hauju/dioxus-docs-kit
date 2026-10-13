// Static-export search: the browser-side twin of oxidt-docs-kit-core's
// `search::rank` + `build_snippet` and topcoat-docs-kit's `search_results`
// markup. Keep the weights and markup in step with those.
(async () => {
  const script = document.currentScript;
  const base = script.dataset.base;
  const query = new URLSearchParams(location.search).get("q") || "";
  const input = document.querySelector('.dk-search-form input[name="q"]');
  if (input) input.value = query;
  const terms = query.split(/\s+/).filter(Boolean).map((t) => t.toLowerCase());
  const page = document.querySelector(".dk-search-page");
  if (!page || !terms.length) return;
  document.title = `Search: ${query}`;

  const entries = await (await fetch(`${base}/search-index.json`)).json();
  const tiers = [["title", 4000], ["heading", 3000], ["description", 2000], ["body", 1000]];
  const score = (term, hay, weight) => {
    const pos = hay.indexOf(term);
    if (pos < 0) return null;
    const boundary = pos === 0 || !/[\p{L}\p{N}]/u.test(hay[pos - 1]);
    return weight + (boundary ? 250 : 0) - Math.min([...hay.slice(0, pos)].length, 200);
  };
  const hits = [];
  entries.forEach((e, idx) => {
    let total = 0;
    for (const term of terms) {
      const best = Math.max(...tiers.map(([f, w]) => (e[f] ? score(term, e[f].toLowerCase(), w) ?? -1 : -1)));
      if (best < 0) return;
      total += best;
    }
    hits.push([total, idx, e]);
  });
  hits.sort((a, b) => b[0] - a[0] || a[1] - b[1]);

  const esc = (s) => s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
  const snippet = (text) => {
    const chars = [...text];
    const lower = chars.map((c) => c.toLowerCase());
    const matches = [];
    for (const term of terms) {
      const needle = [...term];
      for (let i = 0; i + needle.length <= lower.length; ) {
        if (needle.every((c, j) => lower[i + j] === c)) {
          matches.push([i, i + needle.length]);
          i += needle.length;
        } else i++;
      }
    }
    if (!matches.length) return "";
    matches.sort((a, b) => a[0] - b[0]);
    const start = Math.max(0, matches[0][0] - 50);
    const end = Math.min(start + 100, chars.length);
    let out = start > 0 ? "…" : "";
    let cursor = start;
    for (let [ms, me] of matches) {
      ms = Math.max(ms, start, cursor);
      me = Math.min(me, end);
      if (ms >= me) continue;
      out += esc(chars.slice(cursor, ms).join(""));
      out += `<mark class="dk-search-mark bg-primary/20 text-primary rounded-sm px-0.5">${esc(chars.slice(ms, me).join(""))}</mark>`;
      cursor = me;
    }
    return out + esc(chars.slice(cursor, end).join("")) + (end < chars.length ? "…" : "");
  };

  const shown = hits.slice(0, 25);
  const summary = shown.length === 1 ? "1 result for" : `${shown.length} results for`;
  const items = shown.map(([, , e]) => {
    const target = e.anchor ? `${e.path}#${e.anchor}` : e.path;
    const [title, context] = e.heading ? [e.heading, e.title] : [e.title, null];
    const snip = snippet(e.body || e.description || "");
    return `<li><a href="${esc(`${base}/docs/${target}`)}" class="dk-search-result block py-4 group">
      <div class="flex items-center gap-2 min-w-0">
        ${e.method ? `<span class="badge badge-xs font-mono font-bold ${e.method_class} shrink-0">${e.method}</span>` : ""}
        ${context ? `<span class="dk-search-context text-sm text-base-content/50 truncate shrink-0">${esc(context)}</span><span class="text-sm text-base-content/30 shrink-0">›</span>` : ""}
        <span class="font-medium truncate group-hover:text-primary transition-colors">${esc(title)}</span>
      </div>
      ${e.breadcrumb ? `<div class="text-xs text-base-content/50 mt-0.5">${esc(e.breadcrumb)}</div>` : ""}
      ${snip ? `<p class="dk-search-snippet text-sm text-base-content/70 mt-1.5">${snip}</p>` : ""}
    </a></li>`;
  });
  page.innerHTML = `<h1 class="text-4xl font-bold tracking-tight mb-3">Search</h1>
    <p class="text-base-content/70 mb-8">${summary} “${esc(query)}”</p>
    ${shown.length ? "" : '<p class="text-base-content/50">No pages match. Try fewer or different words.</p>'}
    <ul class="dk-search-results divide-y divide-base-300">${items.join("")}</ul>`;
})();
