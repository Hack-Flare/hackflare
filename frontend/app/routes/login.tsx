import { useState } from "react"
import { Link, Navigate } from "react-router"
import { useAuth } from "../lib/auth-context"
import { api, friendlyError } from "../lib/api"
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
import { Button } from "../components/ui/button"
import { Input } from "../components/ui/input"
import { Label } from "../components/ui/label"

export default function Login() {
  const { user, ready, refreshUser } = useAuth()
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)
  const [email, setEmail] = useState("")
  const [password, setPassword] = useState("")

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

  const handleEmailLogin = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    setLoading(true)

    try {
      await api.auth.emailLogin({ email, password })
      const user = await refreshUser()
      if (user) {
        console.info("[Auth] email login successful", { userId: user.id })
        window.location.href = "/dash"
      } else {
        setError("Login succeeded but could not load session")
      }
    } catch (err: unknown) {
      const code =
        err && typeof err === "object" && "error" in err
          ? String((err as { error: unknown }).error)
          : ""
      setError(code ? friendlyError(code) : "Login failed")
    } finally {
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
          {error && (
            <div className="mb-4 rounded bg-red-100 p-3 text-sm text-red-800 dark:bg-red-900/30 dark:text-red-400">
              {error}
            </div>
          )}

          <form onSubmit={handleEmailLogin} className="space-y-3">
            <div className="space-y-1.5">
              <Label htmlFor="email">Email</Label>
              <Input
                id="email"
                type="email"
                placeholder="you@example.com"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
              />
            </div>
            <div className="space-y-1.5">
              <div className="flex items-center justify-between">
                <Label htmlFor="password">Password</Label>
                <Link
                  to="/forgot-password"
                  className="text-xs text-zinc-500 underline underline-offset-2 hover:text-zinc-700 dark:text-zinc-400 dark:hover:text-zinc-200"
                >
                  Forgot password?
                </Link>
              </div>
              <Input
                id="password"
                type="password"
                placeholder="Your password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
              />
            </div>
            <Button
              type="submit"
              disabled={loading}
              className="w-full"
              variant="orange"
            >
              {loading ? "Signing in\u2026" : "Sign in"}
            </Button>
          </form>

          <p className="mt-2 text-center text-xs text-zinc-500">
            Don&apos;t have an account?{" "}
            <Link
              to="/register"
              className="font-medium text-zinc-700 underline underline-offset-2 hover:text-zinc-900 dark:text-zinc-300 dark:hover:text-zinc-100"
            >
              Register
            </Link>
          </p>

          <div className="relative my-4">
            <div className="absolute inset-0 flex items-center">
              <div className="w-full border-t border-zinc-200 dark:border-zinc-800" />
            </div>
            <div className="relative flex justify-center text-xs">
              <span className="bg-card px-2 text-muted-foreground">
                or continue with
              </span>
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
              <span className="bg-card px-2 text-muted-foreground">
                preferred method
              </span>
            </div>
          </div>

          <button
            onClick={handleHackclubLogin}
            disabled={loading}
            className="flex w-full items-center justify-center gap-2 rounded-lg bg-hackclub-500 py-2 text-sm font-medium text-white hover:bg-hackclub-600 disabled:bg-hackclub-400"
          >
            <HackClubIcon className="h-6 w-6" />
            {loading ? "Redirecting\u2026" : "Sign in with Hack Club"}
          </button>

          <Link
            to="/"
            className="mt-4 flex w-full items-center justify-center gap-1.5 text-sm text-zinc-500 underline underline-offset-2 hover:text-zinc-700 dark:text-zinc-400 dark:hover:text-zinc-200"
          >
            &larr; Back to home
          </Link>
        </CardContent>
      </Card>
    </div>
  )
}
