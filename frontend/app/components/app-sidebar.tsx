import {
  useLocation,
  NavLink,
  useNavigate,
  useParams,
  Link,
} from "react-router"
import { useState } from "react"
import { getUserDisplayName, useAuth } from "~/lib/auth-context"

import {
  LayoutDashboard,
  Globe,
  ShieldAlert,
  Zap,
  Network,
  BarChart2,
  Activity,
  ScrollText,
  Settings,
  BookOpen,
  ChevronsUpDown,
  LogOut,
  UserCircle,
  Plus,
  Check,
  ChevronRight,
  Shield,
  BadgeQuestionMark,
  MessageCircleQuestionMark,
  ArrowLeftRight,
} from "lucide-react"

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarSeparator,
} from "~/components/ui/sidebar"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  DropdownMenuShortcut,
} from "~/components/ui/dropdown-menu"
import { Avatar, AvatarFallback } from "~/components/ui/avatar"
import { SlackIcon } from "./icons/slack"

// ── Workspaces ───────────────────────────────────────────────────────────────

const workspaces = [
  { id: "1", name: "My Projects", plan: "Free tier", icon: "⚡" },
  { id: "2", name: "Hack Club HQ", plan: "Team", icon: "🏠" },
  { id: "3", name: "Café Nerd", plan: "Free tier", icon: "☕" },
]

// ── Static nav ────────────────────────────────────────────────────────────────

const overviewItems = [
  { title: "Dashboard", icon: LayoutDashboard, href: "/dash" },
]

const edgeItems = [
  { title: "Firewall", icon: ShieldAlert, href: "/dash/firewall", badge: "2" },
  { title: "Workers", icon: Zap, href: "/dash/workers" },
  { title: "Tunnel", icon: Network, href: "/dash/tunnel" },
]

const analyticsItems = [
  { title: "Traffic", icon: BarChart2, href: "/dash/traffic" },
  { title: "Performance", icon: Activity, href: "/dash/performance" },
  { title: "Logs", icon: ScrollText, href: "/dash/logs" },
]

const adminItems = [{ title: "Admin Panel", icon: Shield, href: "/dash/admin" }]

// ── Component ────────────────────────────────────────────────────────────────

export function AppSidebar() {
  const location = useLocation()
  const navigate = useNavigate()
  const { domain } = useParams()
  const { user, logout } = useAuth()

  const [activeWorkspace, setActiveWorkspace] = useState(workspaces[0])
  const [domainsExpanded, setDomainsExpanded] = useState(
    location.pathname.startsWith("/dash/domains")
  )

  const userInitials = user?.name
    ? user.name
        .split(" ")
        .map((n: string) => n[0])
        .join("")
        .toUpperCase()
        .substring(0, 2)
    : user?.email
      ? user.email.split("@")[0].slice(0, 2).toUpperCase()
      : user?.id?.slice(0, 2).toUpperCase() || "?"

  const userLabel = getUserDisplayName(user)

  const handleLogout = async () => {
    await logout()
    navigate("/login")
  }

  const isActive = (href: string) => {
    if (href === "/dash") return location.pathname === "/dash"
    return location.pathname.startsWith(href)
  }

  // Dynamic per-domain subnav — only shown when inside a domain route
  const domainSubItems = domain
    ? [
        { title: "DNS Records", href: `/dash/domains/${domain}/dns` },
        { title: "SSL/TLS", href: `/dash/domains/${domain}/ssl` },
        { title: "Redirects", href: `/dash/domains/${domain}/redirects` },
      ]
    : []

  return (
    <Sidebar>
      {/* Workspace switcher */}
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <SidebarMenuButton
                  size="lg"
                  className="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
                >
                  <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-orange-500 text-base text-white">
                    {activeWorkspace.icon}
                  </div>
                  <div className="flex min-w-0 flex-col leading-none">
                    <span className="truncate text-sm font-semibold">
                      {activeWorkspace.name}
                    </span>
                    <span className="truncate text-xs text-muted-foreground">
                      {activeWorkspace.plan}
                    </span>
                  </div>
                  <ChevronsUpDown className="ml-auto h-4 w-4 shrink-0 text-muted-foreground" />
                </SidebarMenuButton>
              </DropdownMenuTrigger>
              <DropdownMenuContent
                className="w-[--radix-dropdown-menu-trigger-width] min-w-52 rounded-lg"
                align="start"
                side="bottom"
                sideOffset={4}
              >
                <DropdownMenuLabel className="text-xs text-muted-foreground">
                  Workspaces
                </DropdownMenuLabel>
                {workspaces.map((ws, i) => (
                  <DropdownMenuItem
                    key={ws.id}
                    onClick={() => setActiveWorkspace(ws)}
                    className="gap-2 p-2"
                  >
                    <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded-md border bg-background text-sm">
                      {ws.icon}
                    </div>
                    <span className="flex-1 truncate">{ws.name}</span>
                    {activeWorkspace.id === ws.id && (
                      <Check className="h-3.5 w-3.5 text-muted-foreground" />
                    )}
                    <DropdownMenuShortcut>⌘{i + 1}</DropdownMenuShortcut>
                  </DropdownMenuItem>
                ))}
                <DropdownMenuSeparator />
                <DropdownMenuItem className="gap-2 p-2">
                  <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded-md border bg-background">
                    <Plus className="h-4 w-4" />
                  </div>
                  <span className="text-muted-foreground">Add workspace</span>
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>

      {/* Nav */}
      <SidebarContent>
        {/* Overview */}
        <SidebarGroup>
          <SidebarGroupLabel>Overview</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {overviewItems.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton asChild isActive={isActive(item.href)}>
                    <NavLink
                      prefetch="intent"
                      to={item.href}
                      className="flex items-center gap-2"
                    >
                      <item.icon className="h-4 w-4" />
                      <span>{item.title}</span>
                    </NavLink>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}

              {/* Domains collapsible */}
              <SidebarMenuItem>
                <SidebarMenuButton
                  onClick={() => setDomainsExpanded(!domainsExpanded)}
                  isActive={isActive("/dash/domains") && !domainsExpanded}
                  className="flex items-center gap-2"
                >
                  <Globe className="h-4 w-4" />
                  <span>Domains</span>
                  <ChevronRight
                    className={`ml-auto h-4 w-4 transition-transform duration-200 ${domainsExpanded ? "rotate-90" : ""}`}
                  />
                </SidebarMenuButton>

                {domainsExpanded && (
                  <SidebarMenu className="mt-1 ml-2 border-l border-sidebar-border pl-2">
                    {/* Always show All Domains */}
                    <SidebarMenuItem>
                      <SidebarMenuButton
                        asChild
                        isActive={location.pathname === "/dash/domains"}
                        size="sm"
                      >
                        <NavLink
                          prefetch="intent"
                          to="/dash/domains"
                          className="flex items-center gap-2"
                        >
                          <span className="text-xs">All Domains</span>
                        </NavLink>
                      </SidebarMenuButton>
                    </SidebarMenuItem>

                    {/* Per-domain subnav — only when inside a domain */}
                    {domain && (
                      <>
                        <div className="px-2 pt-2 pb-1">
                          <p className="truncate text-xs font-medium text-muted-foreground">
                            {domain}
                          </p>
                        </div>
                        {domainSubItems.map((item) => (
                          <SidebarMenuItem key={item.title}>
                            <SidebarMenuButton
                              asChild
                              isActive={isActive(item.href)}
                              size="sm"
                            >
                              <NavLink
                                prefetch="intent"
                                to={item.href}
                                className="flex items-center gap-2"
                              >
                                <span className="text-xs">{item.title}</span>
                              </NavLink>
                            </SidebarMenuButton>
                          </SidebarMenuItem>
                        ))}
                      </>
                    )}
                  </SidebarMenu>
                )}
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        {/* Edge */}
        <SidebarGroup>
          <SidebarGroupLabel>Edge</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {edgeItems.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton asChild isActive={isActive(item.href)}>
                    <NavLink
                      prefetch="intent"
                      to={item.href}
                      className="flex items-center gap-2"
                    >
                      <item.icon className="h-4 w-4" />
                      <span>{item.title}</span>
                    </NavLink>
                  </SidebarMenuButton>
                  {item.badge && (
                    <SidebarMenuBadge>{item.badge}</SidebarMenuBadge>
                  )}
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        {/* Analytics */}
        <SidebarGroup>
          <SidebarGroupLabel>Analytics</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {analyticsItems.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton asChild isActive={isActive(item.href)}>
                    <NavLink
                      prefetch="intent"
                      to={item.href}
                      className="flex items-center gap-2"
                    >
                      <item.icon className="h-4 w-4" />
                      <span>{item.title}</span>
                    </NavLink>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        {/* Admin */}
        <SidebarGroup>
          <SidebarGroupLabel>Admin</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {adminItems.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton asChild isActive={isActive(item.href)}>
                    <NavLink
                      prefetch="intent"
                      to={item.href}
                      className="flex items-center gap-2"
                    >
                      <item.icon className="h-4 w-4" />
                      <span>{item.title}</span>
                    </NavLink>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      {/* Footer */}
      <SidebarFooter>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton asChild>
              <NavLink
                prefetch="intent"
                to="/dash/help"
                className="flex items-center gap-2"
              >
                <MessageCircleQuestionMark className="h-4 w-4" />
                <span>Help</span>
              </NavLink>
            </SidebarMenuButton>
          </SidebarMenuItem>
          {/*<SidebarMenuItem>
            <SidebarMenuButton asChild>
              <a
                href="https://hackclub.slack.com"
                target="_blank"
                rel="noreferrer"
                className="flex items-center gap-2"
              >
                <SlackIcon className="h-4 w-4" />
                <span>Hack Club Slack</span>
              </a>
            </SidebarMenuButton>
          </SidebarMenuItem>*/}
        </SidebarMenu>

        <SidebarSeparator />

        {/* User dropdown */}
        <SidebarMenu>
          <SidebarMenuItem>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <SidebarMenuButton
                  size="lg"
                  className="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
                >
                  <Avatar className="h-8 w-8 shrink-0 rounded-lg">
                    <AvatarFallback className="rounded-lg bg-orange-500 text-xs font-semibold text-white">
                      {userInitials}
                    </AvatarFallback>
                  </Avatar>
                  <div className="flex min-w-0 flex-col leading-none">
                    <span className="truncate text-sm font-semibold">
                      {userLabel}
                    </span>
                    <span className="truncate text-xs text-muted-foreground">
                      Hack Club session active
                    </span>
                  </div>
                  <ChevronsUpDown className="ml-auto h-4 w-4 shrink-0 text-muted-foreground" />
                </SidebarMenuButton>
              </DropdownMenuTrigger>
              <DropdownMenuContent
                className="w-[--radix-dropdown-menu-trigger-width] min-w-52 rounded-lg"
                side="top"
                align="start"
                sideOffset={4}
              >
                <DropdownMenuLabel className="p-2 font-normal">
                  <p className="text-sm font-semibold">{userLabel}</p>
                  <p className="text-xs text-muted-foreground">
                    Authenticated via Hack Club
                  </p>
                </DropdownMenuLabel>
                <DropdownMenuSeparator />
                <DropdownMenuItem onSelect={() => navigate("/dash/profile")}>
                  <UserCircle className="mr-2 h-4 w-4" />
                  Profile
                </DropdownMenuItem>
                <DropdownMenuItem onSelect={() => navigate("/dash/settings")}>
                  <Settings className="mr-2 h-4 w-4" />
                  Account settings
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem
                  className="text-destructive focus:text-destructive"
                  onSelect={handleLogout}
                >
                  <LogOut className="mr-2 h-4 w-4" />
                  Sign out
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  )
}
