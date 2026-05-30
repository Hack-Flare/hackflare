import { useEffect, useState } from "react"
import {
  Globe,
  CheckCircle,
  AlertTriangle,
  TrendingUp,
  ArrowRight,
  Loader2,
} from "lucide-react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"
import { api, type DnsZone, type HealthResponse } from "~/lib/api"

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
  const [zones, setZones] = useState<DnsZone[]>([])
  const [health, setHealth] = useState<HealthResponse | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const load = async () => {
      try {
        const [zonesData, healthData] = await Promise.all([
          api.dns.listZones(),
          api.health.check(),
        ])
        setZones(zonesData)
        setHealth(healthData)
      } catch {
        // silently fail — dashboard shows defaults
      } finally {
        setLoading(false)
      }
    }
    void load()
  }, [])

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
            {loading ? (
              <Loader2 className="h-5 w-5 animate-spin text-zinc-500" />
            ) : (
              <>
                <p className="text-3xl font-bold">{zones.length}</p>
                <p className="mt-1 text-xs text-green-600">
                  {zones.length === 0
                    ? "No domains yet"
                    : `${zones.length} total`}
                </p>
              </>
            )}
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
            <p className="text-3xl font-bold">—</p>
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
            {loading ? (
              <Loader2 className="h-5 w-5 animate-spin text-zinc-500" />
            ) : (
              <>
                <p
                  className={`text-3xl font-bold ${
                    health?.status === "ok"
                      ? "text-green-600"
                      : "text-orange-500"
                  }`}
                >
                  {health?.status === "ok" ? "Healthy" : "Degraded"}
                </p>
                <p className="mt-1 text-xs text-zinc-600 dark:text-zinc-400">
                  DB: {health?.database ?? "unknown"}
                </p>
              </>
            )}
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
              <p className="py-8 text-center text-sm text-zinc-500">
                Activity feed coming soon
              </p>
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
