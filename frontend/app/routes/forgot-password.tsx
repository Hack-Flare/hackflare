import { useState } from "react"
import { Link } from "react-router"
import { api, friendlyError } from "../lib/api"
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

export default function ForgotPassword() {
  const [email, setEmail] = useState("")
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [sent, setSent] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    setLoading(true)

    try {
      await api.auth.forgotPassword({ email })
      setSent(true)
    } catch (err: unknown) {
      const code =
        err && typeof err === "object" && "error" in err
          ? String((err as { error: unknown }).error)
          : ""
      setError(code ? friendlyError(code) : "Request failed")
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="relative flex min-h-screen items-center justify-center overflow-hidden bg-zinc-50 dark:bg-zinc-950">
      <div className="absolute inset-0 bg-[linear-gradient(to_right,#80808012_1px,transparent_1px),linear-gradient(to_bottom,#80808012_1px,transparent_1px)] bg-size-[24px_24px]" />

      <Card className="relative w-full max-w-md">
        <CardHeader>
          <CardTitle>Reset your password</CardTitle>
          <CardDescription>
            Enter your email and we&apos;ll send you a reset link
          </CardDescription>
        </CardHeader>
        <CardContent>
          {error && (
            <div className="mb-4 rounded bg-red-100 p-3 text-sm text-red-800 dark:bg-red-900/30 dark:text-red-400">
              {error}
            </div>
          )}

          {sent ? (
            <div className="space-y-3">
              <div className="rounded-lg border border-green-200 bg-green-50 p-4 text-sm text-green-800 dark:border-green-900/50 dark:bg-green-950/50 dark:text-green-400">
                If an account with that email exists, we&apos;ve sent a password
                reset link. Please check your inbox (and spam folder).
              </div>
              <Link
                to="/login"
                className="flex w-full items-center justify-center gap-1.5 text-sm text-zinc-500 underline underline-offset-2 hover:text-zinc-700 dark:text-zinc-400 dark:hover:text-zinc-200"
              >
                &larr; Back to sign in
              </Link>
            </div>
          ) : (
            <form onSubmit={handleSubmit} className="space-y-3">
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
              <Button
                type="submit"
                disabled={loading}
                className="w-full"
                variant="orange"
              >
                {loading ? "Sending\u2026" : "Send reset link"}
              </Button>
              <Link
                to="/login"
                className="flex w-full items-center justify-center gap-1.5 text-sm text-zinc-500 underline underline-offset-2 hover:text-zinc-700 dark:text-zinc-400 dark:hover:text-zinc-200"
              >
                &larr; Back to sign in
              </Link>
            </form>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
