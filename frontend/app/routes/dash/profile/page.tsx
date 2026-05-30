import { useEffect, useState } from "react"
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/avatar"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"
import { getUserDisplayName, useAuth } from "~/lib/auth-context"
import { useNavigate } from "react-router"
import { Clock, Globe, Shield, LogOut, Loader2 } from "lucide-react"
import { Button } from "~/components/ui/button"
import { api, type UserSession } from "~/lib/api"

function formatTime(iso: string): string {
  const d = new Date(iso)
  return d.toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  })
}

export default function Profile() {
  const { user, logout } = useAuth()
  const navigate = useNavigate()
  const [sessions, setSessions] = useState<UserSession[]>([])
  const [sessionsLoading, setSessionsLoading] = useState(true)
  const displayName = getUserDisplayName(user)
  const email = user?.email || "Unknown"
  const slackId = user?.slack_id || "Unknown"
  const verificationStatus = user?.slack_id ? "Verified" : "Not verified"
  const accountStatus = user?.eligible ? "Active" : "Not eligible"
  const initials = displayName
    .split(" ")
    .map((part) => part[0])
    .join("")
    .toUpperCase()
    .slice(0, 2)

  const avatar = user?.slack_id
    ? `https://cachet.dunkirk.sh/users/${user.slack_id}/r`
    : undefined

  const userLabel = getUserDisplayName(user)

  useEffect(() => {
    const load = async () => {
      try {
        const data = await api.sessions.list()
        setSessions(data)
      } catch {
        // silently fail
      } finally {
        setSessionsLoading(false)
      }
    }
    void load()
  }, [])

  const activeSessions = sessions.filter((s) => !s.revoked_at)
  const currentSessionId = activeSessions.length > 0
    ? activeSessions.reduce((a, b) =>
        new Date(a.created_at) > new Date(b.created_at) ? a : b
      ).id
    : null

  const handleLogout = async () => {
    await logout()
    navigate("/login")
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold dark:text-white">Profile</h1>
        <p className="mt-2 text-zinc-600 dark:text-zinc-400">
          Account, access + session info
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Account Overview</CardTitle>
          <CardDescription>Current signed-in user details</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-start gap-6">
            <Avatar className="h-20 w-20 rounded-xl">
              <AvatarImage
                src={avatar}
                alt={userLabel}
                className="lounded-lg object-cover"
              />
              <AvatarFallback className="rounded-xl bg-orange-500 text-lg font-semibold text-white">
                {initials}
              </AvatarFallback>
            </Avatar>
            <div className="flex-1 space-y-4">
              <div>
                <p className="text-xs font-medium text-zinc-600 dark:text-zinc-400">
                  Name
                </p>
                <p className="mt-1 text-sm font-medium">{displayName}</p>
              </div>
              <div>
                <p className="text-xs font-medium text-zinc-600 dark:text-zinc-400">
                  Email
                </p>
                <p className="mt-1 text-sm font-medium">{email}</p>
              </div>
              <div>
                <p className="text-xs font-medium text-zinc-600 dark:text-zinc-400">
                  Slack ID
                </p>
                <p className="mt-1 text-sm font-medium">{slackId}</p>
              </div>
              <div className="flex gap-2 pt-2">
                <Button variant="outline">Edit Profile</Button>
                <Button onClick={handleLogout} variant="destructive">
                  <LogOut className="h-4 w-4" />
                  Sign Out
                </Button>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Globe className="h-5 w-5" />
            Active Sessions
          </CardTitle>
          <CardDescription>Active login sessions for your account</CardDescription>
        </CardHeader>
        <CardContent>
          {sessionsLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-5 w-5 animate-spin text-zinc-500" />
            </div>
          ) : sessions.length === 0 ? (
            <p className="py-4 text-center text-sm text-zinc-500">
              No sessions found.
            </p>
          ) : (
            <div className="space-y-3">
              {sessions.map((session) => {
                const isCurrent = session.id === currentSessionId
                const revoked = !!session.revoked_at
                return (
                  <div
                    key={session.id}
                    className="flex items-center justify-between rounded-lg border border-zinc-200 p-3 hover:bg-zinc-50 dark:border-zinc-800 dark:hover:bg-zinc-800/50"
                  >
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <p className="text-sm font-medium">
                          {session.ip_address}
                        </p>
                        {isCurrent && (
                          <span className="rounded bg-green-100 px-2 py-0.5 text-xs font-medium text-green-800 dark:bg-green-900/30 dark:text-green-400">
                            Current
                          </span>
                        )}
                        {revoked && (
                          <span className="rounded bg-zinc-100 px-2 py-0.5 text-xs font-medium text-zinc-600 dark:bg-zinc-800 dark:text-zinc-400">
                            Revoked
                          </span>
                        )}
                      </div>
                      <p className="mt-1 text-xs text-zinc-600 dark:text-zinc-400">
                        Created {formatTime(session.created_at)}
                        {revoked
                          ? ` · Revoked ${formatTime(session.revoked_at!)}`
                          : ` · Expires ${formatTime(session.expires_at)}`}
                      </p>
                    </div>
                  </div>
                )
              })}
            </div>
          )}
        </CardContent>
      </Card>

      <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Clock className="h-5 w-5" />
              Account Status
            </CardTitle>
            <CardDescription>
              Derived from the current user record
            </CardDescription>
          </CardHeader>
          <CardContent className="text-sm">
            <div className="flex items-center justify-between">
              <span>Hack Club verification</span>
              <span className="font-medium text-green-600">
                {verificationStatus}
              </span>
            </div>
            <div className="mt-2 flex items-center justify-between">
              <span>Hackflare access</span>
              <span className="font-medium text-green-600">
                {accountStatus}
              </span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Shield className="h-5 w-5" />
              User Details
            </CardTitle>
            <CardDescription>Data returned by /api/v1/users/me</CardDescription>
          </CardHeader>
          <CardContent className="space-y-2 text-sm">
            <div className="flex items-center justify-between">
              <span>User ID</span>
              <span className="font-medium text-zinc-600 dark:text-zinc-300">
                {user?.id || "Unknown"}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span>First name</span>
              <span className="font-medium text-zinc-600 dark:text-zinc-300">
                {user?.first_name || "Unknown"}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span>Last name</span>
              <span className="font-medium text-zinc-600 dark:text-zinc-300">
                {user?.last_name || "Unknown"}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span>YSWS eligible</span>
              <span className="font-medium text-zinc-600 dark:text-zinc-300">
                {user ? (user.eligible ? "Yes" : "No") : "Unknown"}
              </span>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
