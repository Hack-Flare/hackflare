import { Zap, TrendingUp, Activity, Gauge } from "lucide-react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card"

const regions = [
  { region: "US East", latency: 124, p95: 256, p99: 512 },
  { region: "US West", latency: 156, p95: 289, p99: 578 },
  { region: "EU", latency: 245, p95: 412, p99: 891 },
  { region: "APAC", latency: 312, p95: 567, p99: 1024 },
]

export default function Performance() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold dark:text-white">Performance</h1>
        <p className="text-zinc-600 dark:text-zinc-400 mt-2">Track latency, throughput + cache health</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Gauge className="h-4 w-4" />
              Avg Latency
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">142ms</p>
            <p className="text-xs text-green-600 mt-1">-8% vs yesterday</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Activity className="h-4 w-4" />
              Requests/sec
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">1.8K</p>
            <p className="text-xs text-blue-600 mt-1">Peak 4.2K</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <TrendingUp className="h-4 w-4" />
              Cache Hit Rate
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">87%</p>
            <p className="text-xs text-green-600 mt-1">+3% vs week ago</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Zap className="h-4 w-4" />
              P95 Latency
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">412ms</p>
            <p className="text-xs text-yellow-600 mt-1">95th percentile</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Latency by Region</CardTitle>
          <CardDescription>Response times from edge locations</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-zinc-200 dark:border-zinc-800">
                  <th className="text-left py-3 px-4 font-semibold">Region</th>
                  <th className="text-right py-3 px-4 font-semibold">Avg</th>
                  <th className="text-right py-3 px-4 font-semibold">P95</th>
                  <th className="text-right py-3 px-4 font-semibold">P99</th>
                </tr>
              </thead>
              <tbody>
                {regions.map((r, i) => (
                  <tr key={i} className="border-b border-zinc-100 dark:border-zinc-800 hover:bg-zinc-50 dark:hover:bg-zinc-800/50">
                    <td className="py-3 px-4 font-medium">{r.region}</td>
                    <td className="py-3 px-4 text-right">{r.latency}ms</td>
                    <td className="py-3 px-4 text-right text-zinc-600 dark:text-zinc-400">{r.p95}ms</td>
                    <td className="py-3 px-4 text-right text-zinc-600 dark:text-zinc-400">{r.p99}ms</td>
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