import { Link, useParams, useNavigate } from "react-router"
import { useEffect, useState } from "react"
import { Button } from "~/components/ui/button"
import { Plus, Globe, Zap, Activity, Loader2, AlertCircle, ShieldAlert, List } from "lucide-react"
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
import { api, type DnsZone, type QueryLogEntry } from "~/lib/api"
import { useToast } from "~/lib/toast"

const defaultForm = {
  name: "",
  type: "A" as string,
  value: "",
  ttl: 3600,
}

export default function Dns() {
  const { domain } = useParams<{ domain: string }>()
  const navigate = useNavigate()
  const { toast } = useToast()
  const [records, setRecords] = useState<DnsRecord[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [nsVerified, setNsVerified] = useState(true)
  const [nsVerifiedLoaded, setNsVerifiedLoaded] = useState(false)
  const [open, setOpen] = useState(false)
  const [adding, setAdding] = useState(false)
  const [form, setForm] = useState(defaultForm)
  const [deleteConfirm, setDeleteConfirm] = useState<DnsRecord | null>(null)
  const [deleting, setDeleting] = useState<string | null>(null)
  const [editing, setEditing] = useState<DnsRecord | null>(null)
  const [editForm, setEditForm] = useState(defaultForm)
  const [saving, setSaving] = useState(false)
  const [logs, setLogs] = useState<QueryLogEntry[]>([])
  const [logsLoading, setLogsLoading] = useState(true)

  const fetchRecords = async () => {
    if (!domain) return
    setLoading(true)
    setError(null)
    try {
      const [recordsData, zonesData] = await Promise.all([
        api.dns.listRecords(domain),
        api.dns.listZones(),
      ])
      setRecords(recordsData)
      const zone = zonesData.find((z: DnsZone) => z.name === domain)
      setNsVerified(zone?.ns_verified ?? false)
      setNsVerifiedLoaded(true)
    } catch (err) {
      const msg =
        err && typeof err === "object" && "error" in err
          ? String((err as { error: unknown }).error)
          : "Failed to load DNS records"
      setError(msg)
      toast(msg, "error")
    } finally {
      setLoading(false)
    }
  }

  const fetchLogs = async () => {
    if (!domain) return
    try {
      const data = await api.logs.queryLogsByZone(domain)
      setLogs(data.logs.slice(0, 20))
    } catch {
      // silent
    } finally {
      setLogsLoading(false)
    }
  }

  useEffect(() => {
    void fetchRecords()
    void fetchLogs()
  }, [domain])

  const aRecords = records.filter((r) => r.type === "A")
  const cnameRecords = records.filter((r) => r.type === "CNAME")
  const otherRecords = records.filter(
    (r) => r.type !== "A" && r.type !== "CNAME"
  )

  const handleAdd = async () => {
    if (!form.name || !form.value || !domain) return
    setAdding(true)
    try {
      await api.dns.createRecord(domain, {
        name: form.name,
        type: form.type,
        value: form.value,
        ttl: form.ttl,
      })
      setForm(defaultForm)
      setOpen(false)
      toast("Record added", "success")
      await fetchRecords()
    } catch (err) {
      const msg =
        err && typeof err === "object" && "error" in err
          ? String((err as { error: unknown }).error)
          : "Failed to add record"
      toast(msg, "error")
    } finally {
      setAdding(false)
    }
  }

  const handleDelete = (record: DnsRecord) => {
    setDeleteConfirm(record)
  }

  const confirmDelete = async () => {
    if (!domain || !deleteConfirm) return
    setDeleting(deleteConfirm.id)
    setDeleteConfirm(null)
    try {
      await api.dns.deleteRecord(domain, deleteConfirm.name, deleteConfirm.type)
      toast("Record deleted", "success")
      await fetchRecords()
    } catch (err) {
      const msg =
        err && typeof err === "object" && "error" in err
          ? String((err as { error: unknown }).error)
          : "Failed to delete record"
      toast(msg, "error")
    } finally {
      setDeleting(null)
    }
  }

  const handleEdit = (record: DnsRecord) => {
    setEditing(record)
    setEditForm({
      name: record.name,
      type: record.type,
      value: record.value,
      ttl: record.ttl,
    })
  }

  const handleSaveEdit = async () => {
    if (!domain || !editing) return
    setSaving(true)
    try {
      await api.dns.updateRecord(domain, editing.name, editing.type, {
        value: editForm.value,
        ttl: editForm.ttl,
      })
      setEditing(null)
      toast("Record updated", "success")
      await fetchRecords()
    } catch (err) {
      const msg =
        err && typeof err === "object" && "error" in err
          ? String((err as { error: unknown }).error)
          : "Failed to update record"
      toast(msg, "error")
    } finally {
      setSaving(false)
    }
  }

  const columns = useColumns({ onDelete: handleDelete, onEdit: handleEdit, disabled: !nsVerified })

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
            <Button
              className="flex items-center gap-2 rounded-lg bg-orange-500 py-2 text-white hover:bg-orange-600 disabled:opacity-50"
              disabled={!nsVerified}
              title={!nsVerified ? "Verify NS delegation before adding records" : undefined}
            >
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

        <Dialog open={editing !== null} onOpenChange={(v) => { if (!v) setEditing(null) }}>
          <DialogContent className="sm:max-w-md">
            <DialogHeader>
              <DialogTitle>Edit DNS Record</DialogTitle>
              <DialogDescription>
                Update record{" "}
                <span className="font-medium text-white">{editing?.name}</span> (
                {editing?.type})
              </DialogDescription>
            </DialogHeader>

            <div className="space-y-4 py-2">
              <div className="space-y-2">
                <Label>Type</Label>
                <Select
                  value={editForm.type}
                  onValueChange={(v) => setEditForm({ ...editForm, type: v })}
                  disabled={saving}
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
                  value={editForm.name}
                  onChange={(e) => setEditForm({ ...editForm, name: e.target.value })}
                  disabled={saving}
                />
              </div>

              <div className="space-y-2">
                <Label>Value</Label>
                <Input
                  placeholder={
                    editForm.type === "A"
                      ? "192.0.2.1"
                      : editForm.type === "CNAME"
                        ? "example.com"
                        : ""
                  }
                  value={editForm.value}
                  onChange={(e) => setEditForm({ ...editForm, value: e.target.value })}
                  disabled={saving}
                />
              </div>

              <div className="space-y-2">
                <Label>TTL (seconds)</Label>
                <Input
                  type="number"
                  value={editForm.ttl}
                  onChange={(e) => setEditForm({ ...editForm, ttl: Number(e.target.value) })}
                  disabled={saving}
                />
              </div>
            </div>

            <DialogFooter>
              <Button variant="outline" onClick={() => setEditing(null)} disabled={saving}>
                Cancel
              </Button>
              <Button
                className="bg-orange-500 text-white hover:bg-orange-600"
                onClick={handleSaveEdit}
                disabled={saving}
              >
                {saving ? "Saving..." : "Save Changes"}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>

        <Dialog open={deleteConfirm !== null} onOpenChange={(v) => { if (!v) setDeleteConfirm(null) }}>
          <DialogContent className="sm:max-w-md">
            <DialogHeader>
              <DialogTitle>Delete DNS Record</DialogTitle>
              <DialogDescription>
                Are you sure you want to delete{" "}
                <span className="font-medium text-white">{deleteConfirm?.name}</span> (
                {deleteConfirm?.type})? This will permanently remove this record.
              </DialogDescription>
            </DialogHeader>
            <DialogFooter>
              <Button variant="outline" onClick={() => setDeleteConfirm(null)} disabled={deleting !== null}>
                Cancel
              </Button>
              <Button
                className="bg-red-600 text-white hover:bg-red-700"
                onClick={confirmDelete}
                disabled={deleting !== null}
              >
                {deleting !== null ? "Deleting..." : "Delete"}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      {nsVerifiedLoaded && !nsVerified && (
        <div className="rounded-lg border border-orange-300 bg-orange-50 p-4 dark:border-orange-700 dark:bg-orange-900/20">
          <div className="flex items-center gap-3">
            <ShieldAlert className="h-5 w-5 shrink-0 text-orange-600 dark:text-orange-400" />
            <div className="flex-1">
              <p className="text-sm font-medium text-orange-700 dark:text-orange-300">
                Zone Not Verified
              </p>
              <p className="mt-1 text-xs text-zinc-500 dark:text-zinc-400">
                Record edits are blocked until NS delegation is verified.{" "}
                Point your domain&apos;s nameservers to Hackflare, then{" "}
                <button
                  onClick={() => navigate(`/dash/domains`)}
                  className="text-orange-600 underline hover:text-orange-700 dark:text-orange-400 dark:hover:text-orange-300"
                >
                  verify from the domains page
                </button>
                .
              </p>
            </div>
          </div>
        </div>
      )}

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

      {/* Recent Queries */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Recent Queries</CardTitle>
              <CardDescription>
                DNS query logs for this domain (last 24h)
              </CardDescription>
            </div>
            <Link to={`/dash/logs?zone=${encodeURIComponent(domain ?? "")}`}>
              <Button variant="outline" size="sm" className="gap-2">
                <List className="h-4 w-4" />
                All Logs
              </Button>
            </Link>
          </div>
        </CardHeader>
        <CardContent>
          {logsLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-5 w-5 animate-spin text-zinc-500" />
            </div>
          ) : logs.length === 0 ? (
            <p className="py-8 text-center text-sm text-zinc-500">
              No recent queries for this domain.
            </p>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-zinc-200 text-left dark:border-zinc-800">
                    <th className="pb-2 pr-4 font-medium text-zinc-500 dark:text-zinc-400">Time</th>
                    <th className="pb-2 pr-4 font-medium text-zinc-500 dark:text-zinc-400">Level</th>
                    <th className="pb-2 pr-4 font-medium text-zinc-500 dark:text-zinc-400">Query</th>
                    <th className="pb-2 pr-4 font-medium text-zinc-500 dark:text-zinc-400">Status</th>
                    <th className="pb-2 font-medium text-zinc-500 dark:text-zinc-400">Duration</th>
                  </tr>
                </thead>
                <tbody>
                  {logs.map((log) => (
                    <tr key={log.id} className="border-b border-zinc-100 dark:border-zinc-800">
                      <td className="py-2 pr-4 text-zinc-500">{log.timestamp}</td>
                      <td className="py-2 pr-4">
                        <span
                          className={`rounded px-2 py-0.5 text-xs font-medium ${
                            log.level === "error"
                              ? "bg-red-50 text-red-700 dark:bg-red-900/30 dark:text-red-400"
                              : log.level === "warning"
                                ? "bg-yellow-50 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400"
                                : "bg-green-50 text-green-700 dark:bg-green-900/30 dark:text-green-400"
                          }`}
                        >
                          {log.level}
                        </span>
                      </td>
                      <td className="max-w-[200px] truncate py-2 pr-4 font-mono text-xs">
                        {log.path}
                      </td>
                      <td className="py-2 pr-4 text-zinc-500">{log.status}</td>
                      <td className="py-2 text-zinc-500">{log.ms}ms</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
