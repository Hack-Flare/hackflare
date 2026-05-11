import { Plus, Globe, Shield, Clock } from "lucide-react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"

const domains = [
  {
    id: "1",
    name: "example.com",
    status: "active",
    registrar: "Namecheap",
    dnsProvider: "Cloudflare",
    expiresAt: "2025-06-15",
    ssl: "valid",
  },
  {
    id: "2",
    name: "blog.example.com",
    status: "active",
    registrar: "GoDaddy",
    dnsProvider: "Route 53",
    expiresAt: "2025-08-22",
    ssl: "valid",
  },
  {
    id: "3",
    name: "api.example.com",
    status: "active",
    registrar: "Namecheap",
    dnsProvider: "Cloudflare",
    expiresAt: "2025-10-10",
    ssl: "expiring",
  },
]

export default function Domains() {
  return (
    <div className="flex-1 p-6">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold dark:text-white">Domains</h1>
          <p className="text-zinc-600 dark:text-zinc-400 mt-2">
            Manage and monitor your domains
          </p>
        </div>
        <button className="bg-orange-500 hover:bg-orange-600 text-white px-4 py-2 rounded-lg flex items-center gap-2">
          <Plus className="h-4 w-4" />
          Add Domain
        </button>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Globe className="h-4 w-4" />
              Total Domains
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">3</p>
            <p className="text-xs text-zinc-600 dark:text-zinc-400 mt-1">
              All active
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Shield className="h-4 w-4" />
              SSL Certificates
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">2</p>
            <p className="text-xs text-green-600 mt-1">Valid</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Clock className="h-4 w-4" />
              Renewal Soon
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">1</p>
            <p className="text-xs text-orange-600 mt-1">Next 90 days</p>
          </CardContent>
        </Card>
      </div>

      {/* Domains List */}
      <Card>
        <CardHeader>
          <CardTitle>Your Domains</CardTitle>
          <CardDescription>
            Complete overview of all your registered domains
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-zinc-200 dark:border-zinc-800">
                  <th className="text-left py-3 px-4 font-semibold">Domain</th>
                  <th className="text-left py-3 px-4 font-semibold">Registrar</th>
                  <th className="text-left py-3 px-4 font-semibold">DNS</th>
                  <th className="text-left py-3 px-4 font-semibold">SSL</th>
                  <th className="text-left py-3 px-4 font-semibold">Expires</th>
                  <th className="text-left py-3 px-4 font-semibold">Status</th>
                </tr>
              </thead>
              <tbody>
                {domains.map((domain) => (
                  <tr
                    key={domain.id}
                    className="border-b border-zinc-100 dark:border-zinc-800 hover:bg-zinc-50 dark:hover:bg-zinc-800/50"
                  >
                    <td className="py-3 px-4">
                      <div className="font-medium dark:text-white">
                        {domain.name}
                      </div>
                    </td>
                    <td className="py-3 px-4 text-zinc-600 dark:text-zinc-400">
                      {domain.registrar}
                    </td>
                    <td className="py-3 px-4 text-zinc-600 dark:text-zinc-400">
                      {domain.dnsProvider}
                    </td>
                    <td className="py-3 px-4">
                      <span
                        className={`px-2 py-1 rounded text-xs font-medium ${
                          domain.ssl === "valid"
                            ? "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
                            : "bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400"
                        }`}
                      >
                        {domain.ssl === "valid" ? "Valid" : "Expiring"}
                      </span>
                    </td>
                    <td className="py-3 px-4 text-zinc-600 dark:text-zinc-400">
                      {domain.expiresAt}
                    </td>
                    <td className="py-3 px-4">
                      <span className="px-2 py-1 rounded text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400">
                        Active
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>

      {/* Quick Links */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mt-8">
        <Card className="cursor-pointer hover:border-orange-500 hover:shadow-md transition-all">
          <CardHeader>
            <CardTitle className="text-base">DNS Records</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-zinc-600 dark:text-zinc-400">
              Manage DNS records, MX, CNAME, and A records
            </p>
          </CardContent>
        </Card>

        <Card className="cursor-pointer hover:border-orange-500 hover:shadow-md transition-all">
          <CardHeader>
            <CardTitle className="text-base">SSL Certificates</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-zinc-600 dark:text-zinc-400">
              View and manage SSL/TLS certificates
            </p>
          </CardContent>
        </Card>

        <Card className="cursor-pointer hover:border-orange-500 hover:shadow-md transition-all">
          <CardHeader>
            <CardTitle className="text-base">Domain Settings</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-zinc-600 dark:text-zinc-400">
              Configure domain forwarding and redirects
            </p>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}