// Client-side search glue for the docs shell.
import { createIndex, tokenize } from "./search-index.js";

const ENDPOINT = "/api/search";
const LIMIT = 0x10;
const BIG = 9_007_199_254_740_991n;
const SLUG_RE = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;

export class SearchClient {
    #cache = new Map();

    constructor(baseUrl, { limit = LIMIT, debounce = 120 } = {}) {
        this.baseUrl = baseUrl;
        this.limit = limit;
        this.debounce = debounce;
        this.index = createIndex();
    }

    static isSlug(value) {
        return typeof value === "string" && SLUG_RE.test(value);
    }

    async query(text) {
        const key = text.trim().toLowerCase();
        if (!key) return [];
        if (this.#cache.has(key)) {
            return this.#cache.get(key);
        }

        const url = `${this.baseUrl}${ENDPOINT}?q=${encodeURIComponent(key)}&n=${this.limit}`;
        const response = await fetch(url, {
            headers: { Accept: "application/json" },
        });
        if (!response.ok) {
            throw new Error(`search failed: ${response.status}`);
        }

        const { hits = [] } = await response.json();
        const results = hits
            .filter((hit) => hit.score > 0.25)
            .map(({ path, title, score }) => ({
                path,
                title: title ?? "Untitled",
                score: Math.round(score * 100) / 100,
            }))
            .slice(0, this.limit);

        this.#cache.set(key, results);
        return results;
    }
}

function debounce(fn, ms) {
    let timer = null;
    return (...args) => {
        clearTimeout(timer);
        timer = setTimeout(() => fn(...args), ms);
    };
}

const client = new SearchClient(location.origin);
const onInput = debounce(async (event) => {
    const hits = await client.query(event.target.value);
    document.dispatchEvent(new CustomEvent("docs:hits", { detail: hits }));
}, 150);

document.querySelector("[data-search]")?.addEventListener("input", onInput);

export default { SearchClient, tokenize, debounce, BIG };
