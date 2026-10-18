import { afterAll, beforeAll, expect, test } from 'bun:test';
import { chromium } from '@playwright/test';

const origin = process.env.DOCS_TEST_URL || 'http://127.0.0.1:18479';
let browser;
beforeAll(async () => { browser = await chromium.launch(); });
afterAll(async () => { await browser?.close(); });

test('hydration, client navigation, and history retain working self-hosted diagrams', async () => {
    const page = await browser.newPage();
    const errors = [];
    const externalScripts = [];
    page.on('pageerror', error => errors.push(error.message));
    await page.route('**/*', route => {
        const request = route.request();
        if (new URL(request.url()).origin !== new URL(origin).origin) {
            if (request.resourceType() === 'script') externalScripts.push(request.url());
            return route.abort();
        }
        return route.continue();
    });
    const diagrams = async () => {
        await page.locator('pre.mermaid').first().locator('..').scrollIntoViewIfNeeded();
        await page.waitForFunction(() => document.querySelectorAll('.dk-diagram svg.pele').length === 2);
        expect(await page.locator('.dk-diagram-error').count()).toBe(0);
        expect(await page.locator('.dk-diagram').first().textContent()).toContain('Write MDX');
    };
    try {
        await page.goto(origin + '/docs/guides/basic-usage');
        await diagrams();
        await page.evaluate(() => { window.navigationMarker = 'same-document'; });
        await page.locator('a.dk-nav-item[href="/docs/guides/customization"]').first().click();
        await page.waitForURL('**/docs/guides/customization');
        await page.waitForFunction(() => document.querySelectorAll('.dk-diagram').length === 0);
        await page.locator('a.dk-nav-item[href="/docs/guides/basic-usage"]').first().click();
        await page.waitForURL('**/docs/guides/basic-usage');
        await diagrams();
        expect(await page.evaluate(() => window.navigationMarker)).toBe('same-document');
        await page.goBack();
        await page.waitForURL('**/docs/guides/customization');
        await page.goForward();
        await page.waitForURL('**/docs/guides/basic-usage');
        await diagrams();
        expect(await page.evaluate(() => window.navigationMarker)).toBe('same-document');
        expect(externalScripts).toEqual([]);
        expect(errors).toEqual([]);
    } finally {
        await page.close();
    }
}, 60000);

test('server-rendered diagram source remains readable without JavaScript', async () => {
    const context = await browser.newContext({ javaScriptEnabled: false });
    try {
        const page = await context.newPage();
        await page.goto(origin + '/docs/guides/basic-usage');
        expect(await page.locator('pre.mermaid').count()).toBe(2);
        expect(await page.locator('pre.mermaid').first().textContent()).toContain('Write MDX');
        expect(await page.locator('pre.mermaid').first().isVisible()).toBe(true);
    } finally {
        await context.close();
    }
}, 30000);
