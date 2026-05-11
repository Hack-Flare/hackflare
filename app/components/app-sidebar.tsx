import { useLocation, NavLink, useNavigate } from "react-router"
import { useState } from "react"
import { useAuth } from "~/lib/auth-context"

import {
  LayoutDashboard, Globe, ShieldAlert, ArrowLeftRight,
  Zap, Network, BarChart2, Activity, ScrollText,
  Settings, MessageSquare, ChevronsUpDown, LogOut,
  UserCircle, Plus, Check, ChevronRight, Shield,
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

import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/avatar"
import { SlackIcon } from "./icons/slack"

// ── Workspaces ───────────────────────────────────────────────────────────────

const workspaces = [
  { id: "1", name: "My Projects",  plan: "Free tier", icon: "⚡" },
  { id: "2", name: "Hack Club HQ", plan: "Team",      icon: "🏠" },
  { id: "3", name: "Café Nerd",    plan: "Free tier", icon: "☕" },
]

// ── Nav items ────────────────────────────────────────────────────────────────

const overviewItems = [
  { title: "Dashboard", icon: LayoutDashboard, href: "/dash" },
]

const domainsItems = [
  { title: "All Domains",  href: "/dash/domains" },
  { title: "DNS Records",  href: "/dash/domains/dns" },
  { title: "SSL/TLS",      href: "/dash/domains/ssl" },
  { title: "Redirects",    href: "/dash/domains/redirects" },
]

const edgeItems = [
  { title: "Firewall", icon: ShieldAlert,    href: "/dash/firewall", badge: "2", badgeWarn: false },
  { title: "DNS",      icon: ArrowLeftRight, href: "/dash/dns" },
  { title: "Workers",  icon: Zap,            href: "/dash/workers" },
  { title: "Tunnel",   icon: Network,        href: "/dash/tunnel" },
]

const analyticsItems = [
  { title: "Traffic",     icon: BarChart2,  href: "/dash/traffic" },
  { title: "Performance", icon: Activity,   href: "/dash/performance" },
  { title: "Logs",        icon: ScrollText, href: "/dash/logs" },
]

const adminItems = [
  { title: "Admin Panel", icon: Shield, href: "/dash/admin" },
]

// ── Component ────────────────────────────────────────────────────────────────

export function AppSidebar() {
  const location = useLocation()
  const navigate = useNavigate()
  const { user, logout } = useAuth()
  const [activeWorkspace, setActiveWorkspace] = useState(workspaces[0])
  const [domainsExpanded, setDomainsExpanded] = useState(false)

  const userInitials = user?.name
    ? user.name.split(" ").map(n => n[0]).join("").toUpperCase().substring(0, 2)
    : "?"

  const handleLogout = () => {
    logout()
    navigate("/auth")
  }

  const isActive = (href: string) => {
    if (href === "/dash") return location.pathname === "/dash"
    return location.pathname.startsWith(href)
  }

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
                  <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-orange-500 text-white text-base shrink-0">
                    {activeWorkspace.icon}
                  </div>
                  <div className="flex flex-col leading-none min-w-0">
                    <span className="font-semibold text-sm truncate">{activeWorkspace.name}</span>
                    <span className="text-xs text-muted-foreground truncate">{activeWorkspace.plan}</span>
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
                    <div className="flex h-6 w-6 items-center justify-center rounded-md border bg-background text-sm shrink-0">
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
                  <div className="flex h-6 w-6 items-center justify-center rounded-md border bg-background shrink-0">
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
        <SidebarGroup>
          <SidebarGroupLabel>Overview</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {overviewItems.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton asChild isActive={isActive(item.href)}>
                    <NavLink to={item.href} className="flex items-center gap-2">
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
                  className="flex items-center gap-2"
                >
                  <Globe className="h-4 w-4" />
                  <span>Domains</span>
                  <ChevronRight
                    className={`ml-auto h-4 w-4 transition-transform ${domainsExpanded ? "rotate-90" : ""}`}
                  />
                </SidebarMenuButton>

                {domainsExpanded && (
                  <SidebarMenu className="ml-2 border-l border-sidebar-border mt-1 pl-2">
                    {domainsItems.map((item) => (
                      <SidebarMenuItem key={item.title}>
                        <SidebarMenuButton
                          asChild
                          isActive={isActive(item.href)}
                          size="sm"
                        >
                          <NavLink to={item.href} className="flex items-center gap-2">
                            <span className="text-xs">{item.title}</span>
                          </NavLink>
                        </SidebarMenuButton>
                      </SidebarMenuItem>
                    ))}
                  </SidebarMenu>
                )}
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        <SidebarGroup>
          <SidebarGroupLabel>Edge</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {edgeItems.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton asChild isActive={isActive(item.href)}>
                    <NavLink to={item.href} className="flex items-center gap-2">
                      <item.icon className="h-4 w-4" />
                      <span>{item.title}</span>
                    </NavLink>
                  </SidebarMenuButton>
                  {item.badge && (
                    <SidebarMenuBadge
                      className={item.badgeWarn ? "bg-orange-100 text-orange-700 border border-orange-200" : ""}
                    >
                      {item.badge}
                    </SidebarMenuBadge>
                  )}
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        <SidebarGroup>
          <SidebarGroupLabel>Analytics</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {analyticsItems.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton asChild isActive={isActive(item.href)}>
                    <NavLink to={item.href} className="flex items-center gap-2">
                      <item.icon className="h-4 w-4" />
                      <span>{item.title}</span>
                    </NavLink>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        <SidebarGroup>
          <SidebarGroupLabel>Admin</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {adminItems.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton asChild isActive={isActive(item.href)}>
                    <NavLink to={item.href} className="flex items-center gap-2">
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
            <SidebarMenuButton asChild isActive={isActive("/settings")}>
              <NavLink to="/dash/settings" className="flex items-center gap-2">
                <Settings className="h-4 w-4" />
                <span>Settings</span>
              </NavLink>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton asChild>
              <a href="https://hackclub.slack.com" target="_blank" rel="noreferrer" className="flex items-center gap-2">
                <SlackIcon className="h-4 w-4" />
                <span>Hack Club Slack</span>
              </a>
            </SidebarMenuButton>
          </SidebarMenuItem>
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
                  <Avatar className="h-8 w-8 rounded-lg shrink-0">
                    <AvatarFallback className="rounded-lg bg-orange-500 text-white text-xs font-semibold">
                      {userInitials}
                    </AvatarFallback>
                  </Avatar>
                  <div className="flex flex-col leading-none min-w-0">
                    <span className="font-semibold text-sm truncate">{user?.name || "User"}</span>
                    <span className="text-xs text-muted-foreground truncate">{user?.email}</span>
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
                <DropdownMenuLabel className="font-normal p-2">
                  <p className="font-semibold text-sm">{user?.name || "User"}</p>
                  <p className="text-xs text-muted-foreground">{user?.email}</p>
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