#!/usr/bin/env python3
"""Exercise the real docs/blog search in Chromium (requires agent-browser 0.23.4).

Start `dx serve --port 18479 --open false`, then run:
    python3 scripts/browser-smoke.py http://127.0.0.1:18479
"""

import secrets
import subprocess
import sys
import urllib.request
import urllib.error
from html.parser import HTMLParser


origin = (sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:18479").rstrip("/")
session = "docs-kit-smoke-" + secrets.token_hex(4)


def browser(*args):
    print(f"browser: {' '.join(args)}", flush=True)
    result = subprocess.run(
        ["agent-browser", "--session", session, *args],
        capture_output=True, text=True, timeout=60,
    )
    if result.returncode:
        raise RuntimeError(f"Browser command {args!r} failed:\n{result.stdout}{result.stderr}")
    return result.stdout


def expect(expression):
    browser("wait", "--fn", expression)


def open_search():
    browser("click", ".dk-search-trigger")
    expect("document.querySelector('dialog:modal') && document.activeElement.matches('[role=combobox]')")



class PageHead(HTMLParser):
    def __init__(self):
        super().__init__()
        self.canonical = []
        self.noindex = False

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == "link" and attrs.get("rel") == "canonical":
            self.canonical.append(attrs.get("href"))
        if tag == "meta" and attrs.get("name") == "robots":
            self.noindex = "noindex" in attrs.get("content", "")


def check_category_response(path, status, canonical_suffix=None):
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
        assert 'class="dk-blog-category ' in html, "Category should render on the server"
        assert not head.noindex
    else:
        assert head.noindex, "Not-found pages should be noindex"


try:
    browser("open", origin + "/docs/getting-started/introduction")
    browser("set", "viewport", "1280", "900")
    expect("typeof window.__dkSearchHotkey === 'function'")
    expect("getComputedStyle(document.querySelector('dialog')).display === 'none'")
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
    browser("set", "viewport", "390", "844")
    expect("typeof window.__dkSearchHotkey === 'function'")
    browser("press", "Control+k")
    expect("document.querySelector('dialog:modal') && document.activeElement.matches('[role=combobox]')")
    browser("fill", "[role=combobox]", "rust")
    expect("document.querySelectorAll('[role=option]').length > 0")
    browser("press", "Enter")
    expect("location.pathname.startsWith('/blog/') && !document.querySelector('dialog').open")
    # Category URLs must render without JavaScript and return real not-found statuses.
    check_category_response("/blog/categories/rust", 200, "/blog/categories/rust")
    check_category_response("/blog/categories/rust/page/1", 200, "/blog/categories/rust")
    for path in ["/blog/categories/missing", "/blog/categories/rust/page/0", "/blog/categories/rust/page/99"]:
        check_category_response(path, 404)

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
    expect("location.pathname === '/blog/categories/rust' && document.title === 'Rust'")
    browser("click", '.dk-blog-category article h2 a[href="/blog/building-with-dioxus"]')
    expect("location.pathname === '/blog/building-with-dioxus' && !document.querySelector('.dk-blog-category')")
    browser("click", 'article header a[href="/blog/categories/rust"]')
    expect("location.pathname === '/blog/categories/rust' && document.querySelector('.dk-blog-category h1')?.textContent === 'Rust'")
    expect("document.documentElement.scrollWidth <= innerWidth")
    browser("set", "viewport", "1280", "900")
    browser("screenshot", "/tmp/docs-kit-category-desktop.png", "--full")
    print("Browser smoke passed: categories, metadata, SSR/status codes, history, linked badges, mobile layout; docs/blog search, arrow selection, focus containment/restoration, Escape, empty results, and navigation.")
finally:
    # Preserve the original test failure if browser startup itself failed.
    subprocess.run(["agent-browser", "--session", session, "close"], capture_output=True, timeout=15)
