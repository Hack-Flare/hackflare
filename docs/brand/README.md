# Hackflare brand kit

The canonical Hackflare brand assets and rules live in this folder.
The application serves vendored copies from `hackflare-app/static/brand/`, so change the files here first and then re-vendor them.
See [DESIGN.md](../DESIGN.md) for how the dashboard applies the brand.

## Layout

```
brand/
  logos/
    logo-primary*.svg     primary logo, [fire h]ackflare
    logo-secondary*.svg   mark plus full name
    mark*.svg             the fire h on its own
    mark-small*.svg       mark for under 32 px (no ember)
    app-icon.svg          ink rounded square plus mark
    favicon.svg           bigger mark, no ember
    png/                  app icons (512, 192, 180), favicons (32, 16), logo PNGs
  tokens/
    tokens.css            CSS variables (--hf-flame, ...) plus base styles
    tailwind-v4-theme.css @theme bridge for Tailwind v4
    tailwind-v3-preset.js preset for Tailwind v3
    tokens.json           the same values as JSON, with usage notes
```

Variant suffixes: none = light backgrounds, `-reversed` = dark, `-on-flame` = orange, `-mono-ink` / `-mono-paper` = one color.

## Rules

- Never retype "hackflare", redraw the fire h, or recreate the logo in CSS.
  Always use the logo files.
- Only use the colors defined in `tokens/`.
  Never put white text on Flame (`#F2611D`).
- Instrument Sans for UI text.
  Code uses the system monospace stack so pages do not download an additional code font.
- The product name in copy is "Hackflare", not "HackFlare".

## Using the tokens

- Plain CSS: `@import "./brand/tokens/tokens.css";`
- Tailwind v4: after `@import "tailwindcss";` add `@import "./brand/tokens/tailwind-v4-theme.css";`
- Tailwind v3: `presets: [require('./brand/tokens/tailwind-v3-preset.js')]`

The application itself uses plain CSS, not Tailwind.
The Tailwind files exist for other projects that adopt the brand.

This folder does not load any webfonts.
Instrument Sans is self-hosted by the application; when you reuse the kit elsewhere, load Instrument Sans yourself (self-hosted or from a font CDN).

## How the application serves the brand

Vendored copies live under `hackflare-app/static/brand/` and templates reference them as `/static/brand/...`:

- `logos/` mirrors `docs/brand/logos/`.
- `tokens/tokens.css` mirrors `docs/brand/tokens/tokens.css` plus one addition: an `@font-face` for `static/fonts/instrument-sans-latin.woff2`.

Favicon and app icon, as used in `hackflare-app/templates/base.html`:

```html
<link rel="icon" href="/static/brand/logos/favicon.svg" type="image/svg+xml" />
<link rel="icon" href="/static/brand/logos/png/favicon-32.png" sizes="32x32" />
<link rel="apple-touch-icon" href="/static/brand/logos/png/app-icon-180.png" />
```

After changing a logo or token, copy the updated file into `hackflare-app/static/brand/`.
Static files are read from disk, so no rebuild is needed for them to appear.
