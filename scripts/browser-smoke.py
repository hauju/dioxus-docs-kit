#!/usr/bin/env python3
"""Exercise docs/blog navigation, metadata, status codes, and search in Chromium (requires agent-browser 0.23.4).

Start `dx serve --port 18479 --open false`, then run:
    python3 scripts/browser-smoke.py http://127.0.0.1:18479
"""

import atexit
import json
import secrets
import subprocess
import sys
import urllib.request
import urllib.error
from html.parser import HTMLParser


origin = (sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:18479").rstrip("/")
session = "docs-kit-smoke-" + secrets.token_hex(4)


def close_session():
    # Otherwise every run leaves a browser daemon and its Chromium processes behind.
    try:
        subprocess.run(["agent-browser", "--session", session, "close"], capture_output=True, timeout=30)
    except (OSError, subprocess.TimeoutExpired):
        pass


atexit.register(close_session)


def browser(*args):
    display = " ".join(args)
    print(f"browser: {display[:180]}" + ("…" if len(display) > 180 else ""), flush=True)
    result = subprocess.run(
        ["agent-browser", "--session", session, *args],
        capture_output=True, text=True, timeout=60,
    )
    if result.returncode:
        raise RuntimeError(f"Browser command {args!r} failed:\n{result.stdout}{result.stderr}")
    return result.stdout


DIAGNOSTICS = (
    "JSON.stringify({href: location.href, readyState: document.readyState, title: document.title, "
    "hotkey: typeof window.__dkSearchHotkey, dialogOpen: document.querySelector('dialog')?.open, "
    "category: !!document.querySelector('.dk-blog-category'), h1: document.querySelector('h1')?.textContent, "
    "active: document.activeElement?.tagName, scrollY, width: innerWidth, height: innerHeight})"
)


def click_element(selector):
    # A wrapped inline link has its bounding-box centre in the gap between
    # its lines (Linux font metrics), where a pointer click hits the parent.
    # Dispatch the click on the element itself instead.
    browser("eval", f"document.querySelector({json.dumps(selector)}).click()")


def expect(expression):
    try:
        browser("wait", "--fn", expression)
    except RuntimeError:
        for label, args in (("page state", ("eval", DIAGNOSTICS)), ("console", ("console",)), ("page errors", ("errors",))):
            try:
                print(f"{label}: " + browser(*args).strip(), flush=True)
            except (RuntimeError, subprocess.TimeoutExpired):
                pass
        raise


def open_search():
    browser("click", ".dk-search-trigger")
    expect("document.querySelector('dialog:modal') && document.activeElement.matches('[role=combobox]')")



class PageHead(HTMLParser):
    def __init__(self):
        super().__init__()
        self.canonical = []
        self.noindex = False
        self.descriptions = []
        self.markdown = []
        self.title = ""
        self.in_title = False

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == "title":
            self.in_title = True
        if tag == "meta" and attrs.get("name") == "description":
            self.descriptions.append(attrs.get("content"))
        if tag == "link" and attrs.get("type") == "text/markdown":
            self.markdown.append(attrs.get("href"))
        if tag == "link" and attrs.get("rel") == "canonical":
            self.canonical.append(attrs.get("href"))
        if tag == "meta" and attrs.get("name") == "robots":
            self.noindex = "noindex" in attrs.get("content", "")

    def handle_data(self, data):
        if self.in_title:
            self.title += data

    def handle_endtag(self, tag):
        if tag == "title":
            self.in_title = False



def check_page_response(path, status, canonical_suffix=None, title=None, description=None, marker=None):
    try:
        response = urllib.request.urlopen(origin + path, timeout=30)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        assert response.status == status, (path, response.status)
        html = response.read().decode()
    head = PageHead()
    head.feed(html)
    if canonical_suffix:
        assert len(head.canonical) == 1 and head.canonical[0].endswith(canonical_suffix), head.canonical
        if marker:
            assert marker in html, "Page should render on the server"
        if title:
            assert head.title == title, head.title
        if description:
            assert head.descriptions == [description], head.descriptions
        assert not head.noindex
    else:
        assert head.noindex, "Not-found pages should be noindex"
        assert not head.canonical and not head.markdown, "Not-found pages should not advertise content URLs"


def expect_head(path, title, description, kind="TechArticle", markdown=True):
    expected = json.dumps(dict(path=path, title=title, description=description, kind=kind, markdown=markdown))
    expect("""(() => {
        const expected = """ + expected + """;
        const one = (selector, attribute, value) => {
            const nodes = document.head.querySelectorAll(selector);
            return nodes.length === 1 && nodes[0].getAttribute(attribute) === value;
        };
        const canonical = document.head.querySelectorAll('link[rel=canonical]');
        const alternates = document.head.querySelectorAll('link[type="text/markdown"]');
        const scripts = document.head.querySelectorAll('script[type="application/ld+json"]');
        if (location.pathname !== expected.path || document.title !== expected.title || canonical.length !== 1 ||
            new URL(canonical[0].href).pathname !== expected.path ||
            !one('meta[name=description]', 'content', expected.description) ||
            !one('meta[property="og:title"]', 'content', expected.title) ||
            !one('meta[property="og:description"]', 'content', expected.description) ||
            !one('meta[property="og:url"]', 'content', canonical[0].href) ||
            !one('meta[name="twitter:title"]', 'content', expected.title) ||
            !one('meta[name="twitter:description"]', 'content', expected.description) ||
            document.head.querySelector('meta[name=robots][content*=noindex]')) return false;
        if (expected.markdown ? (alternates.length !== 1 || new URL(alternates[0].href).pathname !== expected.path + '.md') : alternates.length !== 0) return false;
        if (!expected.kind) return scripts.length === 0;
        if (scripts.length !== 1) return false;
        const data = JSON.parse(scripts[0].textContent);
        const article = (data['@graph'] || [data]).find(item => item['@type'] === expected.kind);
        return article?.headline === expected.title && article.mainEntityOfPage['@id'] === canonical[0].href;
    })()""")


INTRO = ("/docs/getting-started/introduction", "Introduction", "Welcome to the documentation for your Dioxus application")
QUICKSTART = ("/docs/getting-started/quickstart", "Quick Start", "Get up and running with your documentation site in minutes")
API = ("/docs/api-reference/list-pets", "List all pets", "Returns a paginated list of all pets in the store.")
POST = ("/blog/building-with-dioxus", "Building Web Apps with Dioxus 0.7", "A deep dive into building fullstack web applications with Dioxus 0.7, featuring server functions, signals, and RSX.")


try:
    browser("open", origin + "/docs/getting-started/introduction")
    browser("set", "viewport", "1280", "900")
    expect("typeof window.__dkSearchHotkey === 'function'")
    expect("getComputedStyle(document.querySelector('dialog')).display === 'none'")
    for path, title, description in [INTRO, QUICKSTART, API, POST]:
        check_page_response(path, 200, path, title, description)
    expect_head(*INTRO)
    browser("click", '.dk-sidebar a[href="/docs/getting-started/quickstart"]')
    expect_head(*QUICKSTART)
    browser("back")
    expect_head(*INTRO)
    browser("forward")
    expect_head(*QUICKSTART)
    open_search()
    browser("fill", "[role=combobox]", "List all pets")
    expect("document.querySelectorAll('[role=option]').length > 0")
    browser("press", "Enter")
    expect_head(*API, markdown=False)
    browser("back")
    expect_head(*QUICKSTART)

    open_search()
    browser("fill", "[role=combobox]", "theme")
    expect("document.querySelectorAll('[role=option]').length > 1")
    browser("press", "ArrowDown")
    expect("document.querySelectorAll('[role=option]')[1].getAttribute('aria-selected') === 'true'")
    expect("document.activeElement.getAttribute('aria-activedescendant') === document.querySelector('[aria-selected=true]').id")
    browser("press", "ArrowUp")
    browser("press", "ArrowUp")
    expect("[...document.querySelectorAll('[role=option]')].at(-1).getAttribute('aria-selected') === 'true'")

    # Tab stays inside the dialog; Escape also works from its close button.
    browser("press", "Tab")
    expect("document.activeElement.getAttribute('aria-label') === 'Close search'")
    browser("press", "Tab")
    expect("document.activeElement.matches('[role=combobox]')")
    browser("press", "Shift+Tab")
    browser("press", "Escape")
    expect("!document.querySelector('dialog').open && document.activeElement.matches('.dk-search-trigger')")

    open_search()
    expect("document.querySelector('[role=combobox]').value === ''")
    browser("fill", "[role=combobox]", "theme")
    expect("document.querySelectorAll('[role=option]').length > 1")
    browser("press", "ArrowDown")
    expect("document.querySelectorAll('[role=option]')[1].getAttribute('aria-selected') === 'true'")
    browser("eval", "window.__docsKitExpectedTarget = document.querySelector('[aria-selected=true]').dataset.target")
    browser("press", "Enter")
    expect("decodeURI(location.pathname + location.hash) === '/docs/' + window.__docsKitExpectedTarget")
    expect("!document.querySelector('dialog').open && document.querySelector('.dk-article h1')")

    open_search()
    browser("fill", "[role=combobox]", "zzzz-no-matching-document-zzzz")
    expect("document.querySelectorAll('[role=option]').length === 0 && !document.querySelector('[role=combobox]').hasAttribute('aria-activedescendant')")
    browser("press", "ArrowDown")
    browser("press", "Enter")
    expect("document.querySelector('dialog').open")
    browser("press", "Escape")

    # The shared shell must behave identically on a small blog viewport.
    expect("!document.querySelector('dialog').open")
    browser("scrollintoview", 'a[href="/blog"]')
    browser("click", 'a[href="/blog"]')
    expect("location.pathname === '/blog' && document.querySelector('[role=combobox]').getAttribute('aria-label') === 'Search posts...'")
    expect_head("/blog", "Blog", "Latest blog posts and updates.", kind=None, markdown=False)
    browser("set", "viewport", "390", "844")
    expect("typeof window.__dkSearchHotkey === 'function'")
    browser("press", "Control+k")
    expect("document.querySelector('dialog:modal') && document.activeElement.matches('[role=combobox]')")
    browser("fill", "[role=combobox]", "rust")
    expect("document.querySelectorAll('[role=option]').length > 0")
    browser("press", "Enter")
    expect("location.pathname.startsWith('/blog/') && !document.querySelector('dialog').open")
    # Category URLs must render without JavaScript and return real not-found statuses.
    check_page_response("/blog/categories/rust", 200, "/blog/categories/rust")
    check_page_response("/blog/categories/rust/page/1", 200, "/blog/categories/rust")
    for path in ["/blog/categories/missing", "/blog/categories/rust/page/0", "/blog/categories/rust/page/99"]:
        check_page_response(path, 404)

    browser("open", origin + "/blog/categories/rust")
    expect("typeof window.__dkSearchHotkey === 'function'")
    expect("document.querySelector('.dk-blog-category h1')?.textContent === 'Rust'")
    expect("document.querySelectorAll('.dk-blog-category article').length === 2")
    expect("document.querySelectorAll('link[rel=canonical]').length === 1 && document.querySelector('link[rel=canonical]').href.endsWith('/blog/categories/rust')")
    browser("screenshot", "/tmp/docs-kit-category-mobile.png", "--full")
    browser("click", '.dk-blog-category nav[aria-label="Blog categories"] a[href="/blog/categories/dioxus"]')
    expect("location.pathname === '/blog/categories/dioxus' && document.querySelector('.dk-blog-category h1')?.textContent === 'Dioxus'")
    expect("document.title === 'Dioxus' && document.querySelectorAll('link[rel=canonical]').length === 1 && document.querySelector('link[rel=canonical]').href.endsWith('/blog/categories/dioxus')")
    expect("document.querySelectorAll('.dk-blog-category article').length === 2 && document.querySelector('.dk-blog-category article a[href=\"/blog/hello-world\"]')")
    expect("document.querySelector('.dk-blog-category nav a[aria-current=page]').getAttribute('href') === '/blog/categories/dioxus'")
    browser("back")
    expect("typeof window.__dkSearchHotkey === 'function'")
    expect("location.pathname === '/blog/categories/rust' && document.title === 'Rust'")
    click_element('.dk-blog-category article h2 a[href="/blog/building-with-dioxus"]')
    expect("location.pathname === '/blog/building-with-dioxus' && !document.querySelector('.dk-blog-category')")
    browser("click", 'article header a[href="/blog/categories/rust"]')
    expect("location.pathname === '/blog/categories/rust' && document.querySelector('.dk-blog-category h1')?.textContent === 'Rust'")
    expect("document.documentElement.scrollWidth <= innerWidth")
    browser("set", "viewport", "1280", "900")
    browser("screenshot", "/tmp/docs-kit-category-desktop.png", "--full")
    # Direct 404s, followed by client navigation away, Back, and Forward.
    for path, recovery, title in [
        ("/docs/missing-page", INTRO[0], "Documentation page not found"),
        ("/docs/api-reference/missing-operation", INTRO[0], "Documentation page not found"),
        ("/blog/missing-post", "/blog", "Post not found"),
    ]:
        check_page_response(path, 404)
        browser("open", origin + path)
        expect("typeof window.__dkSearchHotkey === 'function'")
        expect("document.title === " + json.dumps(title) + " && document.querySelector('meta[name=robots][content*=noindex]')")
        browser("click", 'a.btn-primary[href="' + recovery + '"]')
        if recovery == INTRO[0]:
            expect_head(*INTRO)
        else:
            expect_head("/blog", "Blog", "Latest blog posts and updates.", kind=None, markdown=False)
        browser("back")
        expect("document.title === " + json.dumps(title) + " && document.querySelector('meta[name=robots][content*=noindex]') && !document.querySelector('link[rel=canonical]') && !document.querySelector('script[type=\"application/ld+json\"]')")
        browser("forward")
        if recovery == INTRO[0]:
            expect_head(*INTRO)
        else:
            expect_head("/blog", "Blog", "Latest blog posts and updates.", kind=None, markdown=False)

    print("Browser smoke passed: docs/API/blog metadata, 404 recovery, categories, SSR/status codes, history, linked badges, mobile layout; docs/blog search, arrow selection, focus containment/restoration, Escape, empty results, and navigation.")
finally:
    # Preserve the original test failure if browser startup itself failed.
    subprocess.run(["agent-browser", "--session", session, "close"], capture_output=True, timeout=15)
