---
name: hackflare-brand
description: Hackflare brand rules (logo, colors, type, buttons). Use for any UI, page, component, email, image, social or marketing work for Hackflare, and whenever the logo, favicon or app icon is placed or changed.
---

# Hackflare brand

Assets live in `brand/`. Always use them. Never redraw, retype or approximate the logo.

## Logo

The logo is a bold lowercase **h** (Instrument Sans Bold, outlined) with an angular **flame** on top of its stem. The flame's base is exactly as wide as the stem.

| Asset | File | Use it for |
|---|---|---|
| Primary logo | `brand/logos/logo-primary.svg` | Default everywhere: headers, footers, docs, social banners. The fire h starts the word: [fire h]ackflare |
| Secondary lockup | `brand/logos/logo-secondary.svg` | Mark + full name ("h hackflare"). Only when the mark must also be seen on its own, e.g. next to an app icon |
| Mark | `brand/logos/mark.svg` | Square or tiny spaces: avatars, app icon, loading states |
| Mark, small | `brand/logos/mark-small.svg` | Mark under 32 px tall (no yellow ember) |
| App icon | `brand/logos/app-icon.svg`, `png/app-icon-*.png` | App stores, PWA manifest, apple-touch-icon (180) |
| Favicon | `brand/logos/favicon.svg`, `png/favicon-*.png` | Browser tab |

Pick the variant by background:

- Paper / light background: `*.svg` (default)
- Ink / dark background: `*-reversed.svg`
- Flame / orange background: `*-on-flame.svg` (all ink, no ember)
- One color jobs (print, stamps, embossing): `*-mono-ink.svg` or `*-mono-paper.svg`
- Photos or mid-tone colors: `*-mono-paper.svg`

### Sizing and space

- Clear space on every side = the flame's height (about 25% of the logo's height). Nothing inside it.
- Minimum height: mark 16 px, primary and secondary logos 24 px.
- Below 32 px tall, use the no-ember versions (`mark-small*.svg`, or drop the ember).
- Scale with `height` only and keep the aspect ratio (primary 850:216, secondary 966:216, mark 120:216).

### Never

- Stretch, squash, rotate, tilt, outline, add shadows or effects to the logo.
- Change the flame's colors (only Flame + Ember, or one-color versions).
- Move, resize or detach the flame from the stem.
- Retype "hackflare" in a font, or set the h in another font. Use the SVGs.
- Put the full-color logo on a Flame orange background (the flame disappears). Use `*-on-flame.svg`.
- Recreate the logo in CSS, emoji or icon fonts.

HTML example:

```html
<a href="/" aria-label="Hackflare home">
  <img src="/brand/logos/logo-primary.svg" alt="Hackflare" height="32">
</a>
```

## Color

Tokens: `brand/tokens/tokens.css` (CSS variables `--hf-*`), `tailwind-v4-theme.css`, `tailwind-v3-preset.js`, `tokens.json`.

| Name | Hex | Role |
|---|---|---|
| Flame | #F2611D | Logo flame, primary buttons, highlights |
| Ember | #FFC24A | Inside of the flame, tiny accents on Ink only |
| Ink | #16140F | Text, dark backgrounds (never pure #000) |
| Paper | #FAF9F5 | Main background (never pure #FFF) |
| Char | #B8400C | Orange text and links on light backgrounds |
| Smoke | #57534A | Secondary text, captions |
| Ash | #ECEBE6 | Cards, panels, dividers |

Rough amounts on a page: Paper 55%, Ink 25%, Ash 12%, Flame 6%, Ember 2%. Flame is the spark, not the background.

Contrast rules (WCAG):

- Text on Paper: Ink, Smoke or Char. Never Flame for body text (3.1:1) and never Ember.
- Text on Flame: always Ink (5.7:1). **Never white text on Flame.**
- Flame text on Ink is fine (5.7:1).
- Don't invent new colors or tints. If one is truly needed, derive it from these and check contrast >= 4.5:1 for text.

## Typography

- **Instrument Sans** for everything (400, 500, 600, 700). Load from Google Fonts (see `tokens.css`).
- **JetBrains Mono** only for code, terminal output and numbers that must line up.
- Don't use Inter, Roboto, Arial or other sans fonts as the main font.

| Style | Size / line height | Weight | Tracking |
|---|---|---|---|
| Display | 64 / 1.0 | 700 | -0.03em |
| H1 | 44 / 1.1 | 700 | -0.027em |
| H2 | 30 / 1.2 | 600 | -0.02em |
| H3 | 22 / 1.3 | 600 | 0 |
| Body | 17 / 1.55 | 400 | 0 |
| Label | 13 / 1.4 | 600, uppercase | 0.115em, color Char |
| Code | 15 / 1.6 | Mono 400 | 0 |

Keep body lines around 60 to 75 characters.

## Components

- Primary button: Flame background, Ink text, weight 600, radius 10 px, min height 44 px.
- Secondary button: Ink background, Paper text.
- Outline button: 2 px Ink border, Ink text, transparent.
- Links: Char, underline on hover.
- Radius: buttons 10 px, cards 16 px, big panels 20 px.
- Cards and panels: Ash on Paper, no heavy shadows.
- Icons: simple stroke icons, never emoji.

## Before you finish UI work

1. Logo is an `<img>`/inline copy of a file in `brand/logos/`, right variant for the background, at least minimum size, with clear space.
2. Only token colors are used; no white on Flame; body text is Ink/Smoke.
3. Fonts are Instrument Sans (+ JetBrains Mono for code).
4. Buttons and radii match the component rules.
