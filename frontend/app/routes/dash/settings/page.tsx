import { useCallback, useEffect, useState } from "react"
import {
  Bell,
  Check,
  Copy,
  Eye,
  EyeOff,
  Key,
  KeyRound,
  Lock,
  Plus,
  Shield,
  Trash2,
} from "lucide-react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"
import { Button } from "~/components/ui/button"
import { Input } from "~/components/ui/input"
import { Label } from "~/components/ui/label"
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "~/components/ui/dialog"
import { getUserDisplayName, useAuth } from "~/lib/auth-context"
import { api, friendlyError, type ApiKey, type CreatedApiKey } from "~/lib/api"

export default function Settings() {
  const { user } = useAuth()
  const displayName = getUserDisplayName(user)

  const [keys, setKeys] = useState<ApiKey[]>([])
  const [loading, setLoading] = useState(true)

  const loadKeys = useCallback(async () => {
    setLoading(true)
    try {
      const data = await api.settings.listApiKeys()
      setKeys(data)
    } catch {
      /* ignore */
    }
    setLoading(false)
  }, [])

  useEffect(() => {
    loadKeys()
  }, [loadKeys])

  const [createOpen, setCreateOpen] = useState(false)
  const [newKeyName, setNewKeyName] = useState("")
  const [creating, setCreating] = useState(false)
  const [createdKey, setCreatedKey] = useState<CreatedApiKey | null>(null)
  const [copied, setCopied] = useState(false)

  const createKey = async () => {
    if (!newKeyName.trim()) return
    setCreating(true)
    try {
      const created = await api.settings.createApiKey(newKeyName.trim())
      setCreatedKey(created)
      setNewKeyName("")
      await loadKeys()
    } catch {
      /* ignore */
    }
    setCreating(false)
  }

  const copyKey = async () => {
    if (!createdKey) return
    try {
      await navigator.clipboard.writeText(createdKey.raw_key)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch {
      /* ignore */
    }
  }

  const closeCreate = () => {
    setCreateOpen(false)
    setCreatedKey(null)
    setNewKeyName("")
    setCopied(false)
  }

  const [revokeId, setRevokeId] = useState<string | null>(null)
  const [revoking, setRevoking] = useState(false)

  const confirmRevoke = async () => {
    if (!revokeId) return
    setRevoking(true)
    try {
      await api.settings.revokeApiKey(revokeId)
      setRevokeId(null)
      await loadKeys()
    } catch {
      /* ignore */
    }
    setRevoking(false)
  }

  const activeKeys = keys.filter((k) => !k.revoked)
  const revokedKeys = keys.filter((k) => k.revoked)

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold dark:text-white">Settings</h1>
        <p className="mt-2 text-zinc-600 dark:text-zinc-400">
          Account, API keys, and preferences
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Account
          </CardTitle>
          <CardDescription>Name, id, account management</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-3 gap-4">
            <div>
              <p className="text-xs font-medium text-zinc-600 dark:text-zinc-400">
                Name
              </p>
              <p className="mt-1 text-sm font-medium">{displayName}</p>
            </div>
            <div>
              <p className="text-xs font-medium text-zinc-600 dark:text-zinc-400">
                User ID
              </p>
              <p className="mt-1 text-sm font-medium">
                {user?.id || "Unknown"}
              </p>
            </div>
              <div>
                <p className="text-xs font-medium text-zinc-600 dark:text-zinc-400">
                  Status
                </p>
                <p className="mt-1 text-sm font-medium">
                  {user?.slack_id && user?.has_password
                    ? "Hack Club + Password"
                    : user?.slack_id
                      ? "Hack Club"
                      : "Email + Password"}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        <PasswordCard />

        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Key className="h-5 w-5" />
                API Keys
              </CardTitle>
              <CardDescription>
                Manage authentication tokens for programmatic access
              </CardDescription>
            </div>
            <Dialog open={createOpen} onOpenChange={setCreateOpen}>
              <DialogTrigger asChild>
                <Button className="rounded bg-orange-500 px-3 py-2 text-sm font-medium text-white hover:bg-orange-600">
                  <Plus className="mr-1 h-4 w-4" />
                  New Key
                </Button>
              </DialogTrigger>
              {!createdKey ? (
                <DialogContent className="sm:max-w-md">
                  <DialogHeader>
                    <DialogTitle>Create API Key</DialogTitle>
                    <DialogDescription>
                      Give your key a name so you can identify it later.
                    </DialogDescription>
                  </DialogHeader>
                  <div className="flex items-center gap-2">
                    <Input
                      value={newKeyName}
                      onChange={(e) => setNewKeyName(e.target.value)}
                      placeholder="e.g. Production CI"
                      className="flex-1"
                      onKeyDown={(e) =>
                        e.key === "Enter" && !creating && createKey()
                      }
                    />
                    <Button
                      onClick={createKey}
                      disabled={creating || !newKeyName.trim()}
                    >
                      {creating ? "Creating..." : "Create"}
                    </Button>
                  </div>
                </DialogContent>
              ) : (
                <DialogContent className="sm:max-w-md">
                  <DialogHeader>
                    <DialogTitle>Key Created</DialogTitle>
                    <DialogDescription>
                      Copy this key now. You won't be able to see it again.
                    </DialogDescription>
                  </DialogHeader>
                  <div className="space-y-3">
                    <p className="text-sm font-medium">
                      {createdKey.key.name}
                    </p>
                    <div className="flex items-center gap-2 rounded border border-zinc-200 bg-zinc-50 p-3 dark:border-zinc-700 dark:bg-zinc-900">
                      <code className="flex-1 break-all font-mono text-xs">
                        {createdKey.raw_key}
                      </code>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8 shrink-0"
                        onClick={copyKey}
                      >
                        <Copy className="h-4 w-4" />
                      </Button>
                    </div>
                    {copied && (
                      <p className="text-xs text-green-600">Copied!</p>
                    )}
                  </div>
                  <DialogFooter>
                    <DialogClose asChild>
                      <Button variant="outline" onClick={closeCreate}>
                        Done
                      </Button>
                    </DialogClose>
                  </DialogFooter>
                </DialogContent>
              )}
            </Dialog>
          </div>
        </CardHeader>
        <CardContent>
          {loading ? (
            <div className="flex items-center justify-center py-8">
              <p className="text-sm text-zinc-500">Loading...</p>
            </div>
          ) : keys.length === 0 ? (
            <div className="flex flex-col items-center gap-2 py-8">
              <Key className="h-8 w-8 text-zinc-400" />
              <p className="text-sm text-zinc-500">No API keys yet</p>
              <p className="text-xs text-zinc-500">
                Create one to access the API programmatically
              </p>
            </div>
          ) : (
            <div className="space-y-3">
              {activeKeys.map((key) => (
                <KeyRow
                  key={key.id}
                  apiKey={key}
                  onRevoke={() => setRevokeId(key.id)}
                />
              ))}
              {revokedKeys.length > 0 && (
                <>
                  <div className="pt-4">
                    <p className="text-xs font-medium uppercase tracking-wide text-zinc-500">
                      Revoked
                    </p>
                  </div>
                  {revokedKeys.map((key) => (
                    <KeyRow key={key.id} apiKey={key} onRevoke={() => {}} />
                  ))}
                </>
              )}
            </div>
          )}
        </CardContent>
      </Card>

      <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Bell className="h-5 w-5" />
              Notifications
            </CardTitle>
            <CardDescription>Email + alert preferences</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <label className="flex cursor-pointer items-center gap-3">
              <input
                type="checkbox"
                defaultChecked
                className="h-4 w-4 rounded"
              />
              <span className="text-sm">Security alerts</span>
            </label>
            <label className="flex cursor-pointer items-center gap-3">
              <input
                type="checkbox"
                defaultChecked
                className="h-4 w-4 rounded"
              />
              <span className="text-sm">Weekly reports</span>
            </label>
            <label className="flex cursor-pointer items-center gap-3">
              <input type="checkbox" className="h-4 w-4 rounded" />
              <span className="text-sm">Marketing emails</span>
            </label>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Lock className="h-5 w-5" />
              Security
            </CardTitle>
            <CardDescription>Account protection settings</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex items-center justify-between rounded border border-zinc-200 p-2 dark:border-zinc-800">
              <span className="text-sm">Two-factor authentication</span>
              <span className="text-xs font-medium text-orange-600">
                Setup
              </span>
            </div>
            <div className="flex items-center justify-between rounded border border-zinc-200 p-2 dark:border-zinc-800">
              <span className="text-sm">Session timeout</span>
              <span className="text-xs text-zinc-600 dark:text-zinc-400">
                30 minutes
              </span>
            </div>
            <div className="flex items-center justify-between rounded border border-zinc-200 p-2 dark:border-zinc-800">
              <span className="text-sm">Active sessions</span>
              <span className="text-xs text-zinc-600 dark:text-zinc-400">
                1
              </span>
            </div>
          </CardContent>
        </Card>
      </div>

      <Dialog
        open={revokeId !== null}
        onOpenChange={(open) => {
          if (!open) setRevokeId(null)
        }}
      >
        <DialogContent className="sm:max-w-sm">
          <DialogHeader>
            <DialogTitle>Revoke API Key</DialogTitle>
            <DialogDescription>
              This will permanently invalidate this key. Any services using it
              will lose access immediately.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter className="gap-2">
            <Button variant="outline" onClick={() => setRevokeId(null)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={confirmRevoke}
              disabled={revoking}
            >
              {revoking ? "Revoking..." : "Revoke"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}

function PasswordCard() {
  const [currentPassword, setCurrentPassword] = useState("")
  const [newPassword, setNewPassword] = useState("")
  const [confirmPassword, setConfirmPassword] = useState("")
  const [showCurrent, setShowCurrent] = useState(false)
  const [showNew, setShowNew] = useState(false)
  const [loading, setLoading] = useState(false)
  const [message, setMessage] = useState<{
    text: string
    type: "success" | "error"
  } | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setMessage(null)

    if (newPassword !== confirmPassword) {
      setMessage({ text: "Passwords do not match.", type: "error" })
      return
    }

    setLoading(true)
    try {
      await api.settings.setPassword({
        current_password:
          currentPassword.length > 0 ? currentPassword : undefined,
        new_password: newPassword,
      })
      setMessage({
        text: "Password updated successfully.",
        type: "success",
      })
      setCurrentPassword("")
      setNewPassword("")
      setConfirmPassword("")
    } catch (err: unknown) {
      const code =
        err && typeof err === "object" && "error" in err
          ? String((err as { error: unknown }).error)
          : ""
      setMessage({
        text: friendlyError(code) || "Failed to update password.",
        type: "error",
      })
    } finally {
      setLoading(false)
    }
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="rounded-full bg-zinc-100 p-2 dark:bg-zinc-800">
              <Lock className="h-5 w-5 text-zinc-600 dark:text-zinc-400" />
            </div>
            <div>
              <CardTitle>Password</CardTitle>
              <CardDescription>
                Set or change your account password
              </CardDescription>
            </div>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-3">
          <div className="relative">
            <Label>Current password</Label>
            <div className="relative">
              <Input
                type={showCurrent ? "text" : "password"}
                value={currentPassword}
                onChange={(e) => setCurrentPassword(e.target.value)}
                placeholder="Leave blank if none set"
                className="pr-9"
              />
              <button
                type="button"
                onClick={() => setShowCurrent(!showCurrent)}
                className="absolute right-2.5 top-1/2 -translate-y-1/2 text-zinc-400 hover:text-zinc-600"
              >
                {showCurrent ? (
                  <EyeOff className="h-4 w-4" />
                ) : (
                  <Eye className="h-4 w-4" />
                )}
              </button>
            </div>
          </div>

          <div className="relative">
            <Label>New password</Label>
            <div className="relative">
              <Input
                type={showNew ? "text" : "password"}
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                placeholder="At least 8 characters"
                required
                className="pr-9"
              />
              <button
                type="button"
                onClick={() => setShowNew(!showNew)}
                className="absolute right-2.5 top-1/2 -translate-y-1/2 text-zinc-400 hover:text-zinc-600"
              >
                {showNew ? (
                  <EyeOff className="h-4 w-4" />
                ) : (
                  <Eye className="h-4 w-4" />
                )}
              </button>
            </div>
          </div>

          <div>
            <Label>Confirm new password</Label>
            <Input
              type="password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              placeholder="Repeat new password"
              required
            />
          </div>

          {message && (
            <div
              className={`rounded p-3 text-sm ${
                message.type === "success"
                  ? "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
                  : "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400"
              }`}
            >
              {message.text}
            </div>
          )}

          <Button type="submit" disabled={loading || !newPassword}>
            {loading ? "Saving..." : "Update Password"}
          </Button>
        </form>
      </CardContent>
    </Card>
  )
}

function KeyRow({
  apiKey,
  onRevoke,
}: {
  apiKey: ApiKey
  onRevoke: () => void
}) {
  return (
    <div className="flex items-center justify-between rounded-lg border border-zinc-200 p-3 hover:bg-zinc-50 dark:border-zinc-800 dark:hover:bg-zinc-800/50">
      <div className="min-w-0 flex-1">
        <p className="text-sm font-medium">{apiKey.name}</p>
        <p className="mt-0.5 text-xs text-zinc-600 dark:text-zinc-400">
          <span className="font-mono">{apiKey.prefix}...</span>
          <span className="mx-1.5">·</span>
          Created {new Date(apiKey.created_at).toLocaleDateString()}
          {apiKey.last_used_at && (
            <>
              <span className="mx-1.5">·</span>
              Last used{" "}
              {new Date(apiKey.last_used_at).toLocaleDateString()}
            </>
          )}
        </p>
      </div>
      <span
        className={`rounded px-2 py-1 text-xs font-medium ${
          apiKey.revoked
            ? "bg-zinc-100 text-zinc-500 dark:bg-zinc-900/30 dark:text-zinc-500"
            : "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
        }`}
      >
        {apiKey.revoked ? "Revoked" : "Active"}
      </span>
      {!apiKey.revoked && (
        <Button
          variant="ghost"
          size="icon"
          className="ml-3 h-8 w-8"
          onClick={onRevoke}
        >
          <Trash2 className="h-4 w-4 text-red-400" />
        </Button>
      )}
    </div>
  )
}
