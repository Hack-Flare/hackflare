import { Plus, Network, Wifi, Eye } from "lucide-react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"
import { Button } from "~/components/ui/button"

const tunnels = [
  {
    id: 1,
    name: "Local Dev",
    target: "192.168.1.100:3000",
    status: "connected",
    uptime: "12h 34m",
    traffic: "2.3 GB",
  },
  {
    id: 2,
    name: "Preview Env",
    target: "staging.internal:443",
    status: "connected",
    uptime: "4d 2h",
    traffic: "156 MB",
  },
  {
    id: 3,
    name: "Webhooks",
    target: "webhook.internal:8000",
    status: "idle",
    uptime: "8d 15h",
    traffic: "45 MB",
  },
  {
    id: 4,
    name: "DB Backup",
    target: "db-primary.internal:5432",
    status: "connected",
    uptime: "2h 8m",
    traffic: "890 MB",
  },
]

export default function Tunnel() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold dark:text-white">Tunnel</h1>
          <p className="mt-2 text-zinc-600 dark:text-zinc-400">
            Manage secure tunnels into internal services
          </p>
        </div>
        <Button className="flex items-center gap-2 rounded-lg bg-orange-500 px-4 py-2 text-white hover:bg-orange-600">
          <Plus className="h-4 w-4" />
          New Tunnel
        </Button>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <Network className="h-4 w-4" />
              Active Tunnels
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">3</p>
            <p className="mt-1 text-xs text-green-600">All connected</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <Wifi className="h-4 w-4" />
              Healthy Endpoints
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">4</p>
            <p className="mt-1 text-xs text-green-600">0 disconnected</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <Eye className="h-4 w-4" />
              Data Transferred
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">3.4GB</p>
            <p className="mt-1 text-xs text-blue-600">Last 7 days</p>
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
                  <th className="px-4 py-3 text-left font-semibold">Name</th>
                  <th className="px-4 py-3 text-left font-semibold">Target</th>
                  <th className="px-4 py-3 text-center font-semibold">
                    Status
                  </th>
                  <th className="px-4 py-3 text-right font-semibold">Uptime</th>
                  <th className="px-4 py-3 text-right font-semibold">
                    Traffic
                  </th>
                </tr>
              </thead>
              <tbody>
                {tunnels.map((tunnel) => (
                  <tr
                    key={tunnel.id}
                    className="border-b border-zinc-100 hover:bg-zinc-50 dark:border-zinc-800 dark:hover:bg-zinc-800/50"
                  >
                    <td className="px-4 py-3 font-medium">{tunnel.name}</td>
                    <td className="px-4 py-3 font-mono text-xs text-zinc-600 dark:text-zinc-400">
                      {tunnel.target}
                    </td>
                    <td className="px-4 py-3 text-center">
                      <span
                        className={`rounded px-2 py-1 text-xs font-medium ${
                          tunnel.status === "connected"
                            ? "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
                            : "bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-400"
                        }`}
                      >
                        {tunnel.status}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-right text-zinc-600 dark:text-zinc-400">
                      {tunnel.uptime}
                    </td>
                    <td className="px-4 py-3 text-right text-zinc-600 dark:text-zinc-400">
                      {tunnel.traffic}
                    </td>
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
