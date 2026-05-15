import { Button } from "~/components/ui/button"
import { Plus, Shield, AlertTriangle, Zap } from "lucide-react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"
import { DataTable } from "./data-table"
import { columns, type FirewallRule } from "./columns"

export const rules: FirewallRule[] = [
  { id: 1, type: "Blocklist",    pattern: "*.shadowban-ip.net",    action: "Block",     enabled: true,  requests: 245  },
  { id: 2, type: "Allowlist",    pattern: "192.168.1.0/24",        action: "Allow",     enabled: true,  requests: 8542 },
  { id: 3, type: "Rate Limit",   pattern: "/api/*",                action: "Limit",     enabled: true,  requests: 156  },
  { id: 4, type: "Bot Challenge",pattern: "*.googlebot.com",       action: "Challenge", enabled: false, requests: 0    },
  { id: 5, type: "Blocklist",    pattern: "User-Agent: badbot*",   action: "Block",     enabled: true,  requests: 89   },
]

export default function Firewall() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Firewall</h1>
          <p className="mt-2 text-zinc-600 dark:text-zinc-400">
            Block bad traffic + allow trusted sources
          </p>
        </div>
        <Button variant="orange">
          <Plus className="h-5 w-5" />
          Add Rule
        </Button>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <AlertTriangle className="h-4 w-4" />
              Blocked Requests
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">2.4K</p>
            <p className="mt-1 text-xs text-orange-600 dark:text-orange-500">Last 24 hours</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <Shield className="h-4 w-4" />
              Rules Active
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">{rules.filter((r) => r.enabled).length}</p>
            <p className="mt-1 text-xs text-green-600 dark:text-green-500">{rules.length} total</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <Zap className="h-4 w-4" />
              Challenge Rate
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">8.2%</p>
            <p className="mt-1 text-xs text-blue-600 dark:text-blue-500">Avg challenge success</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Security Rules</CardTitle>
          <CardDescription>Manage firewall rules, rate limits, bot challenges</CardDescription>
        </CardHeader>
        <CardContent>
          <DataTable columns={columns} data={rules} />
        </CardContent>
      </Card>
    </div>
  )
}