import { type RouteConfig, index, route, layout } from "@react-router/dev/routes"

export default [
  index("routes/home.tsx"),
  route("auth", "routes/auth.tsx"),
  route("login", "routes/login.tsx"),
  route("auth/hackclub", "routes/auth/hackclub-callback.tsx"),
  route("team", "routes/team.tsx"),

  layout("layouts/sidebar-layout.tsx", [
    route("dash", "routes/dash.tsx"),
    route("dash/firewall", "routes/dash/firewall/page.tsx"),
    route("dash/traffic", "routes/dash/traffic.tsx"),
    route("dash/settings", "routes/dash/settings.tsx"),
    route("dash/tunnel", "routes/dash/tunnel.tsx"),
    route("dash/workers", "routes/dash/workers.tsx"),
    route("dash/logs", "routes/dash/logs.tsx"),
    route("dash/performance", "routes/dash/performance.tsx"),
    route("dash/profile", "routes/dash/profile.tsx"),
    route("dash/admin", "routes/dash/admin/admin.tsx"),
    route("dash/dns", "routes/dash/domains/$domain/dns/page.tsx"),

    route("dash/domains", "routes/dash/domains/$domain/page.tsx"),
    route("dash/domains/:domain/dns", "routes/dash/domains/$domain/dns/page.tsx"),
    route("dash/domains/:domain/ssl", "routes/dash/domains/$domain/ssl/page.tsx"),
    route("dash/domains/:domain/redirects", "routes/dash/domains/$domain/redirects/page.tsx"),
  ]),
] satisfies RouteConfig