import { Plus, Zap, Clock, Upload } from "lucide-react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"
import { Button } from "~/components/ui/button"

const workers = [
  {
    id: 1,
    name: "Request Logger",
    type: "middleware",
    status: "active",
    deployments: 5,
    executions: "14.2K",
  },
  {
    id: 2,
    name: "Webhook Relay",
    type: "background",
    status: "active",
    deployments: 3,
    executions: "8.5K",
  },
  {
    id: 3,
    name: "Cache Warmer",
    type: "cron",
    status: "active",
    deployments: 2,
    executions: "240",
  },
  {
    id: 4,
    name: "Analytics Sync",
    type: "background",
    status: "paused",
    deployments: 1,
    executions: "0",
  },
  {
    id: 5,
    name: "Cleanup Job",
    type: "cron",
    status: "active",
    deployments: 4,
    executions: "98",
  },
]

export default function Workers() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold dark:text-white">Workers</h1>
          <p className="mt-2 text-zinc-600 dark:text-zinc-400">
            Deploy edge scripts + background jobs
          </p>
        </div>
        <Button className="flex items-center gap-2 rounded-lg bg-orange-500 px-4 py-2 text-white hover:bg-orange-600">
          <Plus className="h-4 w-4" />
          Deploy Worker
        </Button>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <Zap className="h-4 w-4" />
              Active Workers
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">4</p>
            <p className="mt-1 text-xs text-green-600">1 paused</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <Clock className="h-4 w-4" />
              Queued Jobs
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">8</p>
            <p className="mt-1 text-xs text-blue-600">Est. 2m runtime</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <Upload className="h-4 w-4" />
              Deploys Today
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">12</p>
            <p className="mt-1 text-xs text-purple-600">All successful</p>
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
                  <th className="px-4 py-3 text-left font-semibold">Name</th>
                  <th className="px-4 py-3 text-left font-semibold">Type</th>
                  <th className="px-4 py-3 text-center font-semibold">
                    Status
                  </th>
                  <th className="px-4 py-3 text-right font-semibold">
                    Deploys
                  </th>
                  <th className="px-4 py-3 text-right font-semibold">
                    Executions
                  </th>
                </tr>
              </thead>
              <tbody>
                {workers.map((worker) => (
                  <tr
                    key={worker.id}
                    className="border-b border-zinc-100 hover:bg-zinc-50 dark:border-zinc-800 dark:hover:bg-zinc-800/50"
                  >
                    <td className="px-4 py-3 font-medium">{worker.name}</td>
                    <td className="px-4 py-3">
                      <span className="rounded bg-zinc-100 px-2 py-1 text-xs font-medium text-zinc-800 dark:bg-zinc-800 dark:text-zinc-300">
                        {worker.type}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-center">
                      <span
                        className={`rounded px-2 py-1 text-xs font-medium ${
                          worker.status === "active"
                            ? "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
                            : "bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-400"
                        }`}
                      >
                        {worker.status}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-right text-zinc-600 dark:text-zinc-400">
                      {worker.deployments}
                    </td>
                    <td className="px-4 py-3 text-right text-zinc-600 dark:text-zinc-400">
                      {worker.executions}
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
