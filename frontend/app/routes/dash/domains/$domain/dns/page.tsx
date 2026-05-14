import { Button } from "~/components/ui/button"
import { Plus, Globe, Zap, Activity } from "lucide-react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"
import { DataTable } from "./data-table"
import { columns, type DnsRecord } from "./columns"

export const records: DnsRecord[] = [
  {
    id: 1,
    name: "@",
    type: "A",
    value: "192.0.2.1",
    ttl: 3600,
    status: "active",
  },
  {
    id: 2,
    name: "www",
    type: "CNAME",
    value: "example.com",
    ttl: 3600,
    status: "active",
  },
  {
    id: 3,
    name: "@",
    type: "MX",
    value: "mail.example.com (10)",
    ttl: 3600,
    status: "active",
  },
  {
    id: 4,
    name: "_acme-challenge",
    type: "TXT",
    value: "v=spf1 include:_spf.google.com ~all",
    ttl: 300,
    status: "active",
  },
  {
    id: 5,
    name: "api",
    type: "A",
    value: "192.0.2.10",
    ttl: 3600,
    status: "active",
  },
  {
    id: 6,
    name: "cdn",
    type: "CNAME",
    value: "d111111abcdef8.cloudfront.net",
    ttl: 3600,
    status: "pending",
  },
]

export default function Dns() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold dark:text-white">DNS</h1>
          <p className="mt-2 text-zinc-600 dark:text-zinc-400">
            Manage DNS zones + configure records
          </p>
        </div>
        <Button className="flex items-center gap-2 rounded-lg bg-orange-500 py-2 text-white hover:bg-orange-600">
          <Plus className="h-5 w-5" />
          Add Record
        </Button>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <Globe className="h-4 w-4" />A Records
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">6</p>
            <p className="mt-1 text-xs text-green-600">Root + subdomains</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <Zap className="h-4 w-4" />
              CNAME Records
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">3</p>
            <p className="mt-1 text-xs text-zinc-600 dark:text-zinc-400">
              Aliases
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <Activity className="h-4 w-4" />
              Other Records
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">5</p>
            <p className="mt-1 text-xs text-blue-600">MX, TXT, etc</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>DNS Records</CardTitle>
          <CardDescription>
            Create records, point nameservers, verify zones
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <DataTable columns={columns} data={records} />
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
