import * as React from "react"
import {
  Area,
  AreaChart,
  CartesianGrid,
  XAxis,
  YAxis,
  Bar,
  BarChart,
  Cell,
} from "recharts"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"
import {
  ChartContainer,
  ChartLegend,
  ChartLegendContent,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from "~/components/ui/chart"
import { api, type TrafficSummary } from "~/lib/api"
import { Loader2 } from "lucide-react"

const trafficConfig = {
  requests: {
    label: "Requests",
    color: "hsl(200, 100%, 50%)",
  },
  errors: {
    label: "Errors",
    color: "hsl(0, 100%, 50%)",
  },
} satisfies ChartConfig

const statusColors = [
  "hsl(120, 100%, 40%)",
  "hsl(40, 100%, 50%)",
  "hsl(0, 100%, 50%)",
  "hsl(240, 100%, 60%)",
]

export default function Traffic() {
  const [summary, setSummary] = React.useState<TrafficSummary | null>(null)
  const [timeseries, setTimeseries] = React.useState<
    { date: string; requests: number; errors: number; nxdomain: number }[]
  >([])
  const [byZone, setByZone] = React.useState<
    { zone: string; requests: number; errors: number; avg_ms: number }[]
  >([])
  const [topQueries, setTopQueries] = React.useState<
    { query: string; count: number }[]
  >([])
  const [loading, setLoading] = React.useState(true)
  const [error, setError] = React.useState<string | null>(null)

  React.useEffect(() => {
    Promise.all([
      api.traffic.summary(),
      api.traffic.timeseries(30),
      api.traffic.byZone(),
      api.traffic.topQueries(5),
    ])
      .then(([s, ts, bz, tq]) => {
        setSummary(s)
        setTimeseries(ts)
        setByZone(bz)
        setTopQueries(tq)
      })
      .catch((err) => {
        setError(err && typeof err === "object" && "error" in err
          ? String((err as { error: unknown }).error)
          : "Failed to load traffic data")
      })
      .finally(() => setLoading(false))
  }, [])

  if (loading) {
    return (
      <div className="flex items-center justify-center py-24">
        <Loader2 className="h-8 w-8 animate-spin text-zinc-500" />
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex items-center justify-center gap-2 py-24 text-red-500">
        <p className="text-sm">{error}</p>
      </div>
    )
  }

  const statusCodeData = [
    { name: "NOERROR", value: byZone.reduce((a, b) => a + (b.requests - b.errors), 0), fill: statusColors[0] },
    { name: "NXDOMAIN", value: timeseries.reduce((a, b) => a + b.nxdomain, 0), fill: statusColors[1] },
    { name: "Errors", value: timeseries.reduce((a, b) => a + b.errors, 0), fill: statusColors[2] },
  ]

  return (
    <div className="space-y-6">
      <div className="mb-8">
        <h1 className="text-3xl font-bold dark:text-white">Traffic</h1>
        <p className="mt-2 text-zinc-600 dark:text-zinc-400">
          DNS query analytics for your domains
        </p>
      </div>

      {summary && (
        <div className="mb-8 grid grid-cols-1 gap-4 md:grid-cols-4">
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium">Total Queries</CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-2xl font-bold">{summary.total_requests.toLocaleString()}</p>
              <p className="mt-2 text-xs text-zinc-500">All time</p>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium">Avg Processing Time</CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-2xl font-bold">{summary.avg_processing_ms.toFixed(1)}ms</p>
              <p className="mt-2 text-xs text-zinc-500">Per query</p>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium">Success Rate</CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-2xl font-bold text-green-600">{summary.success_rate.toFixed(1)}%</p>
              <p className="mt-2 text-xs text-zinc-500">NOERROR responses</p>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium">Error Rate</CardTitle>
            </CardHeader>
            <CardContent>
              <p className={`text-2xl font-bold ${summary.error_rate > 1 ? "text-red-600" : "text-zinc-900 dark:text-white"}`}>
                {summary.error_rate.toFixed(2)}%
              </p>
              <p className="mt-2 text-xs text-zinc-500">SERVFAIL + other errors</p>
            </CardContent>
          </Card>
        </div>
      )}

      <div className="mb-6 grid grid-cols-1 gap-6 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Queries Over Time</CardTitle>
            <CardDescription>DNS queries over the last 30 days</CardDescription>
          </CardHeader>
          <CardContent>
            <ChartContainer config={trafficConfig} className="h-62.5 w-full">
              <AreaChart data={timeseries}>
                <defs>
                  <linearGradient id="fillRequests" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="var(--color-requests)" stopOpacity={0.8} />
                    <stop offset="95%" stopColor="var(--color-requests)" stopOpacity={0.1} />
                  </linearGradient>
                  <linearGradient id="fillErrors" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="var(--color-errors)" stopOpacity={0.8} />
                    <stop offset="95%" stopColor="var(--color-errors)" stopOpacity={0.1} />
                  </linearGradient>
                </defs>
                <CartesianGrid vertical={false} />
                <XAxis
                  dataKey="date"
                  tickLine={false}
                  axisLine={false}
                  tickMargin={8}
                  tickFormatter={(value) => {
                    const date = new Date(value)
                    return date.toLocaleDateString("en-US", { month: "short", day: "numeric" })
                  }}
                />
                <YAxis tickLine={false} axisLine={false} />
                <ChartTooltip
                  cursor={false}
                  content={
                    <ChartTooltipContent
                      labelFormatter={(value) => new Date(value).toLocaleDateString("en-US", { month: "short", day: "numeric" })}
                      indicator="dot"
                    />
                  }
                />
                <Area dataKey="errors" type="natural" fill="url(#fillErrors)" stroke="var(--color-errors)" stackId="a" />
                <Area dataKey="requests" type="natural" fill="url(#fillRequests)" stroke="var(--color-requests)" stackId="a" />
                <ChartLegend content={<ChartLegendContent />} />
              </AreaChart>
            </ChartContainer>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Response Code Distribution</CardTitle>
            <CardDescription>DNS response codes</CardDescription>
          </CardHeader>
          <CardContent>
            <ChartContainer config={trafficConfig} className="h-62.5 w-full">
              <BarChart data={statusCodeData}>
                <CartesianGrid vertical={false} />
                <XAxis dataKey="name" tickLine={false} tickMargin={10} axisLine={false} />
                <YAxis tickLine={false} axisLine={false} />
                <ChartTooltip cursor={false} content={<ChartTooltipContent hideLabel />} />
                <Bar dataKey="value" radius={4}>
                  {statusCodeData.map((entry, i) => (
                    <Cell key={entry.name} fill={entry.fill} />
                  ))}
                </Bar>
              </BarChart>
            </ChartContainer>
          </CardContent>
        </Card>
      </div>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Per-Zone Traffic</CardTitle>
            <CardDescription>Query count by domain</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-zinc-200 dark:border-zinc-800">
                    <th className="px-4 py-2 text-left dark:text-zinc-400">Zone</th>
                    <th className="px-4 py-2 text-right dark:text-zinc-400">Requests</th>
                    <th className="px-4 py-2 text-right dark:text-zinc-400">Errors</th>
                    <th className="px-4 py-2 text-right dark:text-zinc-400">Avg</th>
                  </tr>
                </thead>
                <tbody>
                  {byZone.length === 0 ? (
                    <tr>
                      <td colSpan={4} className="py-8 text-center text-zinc-500">No traffic data yet</td>
                    </tr>
                  ) : (
                    byZone.map((z) => (
                      <tr key={z.zone} className="border-b border-zinc-100 dark:border-zinc-800">
                        <td className="px-4 py-3 font-mono text-xs dark:text-white">{z.zone}</td>
                        <td className="px-4 py-3 text-right dark:text-zinc-300">{z.requests.toLocaleString()}</td>
                        <td className="px-4 py-3 text-right text-red-500">{z.errors}</td>
                        <td className="px-4 py-3 text-right dark:text-zinc-300">{z.avg_ms.toFixed(1)}ms</td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Top Queries</CardTitle>
            <CardDescription>Most frequently queried names</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-zinc-200 dark:border-zinc-800">
                    <th className="px-4 py-2 text-left dark:text-zinc-400">Query</th>
                    <th className="px-4 py-2 text-right dark:text-zinc-400">Requests</th>
                  </tr>
                </thead>
                <tbody>
                  {topQueries.length === 0 ? (
                    <tr>
                      <td colSpan={2} className="py-8 text-center text-zinc-500">No query data yet</td>
                    </tr>
                  ) : (
                    topQueries.map((q, i) => (
                      <tr key={i} className="border-b border-zinc-100 dark:border-zinc-800">
                        <td className="px-4 py-3 font-mono text-xs dark:text-white">{q.query}</td>
                        <td className="px-4 py-3 text-right dark:text-zinc-300">{q.count.toLocaleString()}</td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
