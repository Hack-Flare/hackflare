import { Plus, Globe, Shield, Clock, Loader2, AlertCircle } from "lucide-react"
import { NavLink } from "react-router"
import { useEffect, useState } from "react"
import { api, type DnsZone } from "~/lib/api"
import { Button } from "~/components/ui/button"
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
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "~/components/ui/table"
import { Input } from "~/components/ui/input"
import { Label } from "~/components/ui/label"

export default function Domains() {
  const [zones, setZones] = useState<DnsZone[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [open, setOpen] = useState(false)
  const [domainInput, setDomainInput] = useState("")
  const [adding, setAdding] = useState(false)
  const [addError, setAddError] = useState<string | null>(null)
  const [verifying, setVerifying] = useState<Record<string, boolean>>({})
  const [verifyMsg, setVerifyMsg] = useState<Record<string, string>>({})

  const fetchZones = async () => {
    setLoading(true)
    setError(null)
    try {
      const data = await api.dns.listZones()
      setZones(data)
    } catch (err) {
      const msg =
        err && typeof err === "object" && "error" in err
          ? String((err as { error: unknown }).error)
          : "Failed to load domains"
      setError(msg)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    void fetchZones()
  }, [])

  const handleAddDomain = async () => {
    const name = domainInput.trim()
    if (!name) return

    setAdding(true)
    setAddError(null)
    try {
      await api.dns.createZone(name)
      setDomainInput("")
      setOpen(false)
      await fetchZones()
    } catch (err) {
      const msg =
        err && typeof err === "object" && "error" in err
          ? String((err as { error: unknown }).error)
          : "Failed to add domain"
      setAddError(msg)
    } finally {
      setAdding(false)
    }
  }

  const handleVerify = async (zoneName: string) => {
    setVerifying((prev) => ({ ...prev, [zoneName]: true }))
    setVerifyMsg((prev) => ({ ...prev, [zoneName]: "" }))
    try {
      const result = await api.dns.verifyZone(zoneName)
      setVerifyMsg((prev) => ({
        ...prev,
        [zoneName]: result.verified
          ? "Verified!"
          : result.message || "Verification failed",
      }))
    } catch (err) {
      const msg =
        err && typeof err === "object" && "error" in err
          ? String((err as { error: unknown }).error)
          : "Verification request failed"
      setVerifyMsg((prev) => ({ ...prev, [zoneName]: msg }))
    } finally {
      setVerifying((prev) => ({ ...prev, [zoneName]: false }))
    }
  }

  return (
    <div className="flex-1 p-1">
      <div className="mb-8 flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold dark:text-white">Domains</h1>
          <p className="mt-2 text-zinc-600 dark:text-zinc-400">
            Manage and monitor your domains
          </p>
        </div>

        <Dialog open={open} onOpenChange={setOpen}>
          <DialogTrigger asChild>
            <Button className="flex items-center gap-2 rounded-lg bg-orange-500 px-4 py-2 text-white hover:bg-orange-600">
              <Plus className="h-4 w-4" />
              Add Domain
            </Button>
          </DialogTrigger>
          <DialogContent className="sm:max-w-md">
            <DialogHeader>
              <DialogTitle>Add a Domain</DialogTitle>
              <DialogDescription>
                Enter the domain you want to manage. You'll need to verify
                nameservers after adding.
              </DialogDescription>
            </DialogHeader>
            <div className="space-y-4 py-2">
              {addError && (
                <div className="rounded bg-red-100 px-3 py-2 text-sm text-red-800 dark:bg-red-900/30 dark:text-red-400">
                  {addError}
                </div>
              )}
              <div className="space-y-2">
                <Label htmlFor="domain">Domain name</Label>
                <Input
                  id="domain"
                  placeholder="example.com"
                  value={domainInput}
                  onChange={(e) => setDomainInput(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && !adding && handleAddDomain()}
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
                onClick={handleAddDomain}
                disabled={adding}
              >
                {adding ? "Adding..." : "Add Domain"}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      {/* Stats */}
      <div className="mb-8 grid grid-cols-1 gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <Globe className="h-4 w-4" />
              Total Domains
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">{zones.length}</p>
            <p className="mt-1 text-xs text-zinc-600 dark:text-zinc-400">
              {zones.length === 0 ? "Add your first domain" : "All active"}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <Shield className="h-4 w-4" />
              Verified Zones
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">
              {zones.filter((z: any) => z.ns_verified).length}
            </p>
            <p className="mt-1 text-xs text-green-600">NS verified</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <Clock className="h-4 w-4" />
              Pending
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">
              {zones.filter((z: any) => !z.ns_verified).length}
            </p>
            <p className="mt-1 text-xs text-orange-600">Need verification</p>
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
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow className="border-zinc-800 hover:bg-transparent">
                    <TableHead className="font-semibold text-zinc-400">
                      Domain
                    </TableHead>
                    <TableHead className="font-semibold text-zinc-400">
                      Registrar
                    </TableHead>
                    <TableHead className="font-semibold text-zinc-400">
                      DNS
                    </TableHead>
                    <TableHead className="font-semibold text-zinc-400">
                      SSL
                    </TableHead>
                    <TableHead className="font-semibold text-zinc-400">
                      Expires
                    </TableHead>
                    <TableHead className="font-semibold text-zinc-400">
                      Status
                    </TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {zones.length === 0 ? (
                    <TableRow>
                      <TableCell
                        colSpan={6}
                        className="py-8 text-center text-zinc-600 dark:text-zinc-400"
                      >
                        No domains yet. Add your first domain above.
                      </TableCell>
                    </TableRow>
                  ) : (
                    zones.map((zone) => (
                      <TableRow
                        key={zone.name}
                        className="border-zinc-800 hover:bg-zinc-800/50"
                      >
                        <TableCell className="font-medium">
                          <NavLink
                            to={`/dash/domains/${zone.name}/dns`}
                            className="transition-colors hover:text-orange-500"
                          >
                            {zone.name}
                          </NavLink>
                        </TableCell>
                        <TableCell className="text-zinc-400">—</TableCell>
                        <TableCell className="text-zinc-400">Hackflare</TableCell>
                        <TableCell>
                          <span className="rounded border border-green-700 bg-green-900/30 px-2 py-1 text-xs font-medium text-green-400">
                            Valid
                          </span>
                        </TableCell>
                        <TableCell className="text-zinc-400">—</TableCell>
                        <TableCell>
                          <div className="flex items-center gap-2">
                            <span
                              className={`rounded px-2 py-1 text-xs font-medium ${
                                zone.ns_verified
                                  ? "border border-green-700 bg-green-900/30 text-green-400"
                                  : "border border-orange-700 bg-orange-900/30 text-orange-400"
                              }`}
                            >
                              {zone.ns_verified ? "Verified" : "Pending"}
                            </span>
                            {!zone.ns_verified && (
                              <Button
                                size="sm"
                                variant="ghost"
                                className="h-7 px-2 text-xs text-orange-400 hover:text-orange-300"
                                onClick={() => handleVerify(zone.name)}
                                disabled={verifying[zone.name]}
                              >
                                {verifying[zone.name] ? (
                                  <Loader2 className="h-3 w-3 animate-spin" />
                                ) : (
                                  "Verify"
                                )}
                              </Button>
                            )}
                            {verifyMsg[zone.name] && (
                              <span className="text-xs text-zinc-500 max-w-40 truncate">
                                {verifyMsg[zone.name]}
                              </span>
                            )}
                          </div>
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Quick Links */}
      <div className="mt-8 grid grid-cols-1 gap-4 md:grid-cols-3">
        <Card className="cursor-pointer transition-all hover:border-orange-500 hover:shadow-md">
          <CardHeader>
            <CardTitle className="text-base">DNS Records</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-zinc-600 dark:text-zinc-400">
              Manage DNS records, MX, CNAME, and A records
            </p>
          </CardContent>
        </Card>
        <Card className="cursor-pointer transition-all hover:border-orange-500 hover:shadow-md">
          <CardHeader>
            <CardTitle className="text-base">SSL Certificates</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-zinc-600 dark:text-zinc-400">
              View and manage SSL/TLS certificates
            </p>
          </CardContent>
        </Card>
        <Card className="cursor-pointer transition-all hover:border-orange-500 hover:shadow-md">
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
