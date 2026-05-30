import { useParams } from "react-router"
import { useEffect, useState } from "react"
import { Button } from "~/components/ui/button"
import { Plus, Globe, Zap, Activity, Loader2, AlertCircle } from "lucide-react"
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
import { useColumns, type DnsRecord } from "./columns"
import { api } from "~/lib/api"

const defaultForm = {
  name: "",
  type: "A" as string,
  value: "",
  ttl: 3600,
}

export default function Dns() {
  const { domain } = useParams<{ domain: string }>()
  const [records, setRecords] = useState<DnsRecord[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [open, setOpen] = useState(false)
  const [adding, setAdding] = useState(false)
  const [addError, setAddError] = useState<string | null>(null)
  const [form, setForm] = useState(defaultForm)
  const [deleting, setDeleting] = useState<string | null>(null)

  const fetchRecords = async () => {
    if (!domain) return
    setLoading(true)
    setError(null)
    try {
      const data = await api.dns.listRecords(domain)
      setRecords(data)
    } catch (err) {
      const msg =
        err && typeof err === "object" && "error" in err
          ? String((err as { error: unknown }).error)
          : "Failed to load DNS records"
      setError(msg)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    void fetchRecords()
  }, [domain])

  const aRecords = records.filter((r) => r.type === "A")
  const cnameRecords = records.filter((r) => r.type === "CNAME")
  const otherRecords = records.filter(
    (r) => r.type !== "A" && r.type !== "CNAME"
  )

  const handleAdd = async () => {
    if (!form.name || !form.value || !domain) return
    setAdding(true)
    setAddError(null)
    try {
      await api.dns.createRecord(domain, {
        name: form.name,
        type: form.type,
        value: form.value,
        ttl: form.ttl,
      })
      setForm(defaultForm)
      setOpen(false)
      await fetchRecords()
    } catch (err) {
      const msg =
        err && typeof err === "object" && "error" in err
          ? String((err as { error: unknown }).error)
          : "Failed to add record"
      setAddError(msg)
    } finally {
      setAdding(false)
    }
  }

  const handleDelete = async (record: DnsRecord) => {
    if (!domain) return
    setDeleting(record.id)
    try {
      await api.dns.deleteRecord(domain, record.name, record.type)
      await fetchRecords()
    } catch (err) {
      const msg =
        err && typeof err === "object" && "error" in err
          ? String((err as { error: unknown }).error)
          : "Failed to delete record"
      setError(msg)
    } finally {
      setDeleting(null)
    }
  }

  const handleEdit = (record: DnsRecord) => {
    // TODO: implement edit dialog in a follow-up
    console.log("edit", record)
  }

  const columns = useColumns({ onDelete: handleDelete, onEdit: handleEdit })

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
                Add a new DNS record for{" "}
                <span className="font-medium text-white">{domain}</span>
              </DialogDescription>
            </DialogHeader>

            <div className="space-y-4 py-2">
              {addError && (
                <div className="rounded bg-red-100 px-3 py-2 text-sm text-red-800 dark:bg-red-900/30 dark:text-red-400">
                  {addError}
                </div>
              )}
              <div className="space-y-2">
                <Label>Type</Label>
                <Select
                  value={form.type}
                  onValueChange={(v) =>
                    setForm({ ...form, type: v })
                  }
                >
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {["A", "CNAME", "MX", "AAAA", "TXT", "NS"].map((t) => (
                      <SelectItem key={t} value={t}>
                        {t}
                      </SelectItem>
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
                  disabled={adding}
                />
              </div>

              <div className="space-y-2">
                <Label>Value</Label>
                <Input
                  placeholder={
                    form.type === "A"
                      ? "192.0.2.1"
                      : form.type === "CNAME"
                        ? "example.com"
                        : ""
                  }
                  value={form.value}
                  onChange={(e) => setForm({ ...form, value: e.target.value })}
                  disabled={adding}
                />
              </div>

              <div className="space-y-2">
                <Label>TTL (seconds)</Label>
                <Input
                  type="number"
                  value={form.ttl}
                  onChange={(e) =>
                    setForm({ ...form, ttl: Number(e.target.value) })
                  }
                  disabled={adding}
                />
              </div>
            </div>

            <DialogFooter>
              <Button variant="outline" onClick={() => setOpen(false)} disabled={adding}>
                Cancel
              </Button>
              <Button
                className="bg-orange-500 text-white hover:bg-orange-600"
                onClick={handleAdd}
                disabled={adding}
              >
                {adding ? "Adding..." : "Add Record"}
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
            <p className="text-2xl font-bold">{otherRecords.length}</p>
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
          {loading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-6 w-6 animate-spin text-zinc-500" />
            </div>
          ) : error ? (
            <div className="flex items-center justify-center gap-2 py-12 text-red-500">
              <AlertCircle className="h-5 w-5" />
              <span className="text-sm">{error}</span>
            </div>
          ) : (
            <DataTable columns={columns} data={records} />
          )}
        </CardContent>
      </Card>
    </div>
  )
}
