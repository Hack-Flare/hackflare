import { useEffect, useState } from "react"
import { Bell, Check, CheckCheck, RefreshCw, ExternalLink } from "lucide-react"
import { Link } from "react-router"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"
import { Button } from "~/components/ui/button"
import { api, type Notification } from "~/lib/api"
import { cn } from "~/lib/utils"

const typeStyles: Record<string, string> = {
  domain_verified:
    "border-l-green-500 bg-green-50 dark:bg-green-950/20",
  domain_added:
    "border-l-blue-500 bg-blue-50 dark:bg-blue-950/20",
  warning: "border-l-amber-500 bg-amber-50 dark:bg-amber-950/20",
  error: "border-l-red-500 bg-red-50 dark:bg-red-950/20",
}

export default function NotificationsPage() {
  const [notifications, setNotifications] = useState<Notification[]>([])
  const [loading, setLoading] = useState(true)

  const fetchNotifications = () => {
    setLoading(true)
    api.notifications
      .list()
      .then(setNotifications)
      .catch(() => {})
      .finally(() => setLoading(false))
  }

  useEffect(() => {
    fetchNotifications()
  }, [])

  const handleMarkRead = async (id: string) => {
    try {
      await api.notifications.markRead(id)
      setNotifications((prev) =>
        prev.map((n) => (n.id === id ? { ...n, read: true } : n)),
      )
    } catch {}
  }

  const handleMarkAllRead = async () => {
    try {
      await api.notifications.markAllRead()
      setNotifications((prev) => prev.map((n) => ({ ...n, read: true })))
    } catch {}
  }

  const unread = notifications.filter((n) => !n.read)

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold dark:text-white">Notifications</h1>
          <p className="mt-2 text-zinc-600 dark:text-zinc-400">
            Stay updated on your domains and account
          </p>
        </div>
        <div className="flex items-center gap-2">
          {unread.length > 0 && (
            <Button variant="outline" size="sm" onClick={handleMarkAllRead}>
              <CheckCheck className="mr-1 h-4 w-4" />
              Mark all read
            </Button>
          )}
          <Button variant="ghost" size="icon" onClick={fetchNotifications}>
            <RefreshCw className={`h-4 w-4 ${loading ? "animate-spin" : ""}`} />
          </Button>
        </div>
      </div>

      {loading && notifications.length === 0 ? (
        <Card>
          <CardContent className="flex items-center justify-center py-16">
            <RefreshCw className="h-6 w-6 animate-spin text-zinc-400" />
          </CardContent>
        </Card>
      ) : notifications.length === 0 ? (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-16 text-zinc-500">
            <Bell className="mb-3 h-8 w-8" />
            <p className="text-sm font-medium">No notifications yet</p>
            <p className="mt-1 text-xs">
              Notifications about domain verification and other events will appear here
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-2">
          {notifications.map((notif) => (
            <div
              key={notif.id}
              className={cn(
                "flex items-start gap-4 border-l-4 rounded-lg border bg-card p-4 shadow-sm transition-colors",
                typeStyles[notif.type] ?? "border-l-zinc-400",
                !notif.read && "ring-1 ring-orange-200 dark:ring-orange-800",
              )}
            >
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <h3
                    className={cn(
                      "text-sm font-semibold",
                      !notif.read && "text-orange-700 dark:text-orange-300",
                    )}
                  >
                    {notif.title}
                  </h3>
                  {!notif.read && (
                    <span className="h-2 w-2 shrink-0 rounded-full bg-orange-500" />
                  )}
                </div>
                {notif.message && (
                  <p className="mt-1 text-xs text-zinc-600 dark:text-zinc-400">
                    {notif.message}
                  </p>
                )}
                <p className="mt-1.5 text-[10px] text-zinc-400">
                  {new Date(notif.created_at).toLocaleDateString("en-US", {
                    month: "short",
                    day: "numeric",
                    hour: "2-digit",
                    minute: "2-digit",
                  })}
                </p>
              </div>
              <div className="flex shrink-0 items-center gap-1">
                {notif.link && (
                  <Button variant="ghost" size="icon" className="h-8 w-8" asChild>
                    <Link to={notif.link}>
                      <ExternalLink className="h-3.5 w-3.5" />
                    </Link>
                  </Button>
                )}
                {!notif.read && (
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8"
                    onClick={() => handleMarkRead(notif.id)}
                  >
                    <Check className="h-3.5 w-3.5" />
                  </Button>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
