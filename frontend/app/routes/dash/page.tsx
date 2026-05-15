import {
  Globe,
  CheckCircle,
  AlertTriangle,
  TrendingUp,
  ArrowRight,
} from "lucide-react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"

const recentActivity = [
  {
    id: 1,
    time: "14:32",
    action: "Domain added",
    detail: "example.com",
    type: "domain",
  },
  {
    id: 2,
    time: "12:15",
    action: "DNS record updated",
    detail: "A record for api.example.com",
    type: "dns",
  },
  {
    id: 3,
    time: "10:48",
    action: "Firewall rule created",
    detail: "Blocked 128 requests",
    type: "firewall",
  },
  {
    id: 4,
    time: "09:22",
    action: "Certificate renewed",
    detail: "example.com SSL/TLS",
    type: "ssl",
  },
  {
    id: 5,
    time: "08:10",
    action: "Worker deployed",
    detail: "Request logger v2.1",
    type: "worker",
  },
]

const shortcuts = [
  {
    title: "Add Domain",
    description: "Register new domain",
    href: "/dash/domains",
    icon: Globe,
  },
  {
    title: "DNS Records",
    description: "Manage DNS entries",
    href: "/dash/dns",
    icon: TrendingUp,
  },
  {
    title: "Traffic Analytics",
    description: "View traffic stats",
    href: "/dash/traffic",
    icon: TrendingUp,
  },
  {
    title: "Security",
    description: "Firewall rules",
    href: "/dash/firewall",
    icon: AlertTriangle,
  },
]

export default function Dashboard() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold dark:text-white">Dashboard</h1>
        <p className="mt-2 text-zinc-600 dark:text-zinc-400">
          Quick snapshot of domains, DNS + account health
        </p>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <Globe className="h-4 w-4" />
              Active Domains
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-3xl font-bold">12</p>
            <p className="mt-1 text-xs text-green-600">+1 this month</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <CheckCircle className="h-4 w-4" />
              DNS Changes
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-3xl font-bold">4</p>
            <p className="mt-1 text-xs text-zinc-600 dark:text-zinc-400">
              Today
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <TrendingUp className="h-4 w-4" />
              Service Status
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-3xl font-bold text-green-600">Healthy</p>
            <p className="mt-1 text-xs text-green-600">99.9% uptime</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <AlertTriangle className="h-4 w-4" />
              Issues
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-3xl font-bold">0</p>
            <p className="mt-1 text-xs text-green-600">No alerts</p>
          </CardContent>
        </Card>
      </div>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-3">
        <div className="lg:col-span-2">
          <Card>
            <CardHeader>
              <CardTitle>Recent Activity</CardTitle>
              <CardDescription>Latest changes + events</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {recentActivity.map((item) => (
                  <div
                    key={item.id}
                    className="flex items-center gap-4 rounded-lg border border-zinc-200 p-3 hover:bg-zinc-50 dark:border-zinc-800 dark:hover:bg-zinc-800/50"
                  >
                    <div className="min-w-12 text-xs font-medium text-zinc-600 dark:text-zinc-400">
                      {item.time}
                    </div>
                    <div className="min-w-0 flex-1">
                      <p className="text-sm font-medium">{item.action}</p>
                      <p className="truncate text-xs text-zinc-600 dark:text-zinc-400">
                        {item.detail}
                      </p>
                    </div>
                    <span className="rounded bg-zinc-100 px-2 py-1 text-xs font-medium whitespace-nowrap text-zinc-800 dark:bg-zinc-800 dark:text-zinc-300">
                      {item.type}
                    </span>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </div>

        <Card>
          <CardHeader>
            <CardTitle>Quick Links</CardTitle>
            <CardDescription>Navigate fast</CardDescription>
          </CardHeader>
          <CardContent className="space-y-2">
            {shortcuts.map((shortcut, i) => (
              <a
                key={i}
                href={shortcut.href}
                className="group flex items-center justify-between rounded-lg border border-zinc-200 p-3 transition-colors hover:border-orange-500 hover:bg-orange-50 dark:border-zinc-800 dark:hover:border-orange-500 dark:hover:bg-orange-950/20"
              >
                <div className="min-w-0">
                  <p className="text-sm font-medium group-hover:text-orange-600 dark:group-hover:text-orange-400">
                    {shortcut.title}
                  </p>
                  <p className="text-xs text-zinc-600 dark:text-zinc-400">
                    {shortcut.description}
                  </p>
                </div>
                <ArrowRight className="ml-2 h-4 w-4 shrink-0 text-zinc-400 group-hover:text-orange-600 dark:group-hover:text-orange-400" />
              </a>
            ))}
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
