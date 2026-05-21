# dioxus-docs-kit — theming & customization roadmap

This document captures the theming/customization surface we expose to consumers,
plus a set of larger proposals that community contributors can pick up.

## Guiding principles

1. **Don't remove opinionated defaults.** The out-of-the-box look should still
   be great with zero config. Additions are opt-in.
2. **Prefer CSS custom properties over props for visual tokens.** Easier to
   theme, no component re-renders, SSR-friendly.
3. **Keep the HTML semantic.** Consumers should be able to restyle without
   fighting the markup. Structural nodes are tagged with stable `dk-*` public
   class tokens (`dk-root`, `dk-article`, `dk-search-trigger`, etc.) so CSS
   overrides don't drift across minor versions.

---

## Proposal 1 — Public CSS custom-property surface

**Problem.** The crate currently inherits DaisyUI's tokens (`--p`, `--b1`,
`--bc`, etc.). Consumers not using DaisyUI have to fake those variables or
override deeply-specific selectors.

**Proposal.** Ship an authoritative list of documented CSS variables that map
to every themeable token, with the DaisyUI tokens as a default fallback:

```css
.dk-root {
  --dk-bg: var(--b1, #ffffff);
  --dk-bg-sub: var(--b2, #f5f5f5);
  --dk-border: var(--b3, #e5e5e5);
  --dk-fg: var(--bc, #111);
  --dk-muted: color-mix(in srgb, var(--dk-fg) 65%, transparent);
  --dk-dim:   color-mix(in srgb, var(--dk-fg) 40%, transparent);

  --dk-accent: var(--p, #2563eb);
  --dk-accent-fg: var(--pc, #ffffff);

  --dk-radius-sm: 6px;
  --dk-radius:    10px;
  --dk-radius-lg: 16px;

  --dk-font-body:    'Inter', system-ui, sans-serif;
  --dk-font-heading: var(--dk-font-body);
  --dk-font-mono:    'JetBrains Mono', ui-monospace, monospace;

  --dk-article-width: 72ch;
  --dk-sidebar-width: 240px;
}
```

Consumer usage (no more class overrides):

```css
.my-site .dk-root {
  --dk-accent: #f0a57c;
  --dk-font-heading: 'Instrument Serif', serif;
  --dk-radius-lg: 20px;
}
```

**Status.** Shipped — see `theme.css` at the crate root. Consumers import it
alongside `safelist.html`.

---

## Proposal 2 — Stable `dk-*` class names

**Problem.** Consumers writing CSS overrides today target DaisyUI classes
(`.prose`, `.navbar`, `.btn-ghost`) or undocumented internal classes — both
risk breaking on minor version bumps.

**Proposal.** Every structural node gets a stable, prefixed class in addition
to any styling classes:

| Node | Stable class |
|------|--------------|
| Root wrappers | `dk-root`, `dk-docs-root` |
| Layout | `dk-shell`, `dk-header`, `dk-sidebar`, `dk-main`, `dk-toc` |
| Article | `dk-article`, `dk-article-title`, `dk-article-body` |
| Sidebar | `dk-nav-group`, `dk-nav-group-title`, `dk-nav-item`, `dk-nav-item-active` |
| Search | `dk-search-trigger`, `dk-search-dialog`, `dk-search-input` |
| Page nav | `dk-pagination`, `dk-page-prev`, `dk-page-next` |

These names are semver-stable.

**Status.** Shipped — see components and the updated safelist.

---

## Proposal 3 — Slot props on `DocsLayout`

**Problem.** Today the `header:` slot exists (good!), but everything else is
rendered by the kit itself. Sites that want a custom footer, a secondary
announcement bar, a "last updated" badge next to article titles, or extra
sidebar sections have no way in.

**Proposal.** Expand to a consistent slot surface:

```rust
DocsLayout {
    header: rsx!{ ... },
    announcement_bar: rsx!{ ... },       // optional, above header
    sidebar_header: rsx!{ ... },         // above generated nav
    sidebar_footer: rsx!{ ... },         // e.g. "Edit this page" links
    article_footer: rsx!{ ... },         // "Was this helpful?" widget, etc.
    footer: rsx!{ ... },                 // site-wide footer
    Outlet::<Route> {}
}
```

All default to `None`. Consumers can target the `dk-article-footer-slot`,
`dk-sidebar-header-slot`, etc. classes that wrap each slot.

**Status.** Partially shipped. `announcement_bar`, `sidebar_header`,
`sidebar_footer`, and `footer` are live; per-article slots (below an article's
h1 or after an article's body) are a follow-up since they live in
`DocsPageContent`, not `DocsLayout`.

---

## Proposal 4 — Independent font customization for article headings

**Problem.** Consumers may want body text in Inter but h1/h2/h3 in a display
serif (common editorial pattern). Today it's all one font unless you fight
`.prose` internals.

**Proposal.** Two tokens as part of proposal 1:
- `--dk-font-body`
- `--dk-font-heading`

Apply `--dk-font-heading` to `.dk-article h1, h2, h3` and let consumers leave
it equal to body if they don't care.

**Status.** Shipped.

---

## Proposal 5 — `variant` / `density` prop on layouts

**Problem.** The default layout is optimized for long-form prose. Some sites
want a compact "reference-doc" feel (tighter line-height, smaller article
width, monospace numerals).

**Proposal.** An enum prop on the layouts:

```rust
pub enum DocsVariant {
    Prose,      // default — wide margins, 72ch width, serif-friendly
    Reference,  // tight — narrower, denser, mono-leaning
}
```

Sets internal CSS class `dk-variant-reference` and adjusts a few tokens
(`--dk-article-width`, body line-height, heading scale).

**Status.** Shipped. `DocsVariant::Prose` (default) and `DocsVariant::Reference`
are exposed as a `variant` prop on `DocsLayout`. The `Reference` variant
narrows `--dk-article-width` to `64ch` and tightens the type scale.

---

## Proposal 6 — Neutral defaults (don't hard-depend on DaisyUI)

**Problem.** The crate currently assumes DaisyUI tokens exist. Consumers
without DaisyUI see either unstyled content or pick up wrong colors.

**Proposal.** Ship the defaults in proposal 1 as the primary source of truth,
with DaisyUI tokens as optional fallbacks. Consumers without DaisyUI get a
sensible neutral theme; consumers with DaisyUI continue to inherit it
automatically.

Could optionally split into two features:
- `docs-kit` (neutral, no DaisyUI dependency)
- `docs-kit-daisyui` (adds the DaisyUI token bridge)

**Status.** Partial — `theme.css` declares `--dk-*` with DaisyUI fallbacks, so
consumers without DaisyUI get sensible neutral values. Full removal of
DaisyUI `bg-base-*` / `text-primary` usage throughout the components is the
remaining work.

---

## Proposal 7 — Official "theming" example in the repo

**Problem.** There's no reference for "how do I make this look like *my* brand"
short of reading source.

**Proposal.** Add `examples/custom-theme/` showing the same docs site with
three visual identities (default, warm editorial, brutalist light). Each
example ships one CSS file using only the public variables from proposal 1.

**Status.** Shipped as `examples/themes/` — ships `default.css`,
`warm-editorial.css`, and `brutalist-light.css`, each built entirely on top of
the `--dk-*` tokens with no Tailwind/DaisyUI class overrides.

---

## Good-first-issue candidates

- Audit internal CSS for hard-coded colors still going through DaisyUI classes
  (`bg-base-100`, `text-primary`, etc.) and migrate to `var(--dk-*)` fallbacks
  so consumers without DaisyUI get the theme file's defaults (Proposal 6)
- Add `dk-*` class names to the remaining blog components (currently only
  docs-side components are tagged)
- Add a screenshot gallery to `examples/themes/` so README readers can see
  each variant without cloning
