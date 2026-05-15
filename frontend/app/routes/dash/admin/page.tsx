import {
  Users,
  Shield,
  Activity,
  AlertTriangle,
  Search,
  MoreHorizontal,
} from "lucide-react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"
import { Button } from "~/components/ui/button"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "~/components/ui/table"

const users = [
  {
    id: 1,
    name: "John Doe",
    email: "john@example.com",
    role: "admin",
    status: "active",
    created: "Jan 12, 2024",
    lastActive: "now",
  },
  {
    id: 2,
    name: "Jane Smith",
    email: "jane@example.com",
    role: "user",
    status: "active",
    created: "Feb 3, 2024",
    lastActive: "2h ago",
  },
  {
    id: 3,
    name: "Bob Wilson",
    email: "bob@example.com",
    role: "user",
    status: "active",
    created: "Jan 28, 2024",
    lastActive: "1d ago",
  },
  {
    id: 4,
    name: "Alice Brown",
    email: "alice@example.com",
    role: "user",
    status: "suspended",
    created: "Mar 5, 2024",
    lastActive: "5d ago",
  },
  {
    id: 5,
    name: "Charlie Davis",
    email: "charlie@example.com",
    role: "user",
    status: "active",
    created: "Jan 20, 2024",
    lastActive: "3h ago",
  },
]

const systemStats = [
  { label: "Total Users", value: "247", change: "+12 this month" },
  { label: "Active Sessions", value: "34", change: "Real-time" },
  { label: "Domains Managed", value: "512", change: "+28 this month" },
  { label: "System Health", value: "99.9%", change: "Last 7 days" },
]

const auditLog = [
  {
    id: 1,
    action: "User created",
    user: "John Doe",
    target: "jane@example.com",
    timestamp: "14:32",
    status: "success",
  },
  {
    id: 2,
    action: "Role changed",
    user: "Admin",
    target: "bob@example.com",
    timestamp: "12:15",
    status: "success",
  },
  {
    id: 3,
    action: "User suspended",
    user: "Admin",
    target: "alice@example.com",
    timestamp: "10:48",
    status: "success",
  },
  {
    id: 4,
    action: "Password reset",
    user: "System",
    target: "charlie@example.com",
    timestamp: "09:22",
    status: "success",
  },
  {
    id: 5,
    action: "API key revoked",
    user: "Admin",
    target: "john@example.com",
    timestamp: "08:10",
    status: "success",
  },
]

export default function Admin() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold dark:text-white">Admin Panel</h1>
        <p className="mt-2 text-zinc-600 dark:text-zinc-400">
          User mgmt, system health + audit logs
        </p>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-4">
        {systemStats.map((stat, i) => (
          <Card key={i}>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium">
                {stat.label}
              </CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-2xl font-bold">{stat.value}</p>
              <p className="mt-1 text-xs text-zinc-600 dark:text-zinc-400">
                {stat.change}
              </p>
            </CardContent>
          </Card>
        ))}
      </div>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-3">
        <div className="lg:col-span-2">
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <div>
                  <CardTitle className="flex items-center gap-2">
                    <Users className="h-5 w-5" />
                    Users
                  </CardTitle>
                  <CardDescription>Manage all registered users</CardDescription>
                </div>
                <Button className="rounded bg-orange-500 px-3 py-2 text-sm font-medium text-white hover:bg-orange-600">
                  Add User
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              <div className="mb-4 flex gap-2">
                <div className="relative flex-1">
                  <Search className="absolute top-2.5 left-3 h-4 w-4 text-zinc-400" />
                  <input
                    type="text"
                    placeholder="Search users..."
                    className="w-full rounded border border-zinc-200 bg-white py-2 pr-3 pl-9 text-sm dark:border-zinc-800 dark:bg-zinc-900"
                  />
                </div>
              </div>
              <div className="overflow-x-auto">
                <Table className="[&_td]:h-auto [&_th]:h-auto">
                  <TableHeader>
                    <TableRow className="border-zinc-800 hover:bg-transparent">
                      <TableHead className="font-semibold text-zinc-400">
                        Name
                      </TableHead>
                      <TableHead className="font-semibold text-zinc-400">
                        Email
                      </TableHead>
                      <TableHead className="font-semibold text-zinc-400">
                        Role
                      </TableHead>
                      <TableHead className="text-center font-semibold text-zinc-400">
                        Status
                      </TableHead>
                      <TableHead className="text-right font-semibold text-zinc-400">
                        Actions
                      </TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {users.map((user) => (
                      <TableRow
                        key={user.id}
                        className="border-zinc-800 hover:bg-zinc-800/50"
                      >
                        <TableCell className="font-medium">
                          {user.name}
                        </TableCell>
                        <TableCell className="text-xs text-zinc-400">
                          {user.email}
                        </TableCell>
                        <TableCell>
                          <span
                            className={`rounded px-2 py-1 text-xs font-medium ${
                              user.role === "admin"
                                ? "border border-purple-700 bg-purple-900/30 text-purple-400"
                                : "border border-zinc-700 bg-zinc-800 text-zinc-300"
                            }`}
                          >
                            {user.role}
                          </span>
                        </TableCell>
                        <TableCell className="text-center">
                          <span
                            className={`rounded px-2 py-1 text-xs font-medium ${
                              user.status === "active"
                                ? "border border-green-700 bg-green-900/30 text-green-400"
                                : "border border-red-700 bg-red-900/30 text-red-400"
                            }`}
                          >
                            {user.status}
                          </span>
                        </TableCell>
                        <TableCell className="text-right">
                          <Button variant="ghost" size="icon">
                            <MoreHorizontal className="h-4 w-4" />
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            </CardContent>
          </Card>
        </div>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Shield className="h-5 w-5" />
              System
            </CardTitle>
            <CardDescription>Health + info</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="rounded-lg border border-zinc-200 p-3 dark:border-zinc-800">
              <p className="text-xs font-medium text-zinc-600 dark:text-zinc-400">
                Database
              </p>
              <p className="mt-1 flex items-center gap-2 text-sm font-medium">
                <span className="h-2 w-2 rounded-full bg-green-500" />
                Connected
              </p>
            </div>
            <div className="rounded-lg border border-zinc-200 p-3 dark:border-zinc-800">
              <p className="text-xs font-medium text-zinc-600 dark:text-zinc-400">
                Cache
              </p>
              <p className="mt-1 flex items-center gap-2 text-sm font-medium">
                <span className="h-2 w-2 rounded-full bg-green-500" />
                Healthy
              </p>
            </div>
            <div className="rounded-lg border border-zinc-200 p-3 dark:border-zinc-800">
              <p className="text-xs font-medium text-zinc-600 dark:text-zinc-400">
                API Rate
              </p>
              <p className="mt-1 text-sm font-medium">2.4K req/s</p>
            </div>
            <div className="rounded-lg border border-zinc-200 p-3 dark:border-zinc-800">
              <p className="text-xs font-medium text-zinc-600 dark:text-zinc-400">
                Last Backup
              </p>
              <p className="mt-1 text-sm font-medium">2h ago</p>
            </div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity className="h-5 w-5" />
            Audit Log
          </CardTitle>
          <CardDescription>Recent admin + system actions</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {auditLog.map((log) => (
              <div
                key={log.id}
                className="flex items-center gap-4 rounded-lg border border-zinc-200 p-3 hover:bg-zinc-50 dark:border-zinc-800 dark:hover:bg-zinc-800/50"
              >
                <div className="min-w-12 text-xs font-medium text-zinc-600 dark:text-zinc-400">
                  {log.timestamp}
                </div>
                <div className="min-w-0 flex-1">
                  <p className="text-sm font-medium">{log.action}</p>
                  <p className="text-xs text-zinc-600 dark:text-zinc-400">
                    By {log.user} on {log.target}
                  </p>
                </div>
                <span className="rounded bg-green-100 px-2 py-1 text-xs font-medium text-green-800 dark:bg-green-900/30 dark:text-green-400">
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
