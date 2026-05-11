import { Plus, Network, Wifi, Eye } from "lucide-react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card"

const tunnels = [
  { id: 1, name: "Local Dev", target: "192.168.1.100:3000", status: "connected", uptime: "12h 34m", traffic: "2.3 GB" },
  { id: 2, name: "Preview Env", target: "staging.internal:443", status: "connected", uptime: "4d 2h", traffic: "156 MB" },
  { id: 3, name: "Webhooks", target: "webhook.internal:8000", status: "idle", uptime: "8d 15h", traffic: "45 MB" },
  { id: 4, name: "DB Backup", target: "db-primary.internal:5432", status: "connected", uptime: "2h 8m", traffic: "890 MB" },
]

export default function Tunnel() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold dark:text-white">Tunnel</h1>
          <p className="text-zinc-600 dark:text-zinc-400 mt-2">Manage secure tunnels into internal services</p>
        </div>
        <button className="bg-orange-500 hover:bg-orange-600 text-white px-4 py-2 rounded-lg flex items-center gap-2">
          <Plus className="h-4 w-4" />
          New Tunnel
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Network className="h-4 w-4" />
              Active Tunnels
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">3</p>
            <p className="text-xs text-green-600 mt-1">All connected</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Wifi className="h-4 w-4" />
              Healthy Endpoints
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">4</p>
            <p className="text-xs text-green-600 mt-1">0 disconnected</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Eye className="h-4 w-4" />
              Data Transferred
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">3.4GB</p>
            <p className="text-xs text-blue-600 mt-1">Last 7 days</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Tunnel Connections</CardTitle>
          <CardDescription>Active + idle tunnel endpoints</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-zinc-200 dark:border-zinc-800">
                  <th className="text-left py-3 px-4 font-semibold">Name</th>
                  <th className="text-left py-3 px-4 font-semibold">Target</th>
                  <th className="text-center py-3 px-4 font-semibold">Status</th>
                  <th className="text-right py-3 px-4 font-semibold">Uptime</th>
                  <th className="text-right py-3 px-4 font-semibold">Traffic</th>
                </tr>
              </thead>
              <tbody>
                {tunnels.map((tunnel) => (
                  <tr key={tunnel.id} className="border-b border-zinc-100 dark:border-zinc-800 hover:bg-zinc-50 dark:hover:bg-zinc-800/50">
                    <td className="py-3 px-4 font-medium">{tunnel.name}</td>
                    <td className="py-3 px-4 font-mono text-xs text-zinc-600 dark:text-zinc-400">{tunnel.target}</td>
                    <td className="py-3 px-4 text-center">
                      <span className={`px-2 py-1 rounded text-xs font-medium ${
                        tunnel.status === "connected"
                          ? "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
                          : "bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-400"
                      }`}>
                        {tunnel.status}
                      </span>
                    </td>
                    <td className="py-3 px-4 text-right text-zinc-600 dark:text-zinc-400">{tunnel.uptime}</td>
                    <td className="py-3 px-4 text-right text-zinc-600 dark:text-zinc-400">{tunnel.traffic}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}