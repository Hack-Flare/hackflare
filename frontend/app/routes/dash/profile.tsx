import { Avatar, AvatarFallback } from "~/components/ui/avatar"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card"
import { useAuth } from "~/lib/auth-context"
import { useNavigate } from "react-router"
import { Clock, Globe, Shield, LogOut } from "lucide-react"

const sessions = [
  { id: 1, device: "Chrome on MacOS", ip: "203.0.113.42", location: "San Francisco, CA", lastActive: "now", current: true },
  { id: 2, device: "Safari on iOS", ip: "203.0.113.55", location: "San Francisco, CA", lastActive: "2h ago", current: false },
  { id: 3, device: "Firefox on Windows", ip: "203.0.113.78", location: "New York, NY", lastActive: "3d ago", current: false },
]

export default function Profile() {
  const { user, logout } = useAuth()
  const navigate = useNavigate()
  const initials = user?.name
    ? user.name.split(" ").map(n => n[0]).join("").toUpperCase().slice(0, 2)
    : "?"

  const handleLogout = () => {
    logout()
    navigate("/auth")
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold dark:text-white">Profile</h1>
        <p className="text-zinc-600 dark:text-zinc-400 mt-2">Account, access + session info</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Account Overview</CardTitle>
          <CardDescription>Current signed-in user details</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-start gap-6">
            <Avatar className="h-20 w-20 rounded-xl">
              <AvatarFallback className="rounded-xl bg-orange-500 text-white font-semibold text-lg">
                {initials}
              </AvatarFallback>
            </Avatar>
            <div className="flex-1 space-y-4">
              <div>
                <p className="text-xs text-zinc-600 dark:text-zinc-400 font-medium">Name</p>
                <p className="text-sm font-medium mt-1">{user?.name || "Unknown"}</p>
              </div>
              <div>
                <p className="text-xs text-zinc-600 dark:text-zinc-400 font-medium">Email</p>
                <p className="text-sm font-medium mt-1">{user?.email || "Unknown"}</p>
              </div>
              <div>
                <p className="text-xs text-zinc-600 dark:text-zinc-400 font-medium">Role</p>
                <p className="text-sm font-medium mt-1">{user?.is_admin ? "Admin" : "User"}</p>
              </div>
              <div className="flex gap-2 pt-2">
                <button className="px-3 py-2 rounded text-sm font-medium bg-zinc-100 hover:bg-zinc-200 dark:bg-zinc-800 dark:hover:bg-zinc-700 text-zinc-900 dark:text-zinc-100">
                  Edit Profile
                </button>
                <button
                  onClick={handleLogout}
                  className="px-3 py-2 rounded text-sm font-medium bg-red-100 hover:bg-red-200 dark:bg-red-900/30 dark:hover:bg-red-900/50 text-red-600 dark:text-red-400 flex items-center gap-2"
                >
                  <LogOut className="h-4 w-4" />
                  Sign Out
                </button>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2"><Globe className="h-5 w-5" />Active Sessions</CardTitle>
          <CardDescription>Devices logged into your account</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {sessions.map((session) => (
              <div key={session.id} className="flex items-center justify-between p-3 rounded-lg border border-zinc-200 dark:border-zinc-800 hover:bg-zinc-50 dark:hover:bg-zinc-800/50">
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <p className="text-sm font-medium">{session.device}</p>
                    {session.current && (
                      <span className="px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400">
                        Current
                      </span>
                    )}
                  </div>
                  <p className="text-xs text-zinc-600 dark:text-zinc-400 mt-1">{session.ip} • {session.location}</p>
                  <p className="text-xs text-zinc-600 dark:text-zinc-400">Last active: {session.lastActive}</p>
                </div>
                {!session.current && (
                  <button className="px-3 py-1 rounded text-xs font-medium bg-zinc-100 hover:bg-red-100 dark:bg-zinc-800 dark:hover:bg-red-900/30 text-zinc-700 dark:text-zinc-300 hover:text-red-600 dark:hover:text-red-400">
                    Sign Out
                  </button>
                )}
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2"><Clock className="h-5 w-5" />Account Created</CardTitle>
            <CardDescription>Member since</CardDescription>
          </CardHeader>
          <CardContent className="text-sm">
            <p className="font-medium">January 12, 2024</p>
            <p className="text-zinc-600 dark:text-zinc-400 mt-1">Account age: 4 months</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2"><Shield className="h-5 w-5" />Account Status</CardTitle>
            <CardDescription>Security verification</CardDescription>
          </CardHeader>
          <CardContent className="space-y-2 text-sm">
            <div className="flex items-center justify-between">
              <span>Email verified</span>
              <span className="text-green-600 font-medium">✓</span>
            </div>
            <div className="flex items-center justify-between">
              <span>2FA enabled</span>
              <span className="text-orange-600 font-medium">Setup</span>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
