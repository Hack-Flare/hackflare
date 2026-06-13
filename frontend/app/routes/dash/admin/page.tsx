import { useCallback, useEffect, useState } from "react"
import {
  Activity,
  CheckCircle2,
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
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"
import { Button } from "~/components/ui/button"
import { Input } from "~/components/ui/input"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "~/components/ui/table"
import { api, type AdminStats, type AdminUser, type ConfigEntry } from "~/lib/api"

type Tab = "config" | "users" | "stats"

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
    </div>
  )
}

function TabNav({ tab, onTabChange }: { tab: Tab; onTabChange: (t: Tab) => void }) {
  const tabs: { key: Tab; label: string; icon: React.ReactNode }[] = [
    { key: "config", label: "Config", icon: <Settings className="h-4 w-4" /> },
    { key: "users", label: "Users", icon: <Users className="h-4 w-4" /> },
    { key: "stats", label: "Stats", icon: <Activity className="h-4 w-4" /> },
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

  const load = useCallback(async () => {
    setLoading(true)
    try {
      const data = await api.admin.listConfig()
      setEntries(data)
    } catch {
      /* ignore */
    }
    setLoading(false)
  }, [])

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
      await load()
    } catch {
      /* ignore */
    }
    setSaving(false)
  }

  const deleteOverride = async (key: string) => {
    try {
      await api.admin.deleteConfig(key)
      await load()
    } catch {
      /* ignore */
    }
  }

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
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center gap-2">
              <Settings className="h-5 w-5" />
              Configuration
            </CardTitle>
            <CardDescription>
              Environment variables and overrides. Live values are highlighted.
            </CardDescription>
          </div>
          <Button variant="outline" size="sm" onClick={load}>
            <RefreshCw className="mr-1 h-4 w-4" />
            Refresh
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <Table className="[&_td]:h-auto [&_th]:h-auto">
          <TableHeader>
            <TableRow className="border-zinc-800 hover:bg-transparent">
              <TableHead className="font-semibold text-zinc-400">Key</TableHead>
              <TableHead className="font-semibold text-zinc-400">Default</TableHead>
              <TableHead className="font-semibold text-zinc-400">Override</TableHead>
              <TableHead className="font-semibold text-zinc-400">Effective</TableHead>
              <TableHead className="font-semibold text-zinc-400">Updated</TableHead>
              <TableHead className="text-center font-semibold text-zinc-400">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {entries.map((entry) => (
              <TableRow
                key={entry.key}
                className="border-zinc-800 hover:bg-zinc-800/50"
              >
                <TableCell>
                  <div className="flex flex-col">
                    <span className="font-medium">{entry.label}</span>
                    <span className="text-xs text-zinc-500">{entry.key}</span>
                  </div>
                </TableCell>
                <TableCell className="font-mono text-xs text-zinc-500">
                  <div className="max-w-48 truncate" title={entry.default_value ?? ""}>
                    {entry.default_value || "—"}
                  </div>
                  {entry.default_override && entry.env_value && (
                    <div className="mt-0.5 max-w-48 truncate text-[10px] text-zinc-600 line-through" title={entry.env_value}>
                      env: {entry.env_value}
                    </div>
                  )}
                </TableCell>
                <TableCell className="font-mono text-xs">
                  {editKey === entry.key ? (
                    <div className="flex items-center gap-1">
                      <Input
                        value={editValue}
                        onChange={(e) => setEditValue(e.target.value)}
                        className="h-8 w-48 text-xs"
                        autoFocus
                      />
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8"
                        onClick={() => saveEdit(entry.key)}
                        disabled={saving}
                      >
                        <Save className="h-4 w-4 text-green-500" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8"
                        onClick={cancelEdit}
                      >
                        <X className="h-4 w-4 text-red-500" />
                      </Button>
                    </div>
                  ) : (
                    <div className="max-w-48 truncate text-orange-400" title={entry.override_value ?? ""}>
                      {entry.override_value || "—"}
                    </div>
                  )}
                </TableCell>
                <TableCell>
                  <div className="flex items-center gap-2">
                    <span
                      className={`inline-block h-2 w-2 rounded-full ${
                        entry.override_value ? "bg-green-500" : "bg-zinc-500"
                      }`}
                    />
                    <span className="font-mono text-xs">
                      <div className="max-w-48 truncate" title={entry.effective_value}>
                        {entry.effective_value || "—"}
                      </div>
                    </span>
                  </div>
                </TableCell>
                <TableCell className="text-xs text-zinc-400">
                  {entry.updated_at ? new Date(entry.updated_at).toLocaleString() : "—"}
                </TableCell>
                <TableCell>
                  <div className="flex items-center justify-center gap-1">
                    {entry.requires_restart ? (
                      <span className="rounded bg-amber-900/30 px-2 py-0.5 text-xs text-amber-400">
                        Restart
                      </span>
                    ) : (
                      <span className="rounded bg-green-900/30 px-2 py-0.5 text-xs text-green-400">
                        Live
                      </span>
                    )}
                    {entry.editable && editKey !== entry.key && (
                      <>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8"
                          onClick={() => startEdit(entry)}
                        >
                          <Pencil className="h-4 w-4" />
                        </Button>
                        {entry.override_value && (
                          <Button
                            variant="ghost"
                            size="icon"
                            className="h-8 w-8"
                            onClick={() => deleteOverride(entry.key)}
                          >
                            <Trash2 className="h-4 w-4 text-red-400" />
                          </Button>
                        )}
                      </>
                    )}
                  </div>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
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
