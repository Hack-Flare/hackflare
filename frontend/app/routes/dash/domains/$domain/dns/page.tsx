import { useParams } from "react-router"
import { useState } from "react"
import { Button } from "~/components/ui/button"
import { Plus, Globe, Zap, Activity } from "lucide-react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "~/components/ui/dialog"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "~/components/ui/select"
import { Input } from "~/components/ui/input"
import { Label } from "~/components/ui/label"
import { DataTable } from "./data-table"
import { columns, type DnsRecord } from "./columns"

const initialRecords: Record<string, DnsRecord[]> = {
  "example.com": [
    { id: 1, name: "@",               type: "A",     value: "192.0.2.1",                          ttl: 3600, status: "active"  },
    { id: 2, name: "www",             type: "CNAME", value: "example.com",                         ttl: 3600, status: "active"  },
    { id: 3, name: "@",               type: "MX",    value: "mail.example.com (10)",               ttl: 3600, status: "active"  },
    { id: 4, name: "_acme-challenge", type: "TXT",   value: "v=spf1 include:_spf.google.com ~all", ttl: 300,  status: "active"  },
    { id: 5, name: "api",             type: "A",     value: "192.0.2.10",                          ttl: 3600, status: "active"  },
    { id: 6, name: "cdn",             type: "CNAME", value: "d111111abcdef8.cloudfront.net",       ttl: 3600, status: "pending" },
  ],
  "hackclub.com": [
    { id: 1, name: "@",    type: "A",     value: "10.0.0.1",          ttl: 3600, status: "active" },
    { id: 2, name: "mail", type: "MX",    value: "mail.hackclub.com", ttl: 3600, status: "active" },
    { id: 3, name: "www",  type: "CNAME", value: "hackclub.com",      ttl: 3600, status: "active" },
  ],
  "mycoolsite.dev": [
    { id: 1, name: "@",   type: "A", value: "172.16.0.1", ttl: 300, status: "pending" },
    { id: 2, name: "www", type: "A", value: "172.16.0.1", ttl: 300, status: "pending" },
  ],
}

const defaultForm = {
  name: "",
  type: "A" as DnsRecord["type"],
  value: "",
  ttl: 3600,
}

export default function Dns() {
  const { domain } = useParams<{ domain: string }>()
  const [allRecords, setAllRecords] = useState(initialRecords)
  const [open, setOpen] = useState(false)
  const [form, setForm] = useState(defaultForm)

  const records = domain ? (allRecords[domain] ?? []) : []
  const aRecords     = records.filter((r) => r.type === "A")
  const cnameRecords = records.filter((r) => r.type === "CNAME")
  const otherRecords = records.filter((r) => r.type !== "A" && r.type !== "CNAME")

  const handleAdd = () => {
    if (!form.name || !form.value || !domain) return
    const newRecord: DnsRecord = {
      id: Date.now(),
      name: form.name,
      type: form.type,
      value: form.value,
      ttl: form.ttl,
      status: "pending",
    }
    setAllRecords((prev) => ({
      ...prev,
      [domain]: [...(prev[domain] ?? []), newRecord],
    }))
    setForm(defaultForm)
    setOpen(false)
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold dark:text-white">DNS Records</h1>
          <p className="mt-2 text-zinc-600 dark:text-zinc-400">
            Managing records for{" "}
            <span className="font-medium text-white">{domain}</span>
          </p>
        </div>

        <Dialog open={open} onOpenChange={setOpen}>
          <DialogTrigger asChild>
            <Button className="flex items-center gap-2 rounded-lg bg-orange-500 py-2 text-white hover:bg-orange-600">
              <Plus className="h-5 w-5" />
              Add Record
            </Button>
          </DialogTrigger>
          <DialogContent className="sm:max-w-md">
            <DialogHeader>
              <DialogTitle>Add DNS Record</DialogTitle>
              <DialogDescription>
                Add a new DNS record for <span className="text-white font-medium">{domain}</span>
              </DialogDescription>
            </DialogHeader>

            <div className="space-y-4 py-2">
              <div className="space-y-2">
                <Label>Type</Label>
                <Select
                  value={form.type}
                  onValueChange={(v) => setForm({ ...form, type: v as DnsRecord["type"] })}
                >
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {["A", "CNAME", "MX", "AAAA", "TXT", "NS"].map((t) => (
                      <SelectItem key={t} value={t}>{t}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-2">
                <Label>Name</Label>
                <Input
                  placeholder="@ or subdomain"
                  value={form.name}
                  onChange={(e) => setForm({ ...form, name: e.target.value })}
                />
              </div>

              <div className="space-y-2">
                <Label>Value</Label>
                <Input
                  placeholder={form.type === "A" ? "192.0.2.1" : form.type === "CNAME" ? "example.com" : ""}
                  value={form.value}
                  onChange={(e) => setForm({ ...form, value: e.target.value })}
                />
              </div>

              <div className="space-y-2">
                <Label>TTL (seconds)</Label>
                <Input
                  type="number"
                  value={form.ttl}
                  onChange={(e) => setForm({ ...form, ttl: Number(e.target.value) })}
                />
              </div>
            </div>

            <DialogFooter>
              <Button variant="outline" onClick={() => setOpen(false)}>Cancel</Button>
              <Button
                className="bg-orange-500 hover:bg-orange-600 text-white"
                onClick={handleAdd}
              >
                Add Record
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <Globe className="h-4 w-4" />A Records
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">{aRecords.length}</p>
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
            <p className="text-2xl font-bold">{cnameRecords.length}</p>
            <p className="mt-1 text-xs text-zinc-600 dark:text-zinc-400">Aliases</p>
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
            <p className="text-2xl font-bold">{otherRecords.length}</p>
            <p className="mt-1 text-xs text-blue-600">MX, TXT, etc</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>DNS Records</CardTitle>
          <CardDescription>Create records, point nameservers, verify zones</CardDescription>
        </CardHeader>
        <CardContent>
          <DataTable columns={columns} data={records} />
        </CardContent>
      </Card>
    </div>
  )
}