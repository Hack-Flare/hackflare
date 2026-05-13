import { Plus, Globe, Zap, Activity } from "lucide-react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card"

const records = [
  { id: 1, name: "@", type: "A", value: "192.0.2.1", ttl: 3600, status: "active" },
  { id: 2, name: "www", type: "CNAME", value: "example.com", ttl: 3600, status: "active" },
  { id: 3, name: "@", type: "MX", value: "mail.example.com (10)", ttl: 3600, status: "active" },
  { id: 4, name: "_acme-challenge", type: "TXT", value: "v=spf1 include:_spf.google.com ~all", ttl: 300, status: "active" },
  { id: 5, name: "api", type: "A", value: "192.0.2.10", ttl: 3600, status: "active" },
  { id: 6, name: "cdn", type: "CNAME", value: "d111111abcdef8.cloudfront.net", ttl: 3600, status: "pending" },
]

export default function Dns() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold dark:text-white">DNS</h1>
          <p className="text-zinc-600 dark:text-zinc-400 mt-2">Manage DNS zones + configure records</p>
        </div>
        <button className="bg-orange-500 hover:bg-orange-600 text-white px-4 py-2 rounded-lg flex items-center gap-2">
          <Plus className="h-4 w-4" />
          Add Record
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Globe className="h-4 w-4" />
              A Records
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">6</p>
            <p className="text-xs text-green-600 mt-1">Root + subdomains</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Zap className="h-4 w-4" />
              CNAME Records
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">3</p>
            <p className="text-xs text-zinc-600 dark:text-zinc-400 mt-1">Aliases</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Activity className="h-4 w-4" />
              Other Records
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">5</p>
            <p className="text-xs text-blue-600 mt-1">MX, TXT, etc</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>DNS Records</CardTitle>
          <CardDescription>Create records, point nameservers, verify zones</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-zinc-200 dark:border-zinc-800">
                  <th className="text-left py-3 px-4 font-semibold">Name</th>
                  <th className="text-left py-3 px-4 font-semibold">Type</th>
                  <th className="text-left py-3 px-4 font-semibold">Value</th>
                  <th className="text-center py-3 px-4 font-semibold">TTL</th>
                  <th className="text-center py-3 px-4 font-semibold">Status</th>
                </tr>
              </thead>
              <tbody>
                {records.map((record) => (
                  <tr key={record.id} className="border-b border-zinc-100 dark:border-zinc-800 hover:bg-zinc-50 dark:hover:bg-zinc-800/50">
                    <td className="py-3 px-4 font-medium">{record.name}</td>
                    <td className="py-3 px-4">
                      <span className="px-2 py-1 rounded text-xs font-mono bg-zinc-100 dark:bg-zinc-800 text-zinc-800 dark:text-zinc-300">
                        {record.type}
                      </span>
                    </td>
                    <td className="py-3 px-4 text-zinc-600 dark:text-zinc-400 font-mono text-xs">{record.value}</td>
                    <td className="py-3 px-4 text-center text-zinc-600 dark:text-zinc-400">{record.ttl}s</td>
                    <td className="py-3 px-4 text-center">
                      <span className={`px-2 py-1 rounded text-xs font-medium ${
                        record.status === "active"
                          ? "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
                          : "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400"
                      }`}>
                        {record.status}
                      </span>
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