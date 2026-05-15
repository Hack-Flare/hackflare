import { Avatar, AvatarFallback } from "~/components/ui/avatar"
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
  const initials = displayName
    .split(" ")
    .map((part) => part[0])
    .join("")
    .toUpperCase()
    .slice(0, 2)

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
                  Authenticated via Hack Club
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

      <Card>
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
      </Card>

      <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Clock className="h-5 w-5" />
              Account Created
            </CardTitle>
            <CardDescription>Member since</CardDescription>
          </CardHeader>
          <CardContent className="text-sm">
            <p className="font-medium">January 12, 2024</p>
            <p className="mt-1 text-zinc-600 dark:text-zinc-400">
              Account age: 4 months
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Shield className="h-5 w-5" />
              Account Status
            </CardTitle>
            <CardDescription>Security verification</CardDescription>
          </CardHeader>
          <CardContent className="space-y-2 text-sm">
            <div className="flex items-center justify-between">
              <span>Email verified</span>
              <span className="font-medium text-green-600">✓</span>
            </div>
            <div className="flex items-center justify-between">
              <span>2FA enabled</span>
              <span className="font-medium text-orange-600">Setup</span>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
