import { type RouteConfig, index, route, layout } from "@react-router/dev/routes"

export default [
  index("routes/home.tsx"),
  route("auth", "routes/auth.tsx"),
  route("login", "routes/login.tsx"),
  route("auth/hackclub", "routes/auth/hackclub-callback.tsx"),
  route("team", "routes/team.tsx"),

  layout("layouts/sidebar-layout.tsx", [
    route("dash", "routes/dash.tsx"),
    route("dash/dns", "routes/dash/dns.tsx"),
    route("dash/firewall", "routes/dash/firewall.tsx"),
    route("dash/traffic", "routes/dash/traffic.tsx"),
    route("dash/settings", "routes/dash/settings.tsx"),
    route("dash/domains", "routes/dash/domains.tsx"),
    route("dash/tunnel", "routes/dash/tunnel.tsx"),
    route("dash/workers", "routes/dash/workers.tsx"),
    route("dash/logs", "routes/dash/logs.tsx"),
    route("dash/performance", "routes/dash/performance.tsx"),
    route("dash/profile", "routes/dash/profile.tsx"),
    route("dash/admin", "routes/dash/admin.tsx"),

    // add more pages here as you build them
  ]),
] satisfies RouteConfig