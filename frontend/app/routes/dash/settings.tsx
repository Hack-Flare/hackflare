import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"
import { getUserDisplayName, useAuth } from "~/lib/auth-context"
import { Copy, Eye, Trash2, Bell, Lock, Shield } from "lucide-react"
import { Button } from "~/components/ui/button"

const apiKeys = [
  {
    id: 1,
    name: "Production API",
    created: "Jan 12, 2024",
    lastUsed: "5m ago",
    status: "active",
  },
  {
    id: 2,
    name: "Webhook Integration",
    created: "Dec 28, 2023",
    lastUsed: "2h ago",
    status: "active",
  },
  {
    id: 3,
    name: "Old Staging Key",
    created: "Nov 15, 2023",
    lastUsed: "Never",
    status: "revoked",
  },
]

export default function Settings() {
  const { user } = useAuth()
  const displayName = getUserDisplayName(user)

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold dark:text-white">Settings</h1>
        <p className="mt-2 text-zinc-600 dark:text-zinc-400">
          Tune account, API, workspace defaults
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Account
          </CardTitle>
          <CardDescription>Name, id, account management</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-3 gap-4">
            <div>
              <p className="text-xs font-medium text-zinc-600 dark:text-zinc-400">
                Name
              </p>
              <p className="mt-1 text-sm font-medium">{displayName}</p>
            </div>
            <div>
              <p className="text-xs font-medium text-zinc-600 dark:text-zinc-400">
                User ID
              </p>
              <p className="mt-1 text-sm font-medium">
                {user?.id || "Unknown"}
              </p>
            </div>
            <div>
              <p className="text-xs font-medium text-zinc-600 dark:text-zinc-400">
                Status
              </p>
              <p className="mt-1 text-sm font-medium">
                Authenticated via Hack Club
              </p>
            </div>
          </div>
          <div className="flex gap-2">
            <Button className="rounded bg-orange-500 px-3 py-2 text-sm font-medium text-white hover:bg-orange-600">
              Change Password
            </Button>
            <Button className="rounded bg-zinc-100 px-3 py-2 text-sm font-medium text-zinc-900 hover:bg-zinc-200 dark:bg-zinc-800 dark:text-zinc-100 dark:hover:bg-zinc-700">
              Edit Profile
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Lock className="h-5 w-5" />
            API Keys
          </CardTitle>
          <CardDescription>Manage authentication tokens</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {apiKeys.map((key) => (
              <div
                key={key.id}
                className="flex items-center justify-between rounded-lg border border-zinc-200 p-3 hover:bg-zinc-50 dark:border-zinc-800 dark:hover:bg-zinc-800/50"
              >
                <div className="min-w-0 flex-1">
                  <p className="text-sm font-medium">{key.name}</p>
                  <p className="mt-0.5 text-xs text-zinc-600 dark:text-zinc-400">
                    Created {key.created} • Last used {key.lastUsed}
                  </p>
                </div>
                <span
                  className={`rounded px-2 py-1 text-xs font-medium ${
                    key.status === "active"
                      ? "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
                      : "bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-400"
                  }`}
                >
                  {key.status}
                </span>
                <div className="ml-3 flex gap-1">
                  <Button variant="ghost" size="icon">
                    <Eye className="h-4 w-4" />
                  </Button>
                  <Button variant="ghost" size="icon">
                    <Copy className="h-4 w-4" />
                  </Button>
                  <Button variant="destructive" size="icon">
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            ))}
          </div>
          <Button className="mt-4 w-full rounded bg-zinc-100 px-3 py-2 text-sm font-medium text-zinc-900 hover:bg-zinc-200 dark:bg-zinc-800 dark:text-zinc-100 dark:hover:bg-zinc-700">
            Generate New Key
          </Button>
        </CardContent>
      </Card>

      <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Bell className="h-5 w-5" />
              Notifications
            </CardTitle>
            <CardDescription>Email + alert preferences</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <label className="flex cursor-pointer items-center gap-3">
              <input
                type="checkbox"
                defaultChecked
                className="h-4 w-4 rounded"
              />
              <span className="text-sm">Security alerts</span>
            </label>
            <label className="flex cursor-pointer items-center gap-3">
              <input
                type="checkbox"
                defaultChecked
                className="h-4 w-4 rounded"
              />
              <span className="text-sm">Weekly reports</span>
            </label>
            <label className="flex cursor-pointer items-center gap-3">
              <input type="checkbox" className="h-4 w-4 rounded" />
              <span className="text-sm">Marketing emails</span>
            </label>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Lock className="h-5 w-5" />
              Security
            </CardTitle>
            <CardDescription>Account protection settings</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex items-center justify-between rounded border border-zinc-200 p-2 dark:border-zinc-800">
              <span className="text-sm">Two-factor authentication</span>
              <span className="text-xs font-medium text-orange-600">Setup</span>
            </div>
            <div className="flex items-center justify-between rounded border border-zinc-200 p-2 dark:border-zinc-800">
              <span className="text-sm">Session timeout</span>
              <span className="text-xs text-zinc-600 dark:text-zinc-400">
                30 minutes
              </span>
            </div>
            <div className="flex items-center justify-between rounded border border-zinc-200 p-2 dark:border-zinc-800">
              <span className="text-sm">Active sessions</span>
              <span className="text-xs text-zinc-600 dark:text-zinc-400">
                1
              </span>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
