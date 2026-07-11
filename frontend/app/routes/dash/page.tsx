import { useEffect, useState } from "react"
import {
  Activity,
  AlertTriangle,
  ArrowRight,
  BarChart2,
  CheckCircle2,
  Clock,
  Globe,
  Loader2,
  Shield,
  XCircle,
} from "lucide-react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"
import {
  api,
  type DnsZone,
  type QueryLogEntry,
  type QueryLogsResponse,
} from "~/lib/api"

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
    icon: Globe,
  },
  {
    title: "Traffic Analytics",
    description: "View traffic stats",
    href: "/dash/traffic",
    icon: BarChart2,
  },
  {
    title: "Security",
    description: "Firewall rules",
    href: "/dash/firewall",
    icon: Shield,
  },
]

function formatTime(ts: string) {
  const d = new Date(ts)
  const now = new Date()
  const diffMs = now.getTime() - d.getTime()
  const diffMin = Math.floor(diffMs / 60000)
  if (diffMin < 1) return "just now"
  if (diffMin < 60) return `${diffMin}m ago`
  const diffHr = Math.floor(diffMin / 60)
  if (diffHr < 24) return `${diffHr}h ago`
  return d.toLocaleDateString("en-US", { month: "short", day: "numeric" })
}

export default function Dashboard() {
  const [zones, setZones] = useState<DnsZone[]>([])
  const [logs, setLogs] = useState<QueryLogEntry[]>([])
  const [logsSummary, setLogsSummary] = useState<QueryLogsResponse["summary"] | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const load = async () => {
      try {
        const [zonesData, logsData] = await Promise.all([
          api.dns.listZones(),
          api.logs.queryLogs(),
        ])
        setZones(zonesData)
        setLogs(logsData.logs ?? [])
        setLogsSummary(logsData.summary)
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
          Quick snapshot of your domains and recent activity
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
              <Activity className="h-4 w-4" />
              Queries Today
            </CardTitle>
          </CardHeader>
          <CardContent>
            {loading ? (
              <Loader2 className="h-5 w-5 animate-spin text-zinc-500" />
            ) : (
              <>
                <p className="text-3xl font-bold">
                  {logsSummary
                    ? (logsSummary.info_today +
                        logsSummary.warnings_today +
                        logsSummary.errors_today).toLocaleString()
                    : 0}
                </p>
                <p className="mt-1 text-xs text-zinc-600 dark:text-zinc-400">
                  Total DNS queries today
                </p>
              </>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <XCircle className="h-4 w-4" />
              Today's Errors
            </CardTitle>
          </CardHeader>
          <CardContent>
            {loading ? (
              <Loader2 className="h-5 w-5 animate-spin text-zinc-500" />
            ) : (
              <>
                <p
                  className={`text-3xl font-bold ${
                    (logsSummary?.errors_today ?? 0) > 0
                      ? "text-red-500"
                      : ""
                  }`}
                >
                  {logsSummary?.errors_today ?? 0}
                </p>
                <p className="mt-1 text-xs text-zinc-600 dark:text-zinc-400">
                  Failed queries today
                </p>
              </>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <AlertTriangle className="h-4 w-4" />
              Warnings
            </CardTitle>
          </CardHeader>
          <CardContent>
            {loading ? (
              <Loader2 className="h-5 w-5 animate-spin text-zinc-500" />
            ) : (
              <>
                <p
                  className={`text-3xl font-bold ${
                    (logsSummary?.warnings_today ?? 0) > 0
                      ? "text-amber-500"
                      : ""
                  }`}
                >
                  {logsSummary?.warnings_today ?? 0}
                </p>
                <p className="mt-1 text-xs text-zinc-600 dark:text-zinc-400">
                  {logsSummary && logsSummary.warnings_today > 0
                    ? "Requires attention"
                    : "No warnings"}
                </p>
              </>
            )}
          </CardContent>
        </Card>
      </div>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-3">
        <div className="lg:col-span-2">
          <Card>
            <CardHeader>
              <CardTitle>Recent Activity</CardTitle>
              <CardDescription>Latest DNS queries across your domains</CardDescription>
            </CardHeader>
            <CardContent>
              {loading ? (
                <div className="flex items-center justify-center py-8">
                  <Loader2 className="h-5 w-5 animate-spin text-zinc-500" />
                </div>
              ) : logs.length === 0 ? (
                <p className="py-8 text-center text-sm text-zinc-500">
                  No recent activity
                </p>
              ) : (
                <div className="space-y-1">
                  {logs.slice(0, 10).map((log) => (
                    <div
                      key={log.id}
                      className="flex items-center gap-3 rounded-lg px-3 py-2 text-sm hover:bg-zinc-100 dark:hover:bg-zinc-800"
                    >
                      {log.status === 0 ? (
                        <CheckCircle2 className="h-4 w-4 shrink-0 text-green-500" />
                      ) : (
                        <XCircle className="h-4 w-4 shrink-0 text-red-400" />
                      )}
                      <span className="min-w-0 flex-1 truncate font-mono text-xs">
                        {log.path}
                      </span>
                      <span className="shrink-0 text-xs text-zinc-400">
                        {log.ms}ms
                      </span>
                      <span className="flex shrink-0 items-center gap-1 text-xs text-zinc-500">
                        <Clock className="h-3 w-3" />
                        {formatTime(log.timestamp)}
                      </span>
                    </div>
                  ))}
                </div>
              )} 
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
