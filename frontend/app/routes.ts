import {
  type RouteConfig,
  index,
  route,
  layout,
} from "@react-router/dev/routes"

export default [
  index("routes/home.tsx"),
  route("auth", "routes/auth.tsx"),
  route("login", "routes/login.tsx"),
  route("auth/hackclub", "routes/auth/hackclub-callback.tsx"),
  route("ourteam", "routes/ourteam.tsx"),

  layout("layouts/sidebar-layout.tsx", [
    route("dash", "routes/dash/page.tsx"),
    route("dash/firewall", "routes/dash/firewall/page.tsx"),
    route("dash/traffic", "routes/dash/traffic/page.tsx"),
    route("dash/settings", "routes/dash/settings/page.tsx"),
    route("dash/tunnel", "routes/dash/tunnel/page.tsx"),
    route("dash/workers", "routes/dash/workers/page.tsx"),
    route("dash/logs", "routes/dash/logs/page.tsx"),
    route("dash/performance", "routes/dash/performance/page.tsx"),
    route("dash/profile", "routes/dash/profile/page.tsx"),
    route("dash/admin", "routes/dash/admin/page.tsx"),
    route("dash/help","routes/dash/help/page.tsx"),

    route("dash/domains", "routes/dash/domains/page.tsx"),
    route(
      "dash/domains/:domain/dns",
      "routes/dash/domains/$domain/dns/page.tsx"
    ),
    route(
      "dash/domains/:domain/ssl",
      "routes/dash/domains/$domain/ssl/page.tsx"
    ),
    route(
      "dash/domains/:domain/redirects",
      "routes/dash/domains/$domain/redirects/page.tsx"
    ),
  ]),
] satisfies RouteConfig
