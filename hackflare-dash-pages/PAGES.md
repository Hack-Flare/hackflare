# Dashboard pages: build spec for Claude Code

Five more pages for the HackFlare dashboard. Each has a design reference in `reference/` (`.html` + `.png`). The HTML is a **design reference only** (fixed 1440 px, inline styles, example data). Rebuild each page with the app's existing templates, the shared components you already made for `/dash`, and `brand/tokens/tokens.css`, following `.claude/skills/hackflare-brand/SKILL.md`.

Suggested routes are in brackets. Use the app's existing routes if they already exist.

## Shared

- All dashboard pages (not sign in) use the **same dark top bar** as `/dash`: mark, Overview / Domains / API tokens / Docs, ⌘K search, avatar. Highlight the current section. The avatar opens Account.
- Replace all example data with real data. Keep `[bracket]` placeholders visible until the real value exists (API URL, nameservers, support channel, Hack Club handle).
- Responsive down to 390 px: side panels and sub-navs stack above/below the main content.
- Every button and link must do something real, or be left out. No dead links.

## 1. API tokens (`/dash/tokens`), `api-tokens.png`

- List of tokens: name, last 4 characters, domains it can use, access, created, last used, Revoke (with a confirm step).
- "Last used" older than ~60 days is shown in Char orange so stale tokens stand out.
- After creating a token, show the dark banner with the full token **once**, with a Copy button. Never show it again after the page reloads. Only store a hash of the token.
- "New token" panel: name, domains (all / some), access (read only / read & write), expiry.
- "Try it" box with a real example request.

## 2. Account (`/dash/account`), `account.png`

- Left sub-nav jumps to the sections on the same page.
- **Profile:** avatar, display name, email, Save.
- **Sign-in methods:** Hack Club (connected/disconnect) and email. Don't allow removing the last sign-in method.
- **Sessions:** list signed-in devices, sign out one, sign out everywhere else.
- **Delete account:** the button opens a confirm dialog where you type the account email to confirm. Explain that DNS for all domains stops working.

## 3. Add a domain (`/dash/domains/new`), `add-domain.png`

A 3-step flow. The reference shows step 2.
1. **Your domain:** input with validation (real domain, not already added).
2. **Switch nameservers:** the domain, the real nameservers as big copy buttons, registrar guide links, and a status box ("Not switched yet" + "Check now"). Keep checking in the background and move to step 3 automatically when it works.
3. **Verify:** success state with a link to add the first records.

The user can leave and come back. The domain shows as "Waiting for nameservers" on `/dash`, and the "Needs attention" card links back here.

## 4. Sign in (`/signin`), `sign-in.png`

- Dark brand panel on the left (hidden on mobile), form on the right.
- "Continue with Hack Club" + email. Signing in creates the account if it doesn't exist.
- Keep the existing auth backend. Only change the page.

## 5. Coming soon (`/dash/soon/{tool}`), `coming-soon.png`

- One shared page for tools that aren't built yet. The tool name comes from the URL.
- "Notify me" and "Help build it" (GitHub repo). "Back to overview".

## Check with me before building (may not exist in the backend yet)

- Token domain scopes, read-only access and expiry
- Sessions list and signing out other devices
- Registrar guides (which ones and where they live)
- "Notify me" on the coming soon page (needs somewhere to store the email)

## When you're done

Screenshot each page at 1440 and 390 px wide, compare with `reference/*.png`, and tell me what's different or left out.
