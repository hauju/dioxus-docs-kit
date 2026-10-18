import { afterAll, afterEach, beforeAll, beforeEach, expect, test } from 'bun:test';
import { chromium } from '@playwright/test';

const loader = await Bun.file('crates/dioxus-mdx/src/mermaid.js').text();
let browser, page, requests, errors;
const source = 'flowchart LR\n    A[Draft] --> B[Published]';
const scan = () => page.evaluate(loader);
const insert = (text = source) => page.evaluate(text => {
    const pre = document.createElement('pre');
    pre.className = 'mermaid';
    pre.dataset.peleUrl = '/assets/pele/lazy.js';
    pre.textContent = text;
    document.body.append(pre);
}, text);
const rendered = () => page.waitForSelector('.dk-diagram > svg.pele');

beforeAll(async () => { browser = await chromium.launch(); });
afterAll(async () => { await browser?.close(); });
beforeEach(async () => {
    requests = [];
    errors = [];
    page = await browser.newPage();
    page.on('pageerror', error => errors.push(error.message));
    // Exercise the checked-in modules from the same origin as the document.
    await page.route('http://docs.test/assets/pele/**', async route => {
        const path = new URL(route.request().url()).pathname.split('/assets/pele/')[1];
        requests.push(path);
        await route.fulfill({ contentType: 'text/javascript', body: await Bun.file('crates/dioxus-mdx/assets/pele/' + path).text() });
    });
    await page.route('http://docs.test/', route => route.fulfill({
        contentType: 'text/html',
        body: '<!doctype html><html data-theme="light"><head><style>body{max-width:700px;margin:20px auto;font-family:system-ui}</style></head><body class="dk-root"></body></html>',
    }));
    await page.goto('http://docs.test/');
    await page.addStyleTag({ content: await Bun.file('crates/dioxus-docs-kit/assets/docs-kit.css').text() });
});
afterEach(async () => { await page.close(); expect(errors).toEqual([]); });

test('loads only near the viewport and repeated scans reuse the diagram', async () => {
    await insert();
    await page.locator('pre').evaluate(el => { el.style.marginTop = '2000px'; });
    await scan();
    await page.waitForTimeout(100);
    expect(requests).toHaveLength(0);
    await page.locator('pre').scrollIntoViewIfNeeded();
    await rendered();
    expect(await page.locator('svg').textContent()).toContain('Published');
    await page.evaluate(() => { window.firstSVG = document.querySelector('svg'); });
    const count = requests.length;
    await scan();
    expect(await page.evaluate(() => document.querySelector('svg') === window.firstSVG)).toBe(true);
    expect(requests.length).toBe(count);
});

test('both kit stylesheets apply theme colors without replacing the SVG', async () => {
    for (const kit of ['dioxus-docs-kit', 'topcoat-docs-kit']) {
        await page.locator('style').last().evaluate(el => el.remove());
        await page.addStyleTag({ content: await Bun.file(`crates/${kit}/assets/docs-kit.css`).text() });
        await insert();
        await scan();
        await rendered();
        await page.evaluate(() => { window.firstSVG = document.querySelector('svg'); });
        const color = await page.locator('svg').evaluate(el => getComputedStyle(el).color);
        await page.evaluate(() => document.documentElement.setAttribute('data-theme', 'dark'));
        expect(await page.locator('svg').evaluate(el => getComputedStyle(el).color)).not.toBe(color);
        expect(await page.evaluate(() => document.querySelector('svg') === window.firstSVG)).toBe(true);
        await page.evaluate(() => { document.body.replaceChildren(); document.documentElement.setAttribute('data-theme', 'light'); });
    }
});

test('route changes render replacement source and newly mounted blocks', async () => {
    await insert();
    await scan();
    await rendered();
    await page.locator('pre').evaluate(el => { el.textContent = 'sequenceDiagram\nAlice->>Bob: Hello'; });
    await scan();
    await page.waitForSelector('svg.pele-sequence');
    expect(await page.locator('svg').textContent()).toContain('Hello');
    await page.evaluate(() => document.body.replaceChildren());
    await insert('flowchart TD\n New --> Page');
    await scan();
    await rendered();
    expect(await page.locator('svg').textContent()).toContain('Page');
});

test('source changes during a download cannot render stale content', async () => {
    let release;
    const gate = new Promise(resolve => { release = resolve; });
    await page.route('**/assets/pele/lazy.js', async route => { await gate; await route.fallback(); });
    await insert();
    const requested = page.waitForRequest('**/assets/pele/lazy.js');
    await scan();
    await requested;
    await page.locator('pre').evaluate(el => { el.textContent = 'flowchart TD\n Latest --> Content'; });
    await scan();
    release();
    await rendered();
    expect(await page.locator('svg').textContent()).toContain('Latest');
    expect(await page.locator('svg').textContent()).not.toContain('Draft');
});

test('invalid diagrams retain safe source, and correction can render', async () => {
    const invalid = 'invalid <img src=x onerror=alert(1)>';
    await insert(invalid);
    await scan();
    await page.waitForSelector('.dk-diagram-error');
    expect(await page.locator('pre').textContent()).toBe(invalid);
    expect(await page.locator('img').count()).toBe(0);
    await page.locator('pre').evaluate((el, text) => { el.textContent = text; }, source);
    await scan();
    await rendered();
    expect(await page.locator('.dk-diagram-error').count()).toBe(0);
});

test('module download failures retain the source and show a readable error', async () => {
    await page.route('**/assets/pele/lazy.js', route => route.abort());
    await insert();
    await scan();
    await page.waitForSelector('.dk-diagram-error');
    expect(await page.locator('pre').textContent()).toContain(source);
});

test('multiple diagrams have distinct IDs and fit narrow screens', async () => {
    await page.setViewportSize({ width: 360, height: 740 });
    const diagram = 'flowchart LR\naccTitle: Example\n A[First step] --> B[Second step] --> C[Last step]';
    await insert(diagram);
    await insert(diagram);
    await scan();
    await page.waitForFunction(() => document.querySelectorAll('svg.pele').length === 2);
    const ids = await page.locator('svg [id]').evaluateAll(nodes => nodes.map(node => node.id));
    expect(ids.length).toBeGreaterThan(0);
    expect(new Set(ids).size).toBe(ids.length);
    for (const svg of await page.locator('svg').all()) expect((await svg.boundingBox()).width).toBeLessThanOrEqual(360);
});

test('the existing documentation diagrams render with Pele', async () => {
    const markdown = await Bun.file('docs/guides/basic-usage.mdx').text();
    const examples = [...markdown.matchAll(/^```mermaid\n([\s\S]*?)^```/gm)];
    expect(examples.length).toBeGreaterThan(0);
    for (const [, code] of examples) await insert(code);
    await scan();
    await page.waitForFunction(count => document.querySelectorAll('svg.pele').length === count, examples.length);
    expect(await page.locator('.dk-diagram-error').count()).toBe(0);
});

test('framework-owned text nodes remain attached across source updates', async () => {
    await insert();
    await page.evaluate(() => { window.sourceNode = document.querySelector('pre').firstChild; });
    await scan();
    await rendered();
    expect(await page.evaluate(() => window.sourceNode.isConnected)).toBe(true);
    await page.evaluate(() => { window.sourceNode.data = 'sequenceDiagram\nAlice->>Bob: Updated'; });
    await scan();
    await page.waitForSelector('svg.pele-sequence');
    expect(await page.locator('svg').textContent()).toContain('Updated');
    expect(await page.locator('.dk-diagram').count()).toBe(1);
});
