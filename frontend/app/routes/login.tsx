import { useState } from "react"
import { Navigate } from "react-router"
import { useAuth } from "../lib/auth-context"
import { api } from "../lib/api"
import { HackClubIcon } from "../components/icons/hackclub"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "../components/ui/card"

export default function Login() {
  const { user, ready } = useAuth()
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  if (!ready) {
    return null
  }

  if (user) {
    return <Navigate to="/dash" replace />
  }

  const handleHackclubLogin = async () => {
    setError(null)
    setLoading(true)

    try {
      const target = `${window.location.origin}/auth/hackclub?returnTo=${encodeURIComponent("/dash")}`
      const loginUrl = api.auth.loginUrl(target)

      console.info("[Auth] start Hack Club sign-in", { target, loginUrl })
      window.location.assign(loginUrl)
    } catch {
      console.error("[Auth] failed to start Hack Club sign-in")
      setError("Failed to start Hack Club login")
      setLoading(false)
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-zinc-50 dark:bg-zinc-950">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>Hackflare</CardTitle>
          <CardDescription>Sign in to manage your domains</CardDescription>
        </CardHeader>
        <CardContent>
          {/*<div className="mb-4 rounded-lg border border-zinc-200 bg-zinc-100 px-3 py-2 text-sm text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300">
            Demo login: tes@123.com / 1234
            <button
              type="button"
              onClick={logout}
              className="ml-2 font-medium text-orange-500 hover:text-orange-600"
            >
              Sign out
            </button>
          </div>*/}

          {error && (
            <div className="mb-4 rounded bg-red-100 p-3 text-sm text-red-800 dark:bg-red-900/30 dark:text-red-400">
              {error}
            </div>
          )}

          <button
            onClick={handleHackclubLogin}
            disabled={loading}
            className="flex w-full items-center justify-center gap-2 rounded-lg bg-hackclub-500 py-2 text-sm font-medium text-white hover:bg-hackclub-600 disabled:bg-hackclub-400"
          >
            <HackClubIcon className="h-6 w-6" />
            {loading ? "Redirecting..." : "Sign in with Hack Club"}
          </button>
        </CardContent>
      </Card>
    </div>
  )
}
