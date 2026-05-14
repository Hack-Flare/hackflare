import { Users, Shield, Activity, AlertTriangle, Search, MoreHorizontal } from "lucide-react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card"

const users = [
  { id: 1, name: "John Doe", email: "john@example.com", role: "admin", status: "active", created: "Jan 12, 2024", lastActive: "now" },
  { id: 2, name: "Jane Smith", email: "jane@example.com", role: "user", status: "active", created: "Feb 3, 2024", lastActive: "2h ago" },
  { id: 3, name: "Bob Wilson", email: "bob@example.com", role: "user", status: "active", created: "Jan 28, 2024", lastActive: "1d ago" },
  { id: 4, name: "Alice Brown", email: "alice@example.com", role: "user", status: "suspended", created: "Mar 5, 2024", lastActive: "5d ago" },
  { id: 5, name: "Charlie Davis", email: "charlie@example.com", role: "user", status: "active", created: "Jan 20, 2024", lastActive: "3h ago" },
]

const systemStats = [
  { label: "Total Users", value: "247", change: "+12 this month" },
  { label: "Active Sessions", value: "34", change: "Real-time" },
  { label: "Domains Managed", value: "512", change: "+28 this month" },
  { label: "System Health", value: "99.9%", change: "Last 7 days" },
]

const auditLog = [
  { id: 1, action: "User created", user: "John Doe", target: "jane@example.com", timestamp: "14:32", status: "success" },
  { id: 2, action: "Role changed", user: "Admin", target: "bob@example.com", timestamp: "12:15", status: "success" },
  { id: 3, action: "User suspended", user: "Admin", target: "alice@example.com", timestamp: "10:48", status: "success" },
  { id: 4, action: "Password reset", user: "System", target: "charlie@example.com", timestamp: "09:22", status: "success" },
  { id: 5, action: "API key revoked", user: "Admin", target: "john@example.com", timestamp: "08:10", status: "success" },
]

export default function Admin() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold dark:text-white">Admin Panel</h1>
        <p className="text-zinc-600 dark:text-zinc-400 mt-2">User mgmt, system health + audit logs</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        {systemStats.map((stat, i) => (
          <Card key={i}>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium">{stat.label}</CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-2xl font-bold">{stat.value}</p>
              <p className="text-xs text-zinc-600 dark:text-zinc-400 mt-1">{stat.change}</p>
            </CardContent>
          </Card>
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2">
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <div>
                  <CardTitle className="flex items-center gap-2"><Users className="h-5 w-5" />Users</CardTitle>
                  <CardDescription>Manage all registered users</CardDescription>
                </div>
                <button className="px-3 py-2 rounded text-sm font-medium bg-orange-500 hover:bg-orange-600 text-white">
                  Add User
                </button>
              </div>
            </CardHeader>
            <CardContent>
              <div className="mb-4 flex gap-2">
                <div className="flex-1 relative">
                  <Search className="absolute left-3 top-2.5 h-4 w-4 text-zinc-400" />
                  <input
                    type="text"
                    placeholder="Search users..."
                    className="w-full pl-9 pr-3 py-2 rounded border border-zinc-200 dark:border-zinc-800 bg-white dark:bg-zinc-900 text-sm"
                  />
                </div>
              </div>
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b border-zinc-200 dark:border-zinc-800">
                      <th className="text-left py-3 px-4 font-semibold">Name</th>
                      <th className="text-left py-3 px-4 font-semibold">Email</th>
                      <th className="text-left py-3 px-4 font-semibold">Role</th>
                      <th className="text-center py-3 px-4 font-semibold">Status</th>
                      <th className="text-right py-3 px-4 font-semibold">Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    {users.map((user) => (
                      <tr key={user.id} className="border-b border-zinc-100 dark:border-zinc-800 hover:bg-zinc-50 dark:hover:bg-zinc-800/50">
                        <td className="py-3 px-4 font-medium">{user.name}</td>
                        <td className="py-3 px-4 text-zinc-600 dark:text-zinc-400 text-xs">{user.email}</td>
                        <td className="py-3 px-4">
                          <span className={`px-2 py-1 rounded text-xs font-medium ${
                            user.role === "admin"
                              ? "bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-400"
                              : "bg-zinc-100 text-zinc-800 dark:bg-zinc-800 dark:text-zinc-300"
                          }`}>
                            {user.role}
                          </span>
                        </td>
                        <td className="py-3 px-4 text-center">
                          <span className={`px-2 py-1 rounded text-xs font-medium ${
                            user.status === "active"
                              ? "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
                              : "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400"
                          }`}>
                            {user.status}
                          </span>
                        </td>
                        <td className="py-3 px-4 text-right">
                          <button className="p-2 hover:bg-zinc-100 dark:hover:bg-zinc-700 rounded">
                            <MoreHorizontal className="h-4 w-4" />
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>
        </div>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2"><Shield className="h-5 w-5" />System</CardTitle>
            <CardDescription>Health + info</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="rounded-lg border border-zinc-200 dark:border-zinc-800 p-3">
              <p className="text-xs text-zinc-600 dark:text-zinc-400 font-medium">Database</p>
              <p className="text-sm font-medium mt-1 flex items-center gap-2">
                <span className="h-2 w-2 rounded-full bg-green-500" />
                Connected
              </p>
            </div>
            <div className="rounded-lg border border-zinc-200 dark:border-zinc-800 p-3">
              <p className="text-xs text-zinc-600 dark:text-zinc-400 font-medium">Cache</p>
              <p className="text-sm font-medium mt-1 flex items-center gap-2">
                <span className="h-2 w-2 rounded-full bg-green-500" />
                Healthy
              </p>
            </div>
            <div className="rounded-lg border border-zinc-200 dark:border-zinc-800 p-3">
              <p className="text-xs text-zinc-600 dark:text-zinc-400 font-medium">API Rate</p>
              <p className="text-sm font-medium mt-1">2.4K req/s</p>
            </div>
            <div className="rounded-lg border border-zinc-200 dark:border-zinc-800 p-3">
              <p className="text-xs text-zinc-600 dark:text-zinc-400 font-medium">Last Backup</p>
              <p className="text-sm font-medium mt-1">2h ago</p>
            </div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2"><Activity className="h-5 w-5" />Audit Log</CardTitle>
          <CardDescription>Recent admin + system actions</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {auditLog.map((log) => (
              <div key={log.id} className="flex items-center gap-4 p-3 rounded-lg border border-zinc-200 dark:border-zinc-800 hover:bg-zinc-50 dark:hover:bg-zinc-800/50">
                <div className="text-xs text-zinc-600 dark:text-zinc-400 font-medium min-w-12">{log.timestamp}</div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium">{log.action}</p>
                  <p className="text-xs text-zinc-600 dark:text-zinc-400">By {log.user} on {log.target}</p>
                </div>
                <span className="px-2 py-1 rounded text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400">
                  {log.status}
                </span>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
