import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card"
import { useAuth } from "~/lib/auth-context"
import { Copy, Eye, Trash2, Bell, Lock, Shield } from "lucide-react"

const apiKeys = [
  { id: 1, name: "Production API", created: "Jan 12, 2024", lastUsed: "5m ago", status: "active" },
  { id: 2, name: "Webhook Integration", created: "Dec 28, 2023", lastUsed: "2h ago", status: "active" },
  { id: 3, name: "Old Staging Key", created: "Nov 15, 2023", lastUsed: "Never", status: "revoked" },
]

export default function Settings() {
  const { user } = useAuth()

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold dark:text-white">Settings</h1>
        <p className="text-zinc-600 dark:text-zinc-400 mt-2">Tune account, API, workspace defaults</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2"><Shield className="h-5 w-5" />Account</CardTitle>
          <CardDescription>Email, role, account management</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-3 gap-4">
            <div>
              <p className="text-xs text-zinc-600 dark:text-zinc-400 font-medium">Name</p>
              <p className="text-sm font-medium mt-1">{user?.name || "Unknown"}</p>
            </div>
            <div>
              <p className="text-xs text-zinc-600 dark:text-zinc-400 font-medium">Email</p>
              <p className="text-sm font-medium mt-1">{user?.email || "Unknown"}</p>
            </div>
            <div>
              <p className="text-xs text-zinc-600 dark:text-zinc-400 font-medium">Role</p>
              <p className="text-sm font-medium mt-1">{user?.is_admin ? "Admin" : "User"}</p>
            </div>
          </div>
          <div className="flex gap-2">
            <button className="px-3 py-2 rounded text-sm font-medium bg-orange-500 hover:bg-orange-600 text-white">
              Change Password
            </button>
            <button className="px-3 py-2 rounded text-sm font-medium bg-zinc-100 hover:bg-zinc-200 dark:bg-zinc-800 dark:hover:bg-zinc-700 text-zinc-900 dark:text-zinc-100">
              Edit Profile
            </button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2"><Lock className="h-5 w-5" />API Keys</CardTitle>
          <CardDescription>Manage authentication tokens</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {apiKeys.map((key) => (
              <div key={key.id} className="flex items-center justify-between p-3 rounded-lg border border-zinc-200 dark:border-zinc-800 hover:bg-zinc-50 dark:hover:bg-zinc-800/50">
                <div className="min-w-0 flex-1">
                  <p className="text-sm font-medium">{key.name}</p>
                  <p className="text-xs text-zinc-600 dark:text-zinc-400 mt-0.5">Created {key.created} • Last used {key.lastUsed}</p>
                </div>
                <span className={`px-2 py-1 rounded text-xs font-medium ${
                  key.status === "active"
                    ? "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
                    : "bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-400"
                }`}>
                  {key.status}
                </span>
                <div className="flex gap-1 ml-3">
                  <button className="p-2 hover:bg-zinc-100 dark:hover:bg-zinc-700 rounded">
                    <Eye className="h-4 w-4" />
                  </button>
                  <button className="p-2 hover:bg-zinc-100 dark:hover:bg-zinc-700 rounded">
                    <Copy className="h-4 w-4" />
                  </button>
                  <button className="p-2 hover:bg-red-100 dark:hover:bg-red-900/30 rounded text-red-600">
                    <Trash2 className="h-4 w-4" />
                  </button>
                </div>
              </div>
            ))}
          </div>
          <button className="mt-4 px-3 py-2 rounded text-sm font-medium bg-zinc-100 hover:bg-zinc-200 dark:bg-zinc-800 dark:hover:bg-zinc-700 text-zinc-900 dark:text-zinc-100 w-full">
            Generate New Key
          </button>
        </CardContent>
      </Card>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2"><Bell className="h-5 w-5" />Notifications</CardTitle>
            <CardDescription>Email + alert preferences</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <label className="flex items-center gap-3 cursor-pointer">
              <input type="checkbox" defaultChecked className="w-4 h-4 rounded" />
              <span className="text-sm">Security alerts</span>
            </label>
            <label className="flex items-center gap-3 cursor-pointer">
              <input type="checkbox" defaultChecked className="w-4 h-4 rounded" />
              <span className="text-sm">Weekly reports</span>
            </label>
            <label className="flex items-center gap-3 cursor-pointer">
              <input type="checkbox" className="w-4 h-4 rounded" />
              <span className="text-sm">Marketing emails</span>
            </label>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2"><Lock className="h-5 w-5" />Security</CardTitle>
            <CardDescription>Account protection settings</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex items-center justify-between p-2 rounded border border-zinc-200 dark:border-zinc-800">
              <span className="text-sm">Two-factor authentication</span>
              <span className="text-xs font-medium text-orange-600">Setup</span>
            </div>
            <div className="flex items-center justify-between p-2 rounded border border-zinc-200 dark:border-zinc-800">
              <span className="text-sm">Session timeout</span>
              <span className="text-xs text-zinc-600 dark:text-zinc-400">30 minutes</span>
            </div>
            <div className="flex items-center justify-between p-2 rounded border border-zinc-200 dark:border-zinc-800">
              <span className="text-sm">Active sessions</span>
              <span className="text-xs text-zinc-600 dark:text-zinc-400">1</span>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}