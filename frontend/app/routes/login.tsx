import { useState } from "react"
import { Navigate } from "react-router"
import { useAuth } from "../lib/auth-context"
import { api } from "../lib/api"
import { HackClubIcon } from "../components/icons/hackclub"
import { GoogleIcon } from "../components/icons/google"
import { GitHubIcon } from "../components/icons/github"
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
    <div className="relative flex min-h-screen items-center justify-center overflow-hidden bg-zinc-50 dark:bg-zinc-950">
      <div className="absolute inset-0 bg-[linear-gradient(to_right,#80808012_1px,transparent_1px),linear-gradient(to_bottom,#80808012_1px,transparent_1px)] bg-size-[24px_24px]" />

      <Card className="relative w-full max-w-md">
        <CardHeader>
          <CardTitle>Welcome to HackFlare</CardTitle>
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

          <div className="relative my-4">
            <div className="absolute inset-0 flex items-center">
              <div className="w-full border-t border-zinc-200 dark:border-zinc-800" />
            </div>
            <div className="relative flex justify-center text-xs">
              <span className="bg-card px-2 text-muted-foreground">or continue with</span>
            </div>
          </div>

          <button
            disabled
            className="flex w-full items-center justify-center gap-2 rounded-lg border border-zinc-200 bg-white py-2 text-sm font-medium text-zinc-400 opacity-50 dark:border-zinc-800 dark:bg-zinc-900"
          >
            <GoogleIcon className="h-5 w-5" />
            Sign in with Google
          </button>

          <button
            disabled
            className="mt-2 flex w-full items-center justify-center gap-2 rounded-lg border border-zinc-200 bg-white py-2 text-sm font-medium text-zinc-400 opacity-50 dark:border-zinc-800 dark:bg-zinc-900"
          >
            <GitHubIcon className="h-5 w-5" />
            Sign in with GitHub
          </button>

          <div className="relative my-4">
            <div className="absolute inset-0 flex items-center">
              <div className="w-full border-t border-zinc-200 dark:border-zinc-800" />
            </div>
            <div className="relative flex justify-center text-xs">
              <span className="bg-card px-2 text-muted-foreground">preferred method</span>
            </div>
          </div>

          <div className="mb-4 rounded-lg border border-orange-200 bg-orange-50 p-3 text-xs text-orange-800 dark:border-orange-900/50 dark:bg-orange-950/50 dark:text-orange-400">
            Hack Club Auth is the only supported sign-in method at this time
          </div>

          <button
            onClick={handleHackclubLogin}
            disabled={loading}
            className="flex w-full items-center justify-center gap-2 rounded-lg bg-hackclub-500 py-2 text-sm font-medium text-white hover:bg-hackclub-600 disabled:bg-hackclub-400"
          >
            <HackClubIcon className="h-6 w-6" />
            {loading ? "Redirecting\u2026" : "Sign in with Hack Club"}
          </button>

          <a
            href="/"
            className="mt-4 flex w-full items-center justify-center gap-1.5 text-sm text-zinc-500 underline underline-offset-2 hover:text-zinc-700 dark:text-zinc-400 dark:hover:text-zinc-200"
          >
            &larr; Back to home
          </a>
        </CardContent>
      </Card>
    </div>
  )
}
