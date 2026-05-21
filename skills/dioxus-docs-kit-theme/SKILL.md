---
name: dioxus-docs-kit-theme
description: >-
  Author a new theme preset for dioxus-docs-kit. Covers the public `--dk-*` token
  surface, the MANDATORY DaisyUI bridge (without which only fonts change, no colors),
  the single-mode convention (each preset locks data-theme to light OR dark), the
  file location under `crates/dioxus-docs-kit/examples/themes/`, and how to register
  the preset in the example app's picker. Use when: (1) Creating a new visual identity
  for a docs site built on dioxus-docs-kit, (2) Adding brand colors / typography to
  the docs surface, (3) Building a dark-mode counterpart to an existing light preset,
  (4) Debugging "my theme only changes fonts, no colors appear". Triggers:
  "new dioxus-docs-kit theme", "docs theme preset", "create theme preset",
  "brand the docs site", "customize docs colors", "theme only changes fonts".
---

# dioxus-docs-kit Theme Authoring

A theme preset is a single CSS file that overrides `--dk-*` custom properties on
`.dk-root` and bridges them into DaisyUI's color tokens. With those two layers,
the same component tree renders in any visual identity — magazine, brutalist,
brand colors — without touching component code.

This skill exists because two things about the theme system are non-obvious and
cost real time to discover by trial:

1. **Setting `--dk-*` tokens alone changes almost nothing visible** (just heading
   font and article width). You also have to bridge into DaisyUI's `--color-*`
   tokens or every background, button, and badge keeps its default look.
2. **Presets are single-mode by design.** A preset locks `data-theme` to either
   `light` or `dark`. If you need both, ship two presets.

## File layout

```
crates/dioxus-docs-kit/
├── theme.css                          # base --dk-* token defaults (don't edit per-preset)
└── examples/themes/
    ├── README.md                      # describes the shipped presets
    ├── default.css                    # baseline (overrides nothing)
    ├── warm-editorial.css             # light theme — amber + serif
    └── brutalist-light.css            # light theme — yellow + monospace
```

Add new presets as new files in `examples/themes/`. Don't edit `theme.css`
unless you're adding a new public token to the surface.

## Step 1: Pick a slug, display name, and mode

```
slug:         midnight-terminal           # kebab-case, becomes the filename + storage key
display:      "Midnight terminal"         # what appears in the picker dropdown
mode:         dark                        # one of "light" | "dark"
```

The slug must be valid as a CSS filename and as a `localStorage` value. Avoid
periods, spaces, and quotes.

## Step 2: Scaffold the CSS file

Create `crates/dioxus-docs-kit/examples/themes/<slug>.css`:

```css
/*
 * <slug>.css
 *
 * <one-paragraph identity statement — what does this theme look and feel like?
 * which sites would use it?>
 *
 * Drop-in: import after theme.css.
 */

.dk-root {
    /* -------- surface colors -------- */
    --dk-bg:      #0d0f12;          /* page background */
    --dk-bg-sub:  #14181d;          /* card / panel background */
    --dk-bg-alt:  #1c2127;          /* hover state / divider tint */
    --dk-border:  #2a3038;          /* borders */

    /* -------- foreground -------- */
    --dk-fg:      #c8d1dc;          /* primary body text */

    /* -------- accent -------- */
    --dk-accent:      #5cf2a3;      /* links, active states, primary buttons */
    --dk-accent-fg:   #062513;      /* text *on top of* an accent fill */
    --dk-accent-soft: color-mix(in srgb, #5cf2a3 14%, transparent);

    /* -------- radii -------- */
    --dk-radius-sm: 4px;
    --dk-radius:    8px;
    --dk-radius-lg: 14px;

    /* -------- typography -------- */
    --dk-font-body:    'Inter', system-ui, sans-serif;
    --dk-font-heading: 'Inter', system-ui, sans-serif;
    --dk-font-mono:    'JetBrains Mono', ui-monospace, monospace;

    /* -------- layout -------- */
    --dk-article-width: 68ch;

    /* -------- DAISYUI BRIDGE (mandatory) --------
       Without these, components keep their default DaisyUI colors and your
       preset only affects heading font + article width. Map each --dk-* token
       to the corresponding --color-* token from DaisyUI 5.                  */
    --color-base-100:        var(--dk-bg);
    --color-base-200:        var(--dk-bg-sub);
    --color-base-300:        var(--dk-bg-alt);
    --color-base-content:    var(--dk-fg);
    --color-primary:         var(--dk-accent);
    --color-primary-content: var(--dk-accent-fg);
}
```

### The `--dk-*` token surface

| Token | Drives |
|-------|--------|
| `--dk-bg` | page background, navbar, body |
| `--dk-bg-sub` | feature cards, sidebar surface, code blocks |
| `--dk-bg-alt` | hover states, dividers |
| `--dk-border` | borders (currently advisory — daisyUI handles most borders) |
| `--dk-fg` | primary body text |
| `--dk-muted` / `--dk-dim` | derived from `--dk-fg` via `color-mix`; don't override unless adding new tokens |
| `--dk-accent` | links, active sidebar item, tab underline, primary button fill |
| `--dk-accent-fg` | text rendered *on top of* an accent fill (button labels etc.) |
| `--dk-accent-soft` | softened accent for tinted backgrounds |
| `--dk-radius-sm` / `--dk-radius` / `--dk-radius-lg` | rounded corners across cards, badges, buttons |
| `--dk-font-body` / `--dk-font-heading` / `--dk-font-mono` | typography |
| `--dk-article-width` | docs article column width (ch units recommended) |

Full canonical list with fallbacks lives in `crates/dioxus-docs-kit/theme.css` —
read it before adding a new token to the surface.

## Step 3: Why the bridge matters

`theme.css` defines the `--dk-*` tokens but only *uses* them for:

- `.dk-article h1-h4, .dk-article-title` → `font-family: var(--dk-font-heading)`
- `.dk-root .dk-article` → `max-width: var(--dk-article-width)`

Every other visible color comes from DaisyUI's own `--color-*` tokens (referenced
by Tailwind utility classes like `bg-base-100`, `text-primary`, `bg-base-200`).
The bridge rewrites those DaisyUI tokens inside `.dk-root` to point at the
preset's `--dk-*` values, so every navbar/card/badge/button picks up the new
identity automatically.

**Skipping the bridge is the #1 cause of "my theme only changes the fonts."**

## Step 4: Mode-pin the preset

Presets are single-mode. You will declare which mode (`light` or `dark`) the
preset is built for and the example app's picker will force `data-theme` to
that mode whenever the preset is active.

The picker registration lives in `src/main.rs` of the example app. Find
`preset_mode()`:

```rust
fn preset_mode(name: &str) -> Option<&'static str> {
    match name {
        "warm-editorial" | "brutalist-light" => Some("light"),
        _ => None,
    }
}
```

Add your slug with its locked mode. If your preset is genuinely dark, return
`Some("dark")`.

## Step 5: Register in the picker

In the same `src/main.rs`, three places need a one-line addition:

**1. Embed the CSS file:**

```rust
const THEME_PRESET_MIDNIGHT: &str =
    include_str!("../crates/dioxus-docs-kit/examples/themes/midnight-terminal.css");
```

**2. Route the slug to the CSS in `preset_css()`:**

```rust
fn preset_css(name: &str) -> &'static str {
    match name {
        "warm-editorial" => THEME_PRESET_WARM,
        "brutalist-light" => THEME_PRESET_BRUTALIST,
        "midnight-terminal" => THEME_PRESET_MIDNIGHT,
        _ => THEME_PRESET_DEFAULT,
    }
}
```

**3. Add a dropdown entry in `ThemePresetPicker`:**

```rust
let options: [(&str, &str, Option<&str>); 4] = [
    ("default", "Default", None),
    ("warm-editorial", "Warm editorial", Some("light")),
    ("brutalist-light", "Brutalist light", Some("light")),
    ("midnight-terminal", "Midnight terminal", Some("dark")),
];
```

(Update the array length in the type annotation.)

## Step 6: Verify

```sh
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
dx serve
```

In the browser:
1. Open the dev URL, navigate to `/docs/getting-started/introduction`.
2. Click the **Preset** dropdown in the navbar; your theme should appear with its mode chip.
3. Pick it — the whole page should change colors (background, sidebar active state, tab underline, icons, card backgrounds). If only the fonts change, the bridge block in Step 2 is missing.
4. Confirm the `🔒 Light` (or `🔒 Dark`) badge appears in place of the sun/moon toggle while your preset is active.
5. Read sample paragraphs in the article and a callout — confirm contrast is comfortable, not just technically legible.
6. Switch back to **Default** — `data-theme` should restore from `localStorage['docs-theme']` and the toggle should return.

## Authoring guidance

### Colors

- Pick `--dk-bg` and `--dk-fg` first. Verify >= 4.5:1 contrast (WCAG AA body
  text) before going further. Everything else builds on this pair.
- `--dk-bg-sub` should sit *between* `--dk-bg` and `--dk-fg` by about 4–10%
  lightness for light themes, or 4–10% darker for dark themes. Too much
  contrast makes cards feel like buttons.
- `--dk-accent` is the most over-used token by new authors. Use it sparingly:
  active sidebar item, primary button, tab underline. Don't tint every icon.
- Use `color-mix(in srgb, var(--dk-accent) <N>%, transparent)` for soft
  variants instead of authoring a third accent color — keeps the palette
  honest.

### Typography

- If you override `--dk-font-heading` to a webfont, you're responsible for
  loading it (`<link>` in `index.html` or `@import` in the consuming site's
  CSS). The theme file doesn't ship fonts.
- Keep `--dk-font-mono` even if your theme isn't mono-flavored. Code blocks
  read it.
- `--dk-article-width` in `ch` is recommended over `rem` because it scales
  naturally with the body font. 64–80ch is the comfortable range.

### Radii

- Set all three radii consistently. A theme with `--dk-radius-sm: 0` and
  `--dk-radius-lg: 24px` will feel visually broken — badges look brutalist,
  cards look pillowy.
- `0` (brutalist), `4–6px` (utilitarian), `8–12px` (default), `16–24px`
  (editorial) are the sane bands.

## Common pitfalls

### "My theme only changes the fonts"

You forgot the DaisyUI bridge block (Step 2 → bottom of the CSS file).

### "Colors changed but text is unreadable in the navbar/TOC"

You're running a light preset under `data-theme="dark"` (or vice versa). Make
sure Step 4 mode-pinning is wired so the picker forces the right `data-theme`.

### "Headings render in the right font but body text doesn't"

Tailwind's `prose` plugin has its own typography tokens (`--tw-prose-body`,
`--tw-prose-headings`) that aren't bridged. The current scope of theme.css
covers headings only via `.dk-article h1-h4, .dk-article-title`. If you need
the body font to follow `--dk-font-body` everywhere, override `prose-body`
in your CSS file or add an explicit rule on `.dk-article-body, .dk-article p`.

### "Looks fine on Docs, broken on Home"

The home `<main>` in the example app must have the `dk-root` class for the
tokens to apply. Already done in `Navbar` (`class: "min-h-screen bg-base-100 dk-root"`).
If you build a custom landing page, remember to add `dk-root` to its outer wrapper.

### "Toggle still shows when preset is active"

The toggle is replaced by a `🔒 Light`/`🔒 Dark` badge via the `ThemeControl`
wrapper. If you bypass it and render `ThemeToggle` directly, the lock UI won't
fire. Always go through `ThemeControl { toggle: rsx! { ThemeToggle {} } }`.

## What NOT to do

- **Don't override DaisyUI's `--color-*` tokens directly without going through
  `--dk-*` first.** It works, but you lose the ability to introspect or
  re-skin via the public token surface — the whole point of the kit's theming
  layer.
- **Don't touch Tailwind utility classes inside component RSX.** If your theme
  needs a structural change (e.g., wider sidebar), open an issue to add a new
  `--dk-*` token rather than fork the components.
- **Don't try to make one preset cover both modes** (e.g., reading `prefers-color-scheme`
  inside the CSS file). The single-mode design exists so the picker + toggle
  story stays explainable. If you need both modes for one identity, ship two
  files (`amber-light.css` and `amber-dark.css`).
- **Don't store dynamic accent colors in JS.** The CSS file is the source of
  truth; the Rust side just `include_str!()`s it.

## Reference files

- `crates/dioxus-docs-kit/theme.css` — canonical token surface + base rules
- `crates/dioxus-docs-kit/examples/themes/` — shipped presets to copy from
- `crates/dioxus-docs-kit/THEMING.md` — design rationale and roadmap
- `src/main.rs` (example app) — picker, bridge effect, mode pinning
