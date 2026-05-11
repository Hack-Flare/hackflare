import { Plus, Zap, Clock, Upload } from "lucide-react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card"

const workers = [
  { id: 1, name: "Request Logger", type: "middleware", status: "active", deployments: 5, executions: "14.2K" },
  { id: 2, name: "Webhook Relay", type: "background", status: "active", deployments: 3, executions: "8.5K" },
  { id: 3, name: "Cache Warmer", type: "cron", status: "active", deployments: 2, executions: "240" },
  { id: 4, name: "Analytics Sync", type: "background", status: "paused", deployments: 1, executions: "0" },
  { id: 5, name: "Cleanup Job", type: "cron", status: "active", deployments: 4, executions: "98" },
]

export default function Workers() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold dark:text-white">Workers</h1>
          <p className="text-zinc-600 dark:text-zinc-400 mt-2">Deploy edge scripts + background jobs</p>
        </div>
        <button className="bg-orange-500 hover:bg-orange-600 text-white px-4 py-2 rounded-lg flex items-center gap-2">
          <Plus className="h-4 w-4" />
          Deploy Worker
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Zap className="h-4 w-4" />
              Active Workers
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">4</p>
            <p className="text-xs text-green-600 mt-1">1 paused</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Clock className="h-4 w-4" />
              Queued Jobs
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">8</p>
            <p className="text-xs text-blue-600 mt-1">Est. 2m runtime</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Upload className="h-4 w-4" />
              Deploys Today
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">12</p>
            <p className="text-xs text-purple-600 mt-1">All successful</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Worker Scripts</CardTitle>
          <CardDescription>Edge + background job deployments</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-zinc-200 dark:border-zinc-800">
                  <th className="text-left py-3 px-4 font-semibold">Name</th>
                  <th className="text-left py-3 px-4 font-semibold">Type</th>
                  <th className="text-center py-3 px-4 font-semibold">Status</th>
                  <th className="text-right py-3 px-4 font-semibold">Deploys</th>
                  <th className="text-right py-3 px-4 font-semibold">Executions</th>
                </tr>
              </thead>
              <tbody>
                {workers.map((worker) => (
                  <tr key={worker.id} className="border-b border-zinc-100 dark:border-zinc-800 hover:bg-zinc-50 dark:hover:bg-zinc-800/50">
                    <td className="py-3 px-4 font-medium">{worker.name}</td>
                    <td className="py-3 px-4">
                      <span className="px-2 py-1 rounded text-xs font-medium bg-zinc-100 dark:bg-zinc-800 text-zinc-800 dark:text-zinc-300">
                        {worker.type}
                      </span>
                    </td>
                    <td className="py-3 px-4 text-center">
                      <span className={`px-2 py-1 rounded text-xs font-medium ${
                        worker.status === "active"
                          ? "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
                          : "bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-400"
                      }`}>
                        {worker.status}
                      </span>
                    </td>
                    <td className="py-3 px-4 text-right text-zinc-600 dark:text-zinc-400">{worker.deployments}</td>
                    <td className="py-3 px-4 text-right text-zinc-600 dark:text-zinc-400">{worker.executions}</td>
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