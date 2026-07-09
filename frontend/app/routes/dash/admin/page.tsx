import { useCallback, useEffect, useRef, useState } from "react"
import {
  Activity,
  AlertTriangle,
  BarChart2,
  CheckCircle2,
  ChevronDown,
  ChevronRight,
  Database,
  Globe,
  Pencil,
  RefreshCw,
  Save,
  Settings,
  Trash2,
  Users,
  X,
  XCircle,
} from "lucide-react"
import { Area, AreaChart, Bar, BarChart, CartesianGrid, Cell, XAxis, YAxis } from "recharts"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"
import {
  ChartContainer,
  ChartLegend,
  ChartLegendContent,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from "~/components/ui/chart"
import { Button } from "~/components/ui/button"
import { Input } from "~/components/ui/input"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "~/components/ui/dialog"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "~/components/ui/table"
import { useToast } from "~/lib/toast"
import { api, type AdminStats, type AdminUser, type ConfigEntry, type TrafficSummary, type TimeseriesPoint, type TopQuery } from "~/lib/api"

type Tab = "config" | "users" | "stats" | "traffic"

export default function Admin() {
  const [tab, setTab] = useState<Tab>("config")

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold dark:text-white">Admin Panel</h1>
        <p className="mt-2 text-zinc-600 dark:text-zinc-400">
          System configuration, users, and stats
        </p>
      </div>

      <TabNav tab={tab} onTabChange={setTab} />

      {tab === "config" && <ConfigTab />}
      {tab === "users" && <UsersTab />}
      {tab === "stats" && <StatsTab />}
      {tab === "traffic" && <TrafficTab />}
    </div>
  )
}

function TabNav({ tab, onTabChange }: { tab: Tab; onTabChange: (t: Tab) => void }) {
  const tabs: { key: Tab; label: string; icon: React.ReactNode }[] = [
    { key: "config", label: "Config", icon: <Settings className="h-4 w-4" /> },
    { key: "users", label: "Users", icon: <Users className="h-4 w-4" /> },
    { key: "stats", label: "Stats", icon: <Activity className="h-4 w-4" /> },
    { key: "traffic", label: "Traffic", icon: <BarChart2 className="h-4 w-4" /> },
  ]

  return (
    <div className="flex gap-1 rounded-lg border border-zinc-200 bg-white p-1 dark:border-zinc-800 dark:bg-zinc-900">
      {tabs.map((t) => (
        <button
          key={t.key}
          onClick={() => onTabChange(t.key)}
          className={`flex items-center gap-2 rounded-md px-4 py-2 text-sm font-medium transition-colors ${
            tab === t.key
              ? "bg-orange-500 text-white"
              : "text-zinc-600 hover:bg-zinc-100 dark:text-zinc-400 dark:hover:bg-zinc-800"
          }`}
        >
          {t.icon}
          {t.label}
        </button>
      ))}
    </div>
  )
}

// ── Config Tab ──

function ConfigTab() {
  const [entries, setEntries] = useState<ConfigEntry[]>([])
  const [loading, setLoading] = useState(true)
  const [editKey, setEditKey] = useState<string | null>(null)
  const [editValue, setEditValue] = useState("")
  const [saving, setSaving] = useState(false)
  const [applying, setApplying] = useState(false)
  const [deleteKey, setDeleteKey] = useState<string | null>(null)
  const editRef = useRef<HTMLInputElement>(null)
  const { toast } = useToast()

  const apply = async () => {
    setApplying(true)
    try {
      await api.admin.applyConfig()
      toast("Configuration applied.", "success")
    } catch {
      toast("Failed to apply configuration.", "error")
    }
    setApplying(false)
  }

  const load = useCallback(async () => {
    setLoading(true)
    try {
      const data = await api.admin.listConfig()
      setEntries(data)
    } catch {
      toast("Failed to load configuration.", "error")
    }
    setLoading(false)
  }, [toast])

  useEffect(() => {
    load()
  }, [load])

  const startEdit = (entry: ConfigEntry) => {
    setEditKey(entry.key)
    setEditValue(
      entry.override_value ??
        (entry.default_override ? entry.default_value : entry.env_value) ??
        ""
    )
  }

  const cancelEdit = () => {
    setEditKey(null)
    setEditValue("")
  }

  const saveEdit = async (key: string) => {
    setSaving(true)
    try {
      await api.admin.upsertConfig(key, editValue)
      setEditKey(null)
      toast("Config saved.", "success")
      await load()
    } catch {
      toast("Failed to save config.", "error")
    }
    setSaving(false)
  }

  const confirmDelete = async () => {
    if (!deleteKey) return
    try {
      await api.admin.deleteConfig(deleteKey)
      toast("Override deleted.", "success")
      setDeleteKey(null)
      await load()
    } catch {
      toast("Failed to delete override.", "error")
      setDeleteKey(null)
    }
  }

  const [collapsed, setCollapsed] = useState<Set<string>>(
    () => new Set(["Server", "Authentication", "Database", "DNS", "Email", "Admin", "Frontend"])
  )
  const toggle = (cat: string) =>
    setCollapsed((prev) => {
      const next = new Set(prev)
      if (next.has(cat)) next.delete(cat)
      else next.add(cat)
      return next
    })

  const grouped = entries.reduce<Record<string, ConfigEntry[]>>((acc, e) => {
    ;(acc[e.category] ??= []).push(e)
    return acc
  }, {})

  const categoryOrder = [
    "Server",
    "Authentication",
    "Database",
    "DNS",
    "Email",
    "Admin",
    "Frontend",
  ]

  if (loading) {
    return (
      <Card>
        <CardContent className="flex items-center justify-center py-12">
          <RefreshCw className="h-6 w-6 animate-spin text-zinc-400" />
        </CardContent>
      </Card>
    )
  }

  return (
    <>
      <div className="mb-4 flex items-center justify-between">
        <div>
          <h2 className="flex items-center gap-2 text-lg font-semibold">
            <Settings className="h-5 w-5" />
            Configuration
          </h2>
          <p className="text-sm text-zinc-500">
            Environment variables and live overrides
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={load}>
            <RefreshCw className="mr-1 h-4 w-4" />
            Refresh
          </Button>
          <Button variant="default" size="sm" onClick={apply} disabled={applying}>
            <RefreshCw
              className={`mr-1 h-4 w-4 ${applying ? "animate-spin" : ""}`}
            />
            {applying ? "Applying..." : "Take Effect Now"}
          </Button>
        </div>
      </div>

      <div className="space-y-6">
        {entries.length === 0 ? (
          <Card>
            <CardContent className="flex items-center justify-center py-12 text-zinc-500">
              No configuration entries found.
            </CardContent>
          </Card>
        ) : (
          categoryOrder.map((category) => {
            const catEntries = grouped[category]
            if (!catEntries) return null
            const isCollapsed = collapsed.has(category)
            return (
              <div key={category}>
                <button
                  type="button"
                  onClick={() => toggle(category)}
                  className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left text-sm font-semibold hover:bg-zinc-100 dark:hover:bg-zinc-800"
                >
                  {isCollapsed ? (
                    <ChevronRight className="h-4 w-4 text-zinc-400" />
                  ) : (
                    <ChevronDown className="h-4 w-4 text-zinc-400" />
                  )}
                  {category}
                  <span className="ml-auto text-xs font-normal text-zinc-500">
                    {catEntries.length} {catEntries.length === 1 ? "key" : "keys"}
                  </span>
                </button>
                {!isCollapsed && (
                  <div className="mt-2 space-y-2 pl-2">
                    {catEntries.map((entry) => (
                      <div
                        key={entry.key}
                        className="rounded-lg border border-zinc-200 bg-white p-4 dark:border-zinc-800 dark:bg-zinc-900"
                      >
                        <div className="flex items-start justify-between gap-4">
                          <div className="min-w-0 flex-1">
                            <div className="flex items-center gap-2">
                              <span className="text-sm font-medium">
                                {entry.label}
                              </span>
                              {entry.requires_restart ? (
                                <span className="rounded bg-amber-900/30 px-1.5 py-0.5 text-[10px] font-medium text-amber-400">
                                  Restart
                                </span>
                              ) : (
                                <span className="rounded bg-green-900/30 px-1.5 py-0.5 text-[10px] font-medium text-green-400">
                                  Live
                                </span>
                              )}
                            </div>
                            <div className="mt-0.5 truncate font-mono text-xs text-zinc-500">
                              {entry.key}
                            </div>
                            {entry.description && (
                              <div className="mt-0.5 text-xs text-zinc-400">
                                {entry.description}
                              </div>
                            )}
                          </div>
                          <div className="flex shrink-0 items-center gap-1">
                            {entry.editable && editKey !== entry.key && (
                              <>
                                <Button
                                  variant="ghost"
                                  size="icon"
                                  className="h-8 w-8"
                                  onClick={() => startEdit(entry)}
                                  title="Edit"
                                >
                                  <Pencil className="h-4 w-4" />
                                </Button>
                                {entry.override_value && (
                                  <Button
                                    variant="ghost"
                                    size="icon"
                                    className="h-8 w-8"
                                    onClick={() => setDeleteKey(entry.key)}
                                    title="Delete override"
                                  >
                                    <Trash2 className="h-4 w-4 text-red-400" />
                                  </Button>
                                )}
                              </>
                            )}
                          </div>
                        </div>

                        <div className="mt-3 grid grid-cols-3 gap-4 rounded-md bg-zinc-50 p-3 text-xs dark:bg-zinc-800/50">
                          <div className="min-w-0">
                            <span className="text-zinc-500">Default</span>
                            <div
                              className="mt-0.5 truncate font-mono"
                              title={entry.default_value ?? ""}
                            >
                              {entry.default_value || (
                                <span className="text-zinc-500 italic">
                                  none
                                </span>
                              )}
                            </div>
                            {entry.default_override && entry.env_value && (
                              <div
                                className="mt-0.5 truncate text-[10px] text-zinc-500 line-through"
                                title={entry.env_value}
                              >
                                env: {entry.env_value}
                              </div>
                            )}
                          </div>
                          <div className="min-w-0">
                            <span className="text-zinc-500">Effective</span>
                            <div className="mt-0.5 flex items-center gap-1.5">
                              <span
                                className={`inline-block h-2 w-2 shrink-0 rounded-full ${
                                  entry.override_value
                                    ? "bg-green-500"
                                    : "bg-zinc-500"
                                }`}
                              />
                              <div
                                className="truncate font-mono"
                                title={entry.effective_value}
                              >
                                {entry.effective_value || (
                                  <span className="text-zinc-500 italic">
                                    none
                                  </span>
                                )}
                              </div>
                            </div>
                          </div>
                          <div className="min-w-0">
                            <span className="text-zinc-500">Updated</span>
                            <div className="mt-0.5 truncate text-zinc-400">
                              {entry.updated_at
                                ? new Date(entry.updated_at).toLocaleString()
                                : "—"}
                            </div>
                            {entry.updated_by && (
                              <div className="truncate text-[10px] text-zinc-500">
                                by {entry.updated_by}
                              </div>
                            )}
                          </div>
                        </div>

                        {editKey === entry.key && (
                          <div className="mt-3 flex items-center gap-2">
                            <Input
                              ref={editRef}
                              value={editValue}
                              onChange={(e) => setEditValue(e.target.value)}
                              className="flex-1"
                              autoFocus
                              placeholder="Enter override value..."
                              onKeyDown={(e) => {
                                if (e.key === "Enter") saveEdit(entry.key)
                                if (e.key === "Escape") cancelEdit()
                              }}
                            />
                            <Button
                              variant="default"
                              size="sm"
                              onClick={() => saveEdit(entry.key)}
                              disabled={saving}
                            >
                              <Save className="mr-1 h-4 w-4" />
                              Save
                            </Button>
                            <Button variant="outline" size="sm" onClick={cancelEdit}>
                              <X className="mr-1 h-4 w-4" />
                              Cancel
                            </Button>
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )
          })
        )}
      </div>

      <Dialog open={deleteKey !== null} onOpenChange={(open) => !open && setDeleteKey(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-red-400" />
              Delete override?
            </DialogTitle>
            <DialogDescription>
              This will remove the override for <code className="rounded bg-zinc-800 px-1 py-0.5 font-mono text-sm">{deleteKey}</code>. The
              effective value will revert to the default or environment value.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteKey(null)}>
              Cancel
            </Button>
            <Button variant="destructive" onClick={confirmDelete}>
              <Trash2 className="mr-1 h-4 w-4" />
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}

// ── Users Tab ──

function UsersTab() {
  const [users, setUsers] = useState<AdminUser[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    api.admin
      .listUsers()
      .then(setUsers)
      .catch(() => {})
      .finally(() => setLoading(false))
  }, [])

  if (loading) {
    return (
      <Card>
        <CardContent className="flex items-center justify-center py-12">
          <RefreshCw className="h-6 w-6 animate-spin text-zinc-400" />
        </CardContent>
      </Card>
    )
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Users className="h-5 w-5" />
          Users
        </CardTitle>
        <CardDescription>All registered users</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="overflow-x-auto">
          <Table className="[&_td]:h-auto [&_th]:h-auto">
            <TableHeader>
              <TableRow className="border-zinc-800 hover:bg-transparent">
                <TableHead className="font-semibold text-zinc-400">Name</TableHead>
                <TableHead className="font-semibold text-zinc-400">Email</TableHead>
                <TableHead className="font-semibold text-zinc-400">Status</TableHead>
                <TableHead className="font-semibold text-zinc-400">Created</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {users.map((user) => (
                <TableRow
                  key={user.id}
                  className="border-zinc-800 hover:bg-zinc-800/50"
                >
                  <TableCell className="font-medium">
                    {user.first_name} {user.last_name}
                  </TableCell>
                  <TableCell className="text-xs text-zinc-400">
                    {user.email}
                  </TableCell>
                  <TableCell>
                    <span
                      className={`rounded px-2 py-1 text-xs font-medium ${
                        user.status === "verified"
                          ? "border border-green-700 bg-green-900/30 text-green-400"
                          : "border border-zinc-700 bg-zinc-800 text-zinc-300"
                      }`}
                    >
                      {user.status}
                    </span>
                  </TableCell>
                  <TableCell className="text-xs text-zinc-400">
                    {new Date(user.created_at).toLocaleDateString()}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>
  )
}

// ── Traffic Tab ──

const adminTrafficConfig = {
  requests: {
    label: "Requests",
    color: "hsl(200, 100%, 50%)",
  },
  errors: {
    label: "Errors",
    color: "hsl(0, 100%, 50%)",
  },
} satisfies ChartConfig

const adminStatusColors = [
  "hsl(120, 100%, 40%)",
  "hsl(40, 100%, 50%)",
  "hsl(0, 100%, 50%)",
  "hsl(240, 100%, 60%)",
]

function TrafficTab() {
  const [summary, setSummary] = useState<TrafficSummary | null>(null)
  const [timeseries, setTimeseries] = useState<TimeseriesPoint[]>([])
  const [topQueries, setTopQueries] = useState<TopQuery[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    Promise.all([
      api.admin.trafficSummary(),
      api.admin.trafficTimeseries(30),
      api.admin.trafficTopQueries(10),
    ])
      .then(([s, ts, tq]) => {
        setSummary(s)
        setTimeseries(ts)
        setTopQueries(tq)
      })
      .catch((err) => {
        setError(
          err && typeof err === "object" && "error" in err
            ? String((err as { error: unknown }).error)
            : "Failed to load traffic data",
        )
      })
      .finally(() => setLoading(false))
  }, [])

  if (loading) {
    return (
      <Card>
        <CardContent className="flex items-center justify-center py-12">
          <RefreshCw className="h-6 w-6 animate-spin text-zinc-400" />
        </CardContent>
      </Card>
    )
  }

  if (error) {
    return (
      <Card>
        <CardContent className="flex items-center justify-center gap-2 py-12 text-red-500">
          <p className="text-sm">{error}</p>
        </CardContent>
      </Card>
    )
  }

  const statusCodeData = summary
    ? [
        {
          name: "NOERROR",
          value: Math.round(summary.total_requests * (summary.success_rate / 100)),
          fill: adminStatusColors[0],
        },
        { name: "Other", value: Math.round(summary.total_requests * (summary.error_rate / 100)), fill: adminStatusColors[2] },
      ]
    : []

  return (
    <div className="space-y-6">
      {summary && (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-4">
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium">Total Queries</CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-2xl font-bold">{summary.total_requests.toLocaleString()}</p>
              <p className="mt-2 text-xs text-zinc-500">Platform-wide all time</p>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium">Avg Processing Time</CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-2xl font-bold">{summary.avg_processing_ms.toFixed(1)}ms</p>
              <p className="mt-2 text-xs text-zinc-500">Per query</p>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium">Success Rate</CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-2xl font-bold text-green-600">{summary.success_rate.toFixed(1)}%</p>
              <p className="mt-2 text-xs text-zinc-500">NOERROR responses</p>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium">Error Rate</CardTitle>
            </CardHeader>
            <CardContent>
              <p className={`text-2xl font-bold ${summary.error_rate > 1 ? "text-red-600" : "text-zinc-900 dark:text-white"}`}>
                {summary.error_rate.toFixed(2)}%
              </p>
              <p className="mt-2 text-xs text-zinc-500">SERVFAIL + other errors</p>
            </CardContent>
          </Card>
        </div>
      )}

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Queries Over Time</CardTitle>
            <CardDescription>Platform-wide DNS queries over the last 30 days</CardDescription>
          </CardHeader>
          <CardContent>
            <ChartContainer config={adminTrafficConfig} className="h-62.5 w-full">
              <AreaChart data={timeseries}>
                <defs>
                  <linearGradient id="fillAdminRequests" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="var(--color-requests)" stopOpacity={0.8} />
                    <stop offset="95%" stopColor="var(--color-requests)" stopOpacity={0.1} />
                  </linearGradient>
                  <linearGradient id="fillAdminErrors" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="var(--color-errors)" stopOpacity={0.8} />
                    <stop offset="95%" stopColor="var(--color-errors)" stopOpacity={0.1} />
                  </linearGradient>
                </defs>
                <CartesianGrid vertical={false} />
                <XAxis
                  dataKey="date"
                  tickLine={false}
                  axisLine={false}
                  tickMargin={8}
                  tickFormatter={(value) => {
                    const date = new Date(value)
                    return date.toLocaleDateString("en-US", { month: "short", day: "numeric" })
                  }}
                />
                <YAxis tickLine={false} axisLine={false} />
                <ChartTooltip
                  cursor={false}
                  content={
                    <ChartTooltipContent
                      labelFormatter={(value) => new Date(value).toLocaleDateString("en-US", { month: "short", day: "numeric" })}
                      indicator="dot"
                    />
                  }
                />
                <Area dataKey="errors" type="natural" fill="url(#fillAdminErrors)" stroke="var(--color-errors)" stackId="a" />
                <Area dataKey="requests" type="natural" fill="url(#fillAdminRequests)" stroke="var(--color-requests)" stackId="a" />
                <ChartLegend content={<ChartLegendContent />} />
              </AreaChart>
            </ChartContainer>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Response Code Distribution</CardTitle>
            <CardDescription>Platform-wide DNS response codes</CardDescription>
          </CardHeader>
          <CardContent>
            <ChartContainer config={adminTrafficConfig} className="h-62.5 w-full">
              <BarChart data={statusCodeData}>
                <CartesianGrid vertical={false} />
                <XAxis dataKey="name" tickLine={false} tickMargin={10} axisLine={false} />
                <YAxis tickLine={false} axisLine={false} />
                <ChartTooltip cursor={false} content={<ChartTooltipContent hideLabel />} />
                <Bar dataKey="value" radius={4}>
                  {statusCodeData.map((entry) => (
                    <Cell key={entry.name} fill={entry.fill} />
                  ))}
                </Bar>
              </BarChart>
            </ChartContainer>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Top Queries</CardTitle>
          <CardDescription>Most frequently queried names platform-wide</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-zinc-200 dark:border-zinc-800">
                  <th className="px-4 py-2 text-left dark:text-zinc-400">Query</th>
                  <th className="px-4 py-2 text-right dark:text-zinc-400">Requests</th>
                </tr>
              </thead>
              <tbody>
                {topQueries.length === 0 ? (
                  <tr>
                    <td colSpan={2} className="py-8 text-center text-zinc-500">No query data yet</td>
                  </tr>
                ) : (
                  topQueries.map((q, i) => (
                    <tr key={i} className="border-b border-zinc-100 dark:border-zinc-800">
                      <td className="px-4 py-3 font-mono text-xs dark:text-white">{q.query}</td>
                      <td className="px-4 py-3 text-right dark:text-zinc-300">{q.count.toLocaleString()}</td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

// ── Stats Tab ──

function StatsTab() {
  const [stats, setStats] = useState<AdminStats | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    api.admin
      .getStats()
      .then(setStats)
      .catch(() => {})
      .finally(() => setLoading(false))
  }, [])

  if (loading) {
    return (
      <Card>
        <CardContent className="flex items-center justify-center py-12">
          <RefreshCw className="h-6 w-6 animate-spin text-zinc-400" />
        </CardContent>
      </Card>
    )
  }

  const statCards = stats
    ? [
        {
          label: "Total Users",
          value: stats.total_users,
          icon: <Users className="h-5 w-5 text-blue-500" />,
        },
        {
          label: "Active Sessions",
          value: stats.total_sessions,
          icon: <Activity className="h-5 w-5 text-green-500" />,
        },
        {
          label: "Domains Managed",
          value: stats.total_zones,
          icon: <Globe className="h-5 w-5 text-orange-500" />,
        },
        {
          label: "Database",
          value: "Connected",
          icon: <Database className="h-5 w-5 text-purple-500" />,
          indicator: "green" as const,
        },
      ]
    : []

  return (
    <>
      <div className="grid grid-cols-1 gap-4 md:grid-cols-4">
        {statCards.map((stat, i) => (
          <Card key={i}>
            <CardHeader className="pb-2">
              <div className="flex items-center justify-between">
                <CardTitle className="text-sm font-medium text-zinc-500">
                  {stat.label}
                </CardTitle>
                {stat.icon}
              </div>
            </CardHeader>
            <CardContent>
              <div className="flex items-center gap-2">
                <p className="text-2xl font-bold">{stat.value}</p>
                {"indicator" in stat && stat.indicator === "green" && (
                  <span className="h-2 w-2 rounded-full bg-green-500" />
                )}
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <CheckCircle2 className="h-5 w-5" />
            System Health
          </CardTitle>
          <CardDescription>All systems operational</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
            <div className="flex items-center gap-3 rounded-lg border border-zinc-200 p-4 dark:border-zinc-800">
              <CheckCircle2 className="h-5 w-5 text-green-500" />
              <div>
                <p className="text-sm font-medium">API Server</p>
                <p className="text-xs text-zinc-500">Running</p>
              </div>
            </div>
            <div className="flex items-center gap-3 rounded-lg border border-zinc-200 p-4 dark:border-zinc-800">
              <CheckCircle2 className="h-5 w-5 text-green-500" />
              <div>
                <p className="text-sm font-medium">DNS Server</p>
                <p className="text-xs text-zinc-500">Running</p>
              </div>
            </div>
            <div className="flex items-center gap-3 rounded-lg border border-zinc-200 p-4 dark:border-zinc-800">
              <CheckCircle2 className="h-5 w-5 text-green-500" />
              <div>
                <p className="text-sm font-medium">Database</p>
                <p className="text-xs text-zinc-500">
                  {stats?.total_sessions ?? 0} active sessions
                </p>
              </div>
            </div>
            <div className="flex items-center gap-3 rounded-lg border border-zinc-200 p-4 dark:border-zinc-800">
              <CheckCircle2 className="h-5 w-5 text-green-500" />
              <div>
                <p className="text-sm font-medium">Users</p>
                <p className="text-xs text-zinc-500">
                  {stats?.total_users ?? 0} registered
                </p>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </>
  )
}
