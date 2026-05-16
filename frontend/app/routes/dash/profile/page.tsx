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
import { Clock, Globe, Shield, LogOut } from "lucide-react"
import { Button } from "~/components/ui/button"

const sessions = [
  {
    id: 1,
    device: "Chrome on MacOS",
    ip: "203.0.113.42",
    location: "San Francisco, CA",
    lastActive: "now",
    current: true,
  },
  {
    id: 2,
    device: "Safari on iOS",
    ip: "203.0.113.55",
    location: "San Francisco, CA",
    lastActive: "2h ago",
    current: false,
  },
  {
    id: 3,
    device: "Firefox on Windows",
    ip: "203.0.113.78",
    location: "New York, NY",
    lastActive: "3d ago",
    current: false,
  },
]

export default function Profile() {
  const { user, logout } = useAuth()
  const navigate = useNavigate()
  const displayName = getUserDisplayName(user)
  const email = user?.email || "Unknown"
  const slackId = user?.slack_id || "Unknown"
  const verificationStatus = user ? "Verified through Hack Club" : "Unknown"
  const accountStatus = user?.eligible ? "Eligible for Hackflare" : "Not eligible"
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
              <AvatarImage src={avatar} alt={userLabel} className="lounded-lg object-cover"/>
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
                <p className="mt-1 text-sm font-medium">
                  {email}
                </p>
              </div>
              <div>
                <p className="text-xs font-medium text-zinc-600 dark:text-zinc-400">
                  Slack ID
                </p>
                <p className="mt-1 text-sm font-medium">
                  {slackId}
                </p>
              </div>
              <div className="flex gap-2 pt-2">
                <Button variant="outline">
                  Edit Profile
                </Button>
                <Button
                  onClick={handleLogout}
                  variant="destructive"
                >
                  <LogOut className="h-4 w-4" />
                  Sign Out
                </Button>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/*<Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Globe className="h-5 w-5" />
            Active Sessions
          </CardTitle>
          <CardDescription>Devices logged into your account</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {sessions.map((session) => (
              <div
                key={session.id}
                className="flex items-center justify-between rounded-lg border border-zinc-200 p-3 hover:bg-zinc-50 dark:border-zinc-800 dark:hover:bg-zinc-800/50"
              >
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <p className="text-sm font-medium">{session.device}</p>
                    {session.current && (
                      <span className="rounded bg-green-100 px-2 py-0.5 text-xs font-medium text-green-800 dark:bg-green-900/30 dark:text-green-400">
                        Current
                      </span>
                    )}
                  </div>
                  <p className="mt-1 text-xs text-zinc-600 dark:text-zinc-400">
                    {session.ip} • {session.location}
                  </p>
                  <p className="text-xs text-zinc-600 dark:text-zinc-400">
                    Last active: {session.lastActive}
                  </p>
                </div>
                {!session.current && (
                  <button className="rounded bg-zinc-100 px-3 py-1 text-xs font-medium text-zinc-700 hover:bg-red-100 hover:text-red-600 dark:bg-zinc-800 dark:text-zinc-300 dark:hover:bg-red-900/30 dark:hover:text-red-400">
                    Sign Out
                  </button>
                )}
              </div>
            ))}
          </div>
        </CardContent>
      </Card>*/}

      <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Clock className="h-5 w-5" />
              Account Status
            </CardTitle>
            <CardDescription>Derived from the current user record</CardDescription>
          </CardHeader>
          <CardContent className="text-sm">
            <div className="flex items-center justify-between">
              <span>Hack Club verification</span>
              <span className="font-medium text-green-600">{verificationStatus}</span>
            </div>
            <div className="mt-2 flex items-center justify-between">
              <span>Hackflare access</span>
              <span className="font-medium text-orange-600">{accountStatus}</span>
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
