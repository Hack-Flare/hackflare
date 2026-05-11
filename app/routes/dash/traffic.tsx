"use client"

import * as React from "react"
import { Area, AreaChart, CartesianGrid, XAxis, YAxis, Bar, BarChart } from "recharts"

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

const trafficData = [
  { date: "2024-01-01", requests: 2200, errors: 40 },
  { date: "2024-01-02", requests: 1897, errors: 21 },
  { date: "2024-01-03", requests: 2800, errors: 29 },
  { date: "2024-01-04", requests: 3908, errors: 200 },
  { date: "2024-01-05", requests: 3800, errors: 221 },
  { date: "2024-01-06", requests: 2300, errors: 229 },
  { date: "2024-01-07", requests: 2300, errors: 200 },
  { date: "2024-01-08", requests: 3490, errors: 220 },
  { date: "2024-01-09", requests: 1590, errors: 110 },
  { date: "2024-01-10", requests: 2610, errors: 190 },
  { date: "2024-01-11", requests: 3270, errors: 350 },
  { date: "2024-01-12", requests: 2920, errors: 210 },
]

const statusCodeData = [
  { name: "200 OK", value: 4000, fill: "var(--color-ok)" },
  { name: "304 Not Modified", value: 640, fill: "var(--color-cached)" },
  { name: "4xx Errors", value: 150, fill: "var(--color-client-error)" },
  { name: "5xx Errors", value: 60, fill: "var(--color-server-error)" },
]

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

const statusConfig = {
  ok: {
    label: "200 OK",
    color: "hsl(120, 100%, 40%)",
  },
  cached: {
    label: "304 Not Modified",
    color: "hsl(200, 100%, 40%)",
  },
  clientError: {
    label: "4xx Errors",
    color: "hsl(120, 100%, 50%)",
  },
  serverError: {
    label: "5xx Errors",
    color: "hsl(0, 100%, 50%)",
  },
} satisfies ChartConfig

export default function Traffic() {
  return (
    <div className="flex-1 p-6">
      <div className="mb-8">
        <h1 className="text-3xl font-bold dark:text-white">Traffic</h1>
        <p className="text-zinc-600 dark:text-zinc-400 mt-2">Real-time traffic analytics and insights</p>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Total Requests</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">2.4M</p>
            <p className="text-xs text-green-600 mt-2">+12% vs last week</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Avg Response Time</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">142ms</p>
            <p className="text-xs text-green-600 mt-2">-3% vs last week</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Cache Hit Rate</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">87%</p>
            <p className="text-xs text-green-600 mt-2">+5% vs last week</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Error Rate</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">0.2%</p>
            <p className="text-xs text-green-600 mt-2">-0.1% vs last week</p>
          </CardContent>
        </Card>
      </div>

      {/* Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        {/* Traffic Chart */}
        <Card>
          <CardHeader>
            <CardTitle>Requests Over Time</CardTitle>
            <CardDescription>Total requests and errors</CardDescription>
          </CardHeader>
          <CardContent>
            <ChartContainer config={trafficConfig} className="h-[250px] w-full">
              <AreaChart data={trafficData}>
                <defs>
                  <linearGradient id="fillRequests" x1="0" y1="0" x2="0" y2="1">
                    <stop
                      offset="5%"
                      stopColor="var(--color-requests)"
                      stopOpacity={0.8}
                    />
                    <stop
                      offset="95%"
                      stopColor="var(--color-requests)"
                      stopOpacity={0.1}
                    />
                  </linearGradient>
                  <linearGradient id="fillErrors" x1="0" y1="0" x2="0" y2="1">
                    <stop
                      offset="5%"
                      stopColor="var(--color-errors)"
                      stopOpacity={0.8}
                    />
                    <stop
                      offset="95%"
                      stopColor="var(--color-errors)"
                      stopOpacity={0.1}
                    />
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
                    return date.toLocaleDateString("en-US", {
                      month: "short",
                      day: "numeric",
                    })
                  }}
                />
                <YAxis tickLine={false} axisLine={false} />
                <ChartTooltip
                  cursor={false}
                  content={
                    <ChartTooltipContent
                      labelFormatter={(value) => {
                        return new Date(value).toLocaleDateString("en-US", {
                          month: "short",
                          day: "numeric",
                        })
                      }}
                      indicator="dot"
                    />
                  }
                />
                <Area
                  dataKey="errors"
                  type="natural"
                  fill="url(#fillErrors)"
                  stroke="var(--color-errors)"
                  stackId="a"
                />
                <Area
                  dataKey="requests"
                  type="natural"
                  fill="url(#fillRequests)"
                  stroke="var(--color-requests)"
                  stackId="a"
                />
                <ChartLegend content={<ChartLegendContent />} />
              </AreaChart>
            </ChartContainer>
          </CardContent>
        </Card>

        {/* Status Codes */}
        <Card>
          <CardHeader>
            <CardTitle>Status Code Distribution</CardTitle>
            <CardDescription>HTTP response codes</CardDescription>
          </CardHeader>
          <CardContent>
            <ChartContainer config={statusConfig} className="h-[250px] w-full">
              <BarChart data={statusCodeData}>
                <CartesianGrid vertical={false} />
                <XAxis
                  dataKey="name"
                  tickLine={false}
                  tickMargin={10}
                  axisLine={false}
                  tickFormatter={(value) => value.slice(0, 10)}
                />
                <YAxis tickLine={false} axisLine={false} />
                <ChartTooltip
                  cursor={false}
                  content={<ChartTooltipContent hideLabel />}
                />
                <Bar
                  dataKey="value"
                  fill="var(--color-ok)"
                  radius={4}
                />
              </BarChart>
            </ChartContainer>
          </CardContent>
        </Card>
      </div>

      {/* Top Paths */}
      <Card>
        <CardHeader>
          <CardTitle>Top Paths</CardTitle>
          <CardDescription>Most requested endpoints</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-zinc-200 dark:border-zinc-800">
                  <th className="text-left py-2 px-4 dark:text-zinc-400">Path</th>
                  <th className="text-right py-2 px-4 dark:text-zinc-400">Requests</th>
                  <th className="text-right py-2 px-4 dark:text-zinc-400">Avg Response Time</th>
                  <th className="text-right py-2 px-4 dark:text-zinc-400">Status</th>
                </tr>
              </thead>
              <tbody>
                {[
                  { path: "/", requests: "542K", time: "124ms", status: "✓" },
                  { path: "/api/users", requests: "321K", time: "156ms", status: "✓" },
                  { path: "/dashboard", requests: "289K", time: "145ms", status: "✓" },
                  { path: "/settings", requests: "156K", time: "138ms", status: "✓" },
                ].map((row, i) => (
                  <tr key={i} className="border-b border-zinc-100 dark:border-zinc-800">
                    <td className="py-3 px-4 dark:text-white font-mono text-xs">{row.path}</td>
                    <td className="py-3 px-4 text-right dark:text-zinc-300">{row.requests}</td>
                    <td className="py-3 px-4 text-right dark:text-zinc-300">{row.time}</td>
                    <td className="py-3 px-4 text-right dark:text-green-400">{row.status}</td>
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