import { useState } from "react"
import { Link, Navigate, useNavigate } from "react-router"
import { useAuth } from "../lib/auth-context"
import { api } from "../lib/api"
import { HackClubIcon } from "../components/icons/hackclub"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../components/ui/card"

const DEMO_EMAIL = "tes@123.com"
const DEMO_PASSWORD = "1234"
const DEMO_USER = {
  id: 1,
  email: DEMO_EMAIL,
  name: "Tes Toaster",
  is_admin: false,
}

export default function Login() {
  const navigate = useNavigate()
  const { token, login, logout } = useAuth()
  const [tab, setTab] = useState<"password" | "hackclub">("password")
  const [email, setEmail] = useState("")
  const [password, setPassword] = useState("")
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  if (token) {
    return <Navigate to="/dash" replace />
  }

  const handlePasswordLogin = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    setLoading(true)
    console.log("[Login] Attempting password login:", email)

    try {
      if (email === DEMO_EMAIL && password === DEMO_PASSWORD) {
        login("demo_token", DEMO_USER)
        navigate("/dash")
        return
      }

      const data = await api.auth.login({ email, password })
      login(data.token, data.user)
      console.log("[Login] Success")
      navigate("/dash")
    } catch (err: any) {
      console.error("[Login] Failed:", err.error)
      setError(err.error || "Login failed")
    } finally {
      setLoading(false)
    }
  }

  const handleHackclubLogin = async () => {
    setError(null)
    setLoading(true)
    console.log("[Login] Attempting Hack Club login")

    try {
      // TODO: Remove this demo login and implement real Hack Club OAuth
      login("demo_token", DEMO_USER)
      navigate("/dash")
      return

      // const data = await api.auth.hackclubUrl()
      // console.log("[Login] Redirecting to Hack Club")
      // window.location.href = data.url
    } catch (err: any) {
      console.error("[Login] Failed:", err.error)
      setError(err.error || "Failed to get Hack Club login URL")
      setLoading(false)
    }
  }

  return (
    <div className="flex items-center justify-center min-h-screen bg-zinc-50 dark:bg-zinc-950">
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
            <div className="mb-4 p-3 rounded bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400 text-sm">
              {error}
            </div>
          )}

          <div className="flex gap-2 mb-6 border-b border-zinc-200 dark:border-zinc-800">
            {/* <button
              onClick={() => setTab("password")}
              className={`pb-2 px-1 font-medium text-sm ${
                tab === "password"
                  ? "text-orange-500 border-b-2 border-orange-500"
                  : "text-zinc-600 dark:text-zinc-400"
              }`}
            >
              Email & Password
            </button> */}
            <button
              onClick={() => setTab("hackclub")}
              className="pb-2 px-1 font-medium text-sm text-hackclub-500 border-b-2 border-hackclub-500"
            >
              Hack Club
            </button>
          </div>

          {/* Email & Password auth - implement later */}
          {/* {tab === "password" ? (
            <form onSubmit={handlePasswordLogin} className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-1">Email</label>
                <input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="you@example.com"
                  className="w-full px-3 py-2 border border-zinc-200 dark:border-zinc-800 rounded-lg bg-white dark:bg-zinc-900 text-sm"
                  required
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Password</label>
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="••••••••"
                  className="w-full px-3 py-2 border border-zinc-200 dark:border-zinc-800 rounded-lg bg-white dark:bg-zinc-900 text-sm"
                  required
                />
              </div>
              <button
                type="submit"
                disabled={loading}
                className="w-full bg-orange-500 hover:bg-orange-600 disabled:bg-orange-400 text-white py-2 rounded-lg font-medium text-sm"
              >
                {loading ? "Signing in..." : "Sign in"}
              </button>
            </form>
          ) : (
            <button
              onClick={handleHackclubLogin}
              disabled={loading}
              className="w-full bg-hackclub-500 hover:bg-hackclub-600 disabled:bg-hackclub-400 text-white py-2 rounded-lg font-medium text-sm flex items-center justify-center gap-2"
            >
              <HackClubIcon className="h-6 w-6" />
              {loading ? "Redirecting..." : "Sign in with Hack Club"}
            </button>
          )} */}

          <button
            onClick={handleHackclubLogin}
            disabled={loading}
            className="w-full bg-hackclub-500 hover:bg-hackclub-600 disabled:bg-hackclub-400 text-white py-2 rounded-lg font-medium text-sm flex items-center justify-center gap-2"
          >
            <HackClubIcon className="h-6 w-6" />
            {loading ? "Redirecting..." : "Sign in with Hack Club"}
          </button>

          {/*<p className="text-sm text-zinc-600 dark:text-zinc-400 text-center mt-6">
            Don't have an account?{" "}
            <Link to="/register" className="text-orange-500 hover:text-orange-600 font-medium">
              Sign up
            </Link>
          </p>*/}
        </CardContent>
      </Card>
    </div>
  )
}
