# Theme examples

Three drop-in visual identities for `dioxus-docs-kit`, each expressed purely
through the public `--dk-*` custom properties defined in `theme.css`. Pick one,
copy it into your project, and import it after `theme.css`.

```css
@import "tailwindcss";
@import "dioxus-docs-kit/theme.css";
@import "./warm-editorial.css";   /* <- choose one */
```

## Variants

| File | Style |
|------|-------|
| `default.css` | Baseline; nothing overridden. Here as a reference. |
| `warm-editorial.css` | Amber accent, serif display headings, softer borders. Good for content-heavy sites. |
| `brutalist-light.css` | High contrast, zero-radius, monospace headings. Good for dev-tool docs. |

Each file targets only the `dk-*` tokens. None of them touches a Tailwind or
DaisyUI class directly — that's the whole point of the theming surface. If you
can't express what you want through the tokens alone, that's a sign the kit
needs another token (open an issue).
