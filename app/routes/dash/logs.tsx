import { AlertTriangle, Info, CheckCircle, Filter } from "lucide-react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card"

const logs = [
  { id: 1, timestamp: "2024-01-15 14:32:10", level: "info", path: "/api/v1/auth/login", status: 200, ms: 145 },
  { id: 2, timestamp: "2024-01-15 14:31:45", level: "warning", path: "/missing-asset.js", status: 404, ms: 8 },
  { id: 3, timestamp: "2024-01-15 14:31:20", level: "error", path: "/api/v1/users", status: 500, ms: 1250 },
  { id: 4, timestamp: "2024-01-15 14:30:55", level: "info", path: "/static/css/main.css", status: 304, ms: 2 },
  { id: 5, timestamp: "2024-01-15 14:30:40", level: "error", path: "/api/v1/db/query", status: 503, ms: 5000 },
  { id: 6, timestamp: "2024-01-15 14:30:15", level: "warning", path: "/api/v1/search", status: 429, ms: 50 },
]

const levelColor = (level: string) => {
  switch(level) {
    case "error": return "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400"
    case "warning": return "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400"
    default: return "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
  }
}

const levelIcon = (level: string) => {
  switch(level) {
    case "error": return <AlertTriangle className="h-4 w-4" />
    case "warning": return <AlertTriangle className="h-4 w-4" />
    default: return <Info className="h-4 w-4" />
  }
}

export default function Logs() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold dark:text-white">Logs</h1>
          <p className="text-zinc-600 dark:text-zinc-400 mt-2">View recent events, errors + request history</p>
        </div>
        <button className="bg-orange-500 hover:bg-orange-600 text-white px-4 py-2 rounded-lg flex items-center gap-2">
          <Filter className="h-4 w-4" />
          Filter
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <AlertTriangle className="h-4 w-4" />
              Errors Today
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">7</p>
            <p className="text-xs text-red-600 mt-1">5xx errors</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <AlertTriangle className="h-4 w-4" />
              Warnings
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">14</p>
            <p className="text-xs text-yellow-600 mt-1">4xx + rate limits</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <CheckCircle className="h-4 w-4" />
              Info Events
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">1.2K</p>
            <p className="text-xs text-green-600 mt-1">Successful requests</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Recent Events</CardTitle>
          <CardDescription>Logs + errors from past 24 hours</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-zinc-200 dark:border-zinc-800">
                  <th className="text-left py-3 px-4 font-semibold">Time</th>
                  <th className="text-left py-3 px-4 font-semibold">Level</th>
                  <th className="text-left py-3 px-4 font-semibold">Path</th>
                  <th className="text-center py-3 px-4 font-semibold">Status</th>
                  <th className="text-right py-3 px-4 font-semibold">Duration</th>
                </tr>
              </thead>
              <tbody>
                {logs.map((log) => (
                  <tr key={log.id} className="border-b border-zinc-100 dark:border-zinc-800 hover:bg-zinc-50 dark:hover:bg-zinc-800/50">
                    <td className="py-3 px-4 text-xs text-zinc-600 dark:text-zinc-400">{log.timestamp}</td>
                    <td className="py-3 px-4">
                      <span className={`px-2 py-1 rounded text-xs font-medium flex items-center gap-1 w-fit ${levelColor(log.level)}`}>
                        {levelIcon(log.level)}
                        {log.level}
                      </span>
                    </td>
                    <td className="py-3 px-4 font-mono text-xs text-zinc-600 dark:text-zinc-400">{log.path}</td>
                    <td className="py-3 px-4 text-center">
                      <span className={`px-2 py-1 rounded text-xs font-medium ${
                        log.status < 300 ? "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
                        : log.status < 400 ? "bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400"
                        : log.status < 500 ? "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400"
                        : "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400"
                      }`}>
                        {log.status}
                      </span>
                    </td>
                    <td className="py-3 px-4 text-right text-zinc-600 dark:text-zinc-400">{log.ms}ms</td>
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