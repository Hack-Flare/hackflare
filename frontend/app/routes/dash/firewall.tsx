import { Plus, Shield, AlertTriangle, Zap } from "lucide-react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card"

const rules = [
  { id: 1, type: "Blocklist", pattern: "*.shadowban-ip.net", action: "Block", enabled: true, requests: 245 },
  { id: 2, type: "Allowlist", pattern: "192.168.1.0/24", action: "Allow", enabled: true, requests: 8542 },
  { id: 3, type: "Rate Limit", pattern: "/api/*", action: "Limit", enabled: true, requests: 156 },
  { id: 4, type: "Bot Challenge", pattern: "*.googlebot.com", action: "Challenge", enabled: false, requests: 0 },
  { id: 5, type: "Blocklist", pattern: "User-Agent: badbot*", action: "Block", enabled: true, requests: 89 },
]

export default function Firewall() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold dark:text-white">Firewall</h1>
          <p className="text-zinc-600 dark:text-zinc-400 mt-2">Block bad traffic + allow trusted sources</p>
        </div>
        <button className="bg-orange-500 hover:bg-orange-600 text-white px-4 py-2 rounded-lg flex items-center gap-2">
          <Plus className="h-4 w-4" />
          Add Rule
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <AlertTriangle className="h-4 w-4" />
              Blocked Requests
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">2.4K</p>
            <p className="text-xs text-orange-600 mt-1">Last 24 hours</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Shield className="h-4 w-4" />
              Rules Active
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">{rules.filter(r => r.enabled).length}</p>
            <p className="text-xs text-green-600 mt-1">{rules.length} total</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Zap className="h-4 w-4" />
              Challenge Rate
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">8.2%</p>
            <p className="text-xs text-blue-600 mt-1">Avg challenge success</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Security Rules</CardTitle>
          <CardDescription>Manage firewall rules, rate limits, bot challenges</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-zinc-200 dark:border-zinc-800">
                  <th className="text-left py-3 px-4 font-semibold">Type</th>
                  <th className="text-left py-3 px-4 font-semibold">Pattern</th>
                  <th className="text-left py-3 px-4 font-semibold">Action</th>
                  <th className="text-center py-3 px-4 font-semibold">Status</th>
                  <th className="text-right py-3 px-4 font-semibold">Requests</th>
                </tr>
              </thead>
              <tbody>
                {rules.map((rule) => (
                  <tr key={rule.id} className="border-b border-zinc-100 dark:border-zinc-800 hover:bg-zinc-50 dark:hover:bg-zinc-800/50">
                    <td className="py-3 px-4 font-medium">{rule.type}</td>
                    <td className="py-3 px-4 font-mono text-xs text-zinc-600 dark:text-zinc-400">{rule.pattern}</td>
                    <td className="py-3 px-4">
                      <span className="px-2 py-1 rounded text-xs font-medium bg-zinc-100 dark:bg-zinc-800 text-zinc-800 dark:text-zinc-300">
                        {rule.action}
                      </span>
                    </td>
                    <td className="py-3 px-4 text-center">
                      <span className={`inline-flex h-2 w-2 rounded-full ${rule.enabled ? "bg-green-500" : "bg-zinc-300"}`} />
                    </td>
                    <td className="py-3 px-4 text-right text-zinc-600 dark:text-zinc-400">{rule.requests.toLocaleString()}</td>
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