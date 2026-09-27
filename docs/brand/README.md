# Hackflare brand kit

Everything you need to use the Hackflare brand in code. The full visual guidelines are on the design canvas: https://claude.ai/artifact/BtnK5np6WLDd7DP1txZXS7

## What's in here

```
brand/
  logos/
    logo-primary*.svg     primary logo, [fire h]ackflare
    logo-secondary*.svg   mark + full name
    mark*.svg             the fire h on its own
    mark-small*.svg       mark for under 32 px (no ember)
    app-icon.svg          ink rounded square + mark
    favicon.svg           bigger mark, no ember
    png/                  app icons (512, 192, 180), favicons (32, 16), logo PNGs
  tokens/
    tokens.css            CSS variables (--hf-flame, ...) + base styles + buttons
    tailwind-v4-theme.css @theme block for Tailwind v4
    tailwind-v3-preset.js preset for Tailwind v3
    tokens.json           same values as JSON, with usage notes
.claude/skills/hackflare-brand/SKILL.md   rules for Claude Code and other agents
CLAUDE.md                                 short pointer so every session knows
```

Variant suffixes: none = light backgrounds, `-reversed` = dark, `-on-flame` = orange, `-mono-ink` / `-mono-paper` = one color.

## Setup

1. Copy `brand/` and `.claude/` into the root of your repo.
2. If you already have a `CLAUDE.md`, paste the "Brand" section from this one into yours instead of replacing it.
3. Hook up the tokens:
   - Plain CSS: `@import "./brand/tokens/tokens.css";`
   - Tailwind v4: after `@import "tailwindcss";` add `@import "./brand/tokens/tailwind-v4-theme.css";`
   - Tailwind v3: `presets: [require('./brand/tokens/tailwind-v3-preset.js')]`
4. Favicon, in your `<head>`:

```html
<link rel="icon" href="/brand/logos/favicon.svg" type="image/svg+xml">
<link rel="icon" href="/brand/logos/png/favicon-32.png" sizes="32x32">
<link rel="apple-touch-icon" href="/brand/logos/png/app-icon-180.png">
```

For frameworks that serve static files from `public/` (Astro, Next, Vite), put `logos/` inside `public/brand/` so the paths above work.
