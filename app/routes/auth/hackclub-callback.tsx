import { useEffect, useState } from "react"
import { useSearchParams, useNavigate } from "react-router"
import { useAuth } from "~/lib/auth-context"
import { api } from "~/lib/api"

export default function HackclubCallback() {
  const [searchParams] = useSearchParams()
  const navigate = useNavigate()
  const { login } = useAuth()
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const code = searchParams.get("code")

    if (!code) {
      console.error("[OAuth] No authorization code received")
      setError("No authorization code received")
      setTimeout(() => navigate("/auth?error=no_code"), 2000)
      return
    }

    console.log("[OAuth] Processing callback")
    api.auth
      .hackclubCallback(code)
      .then((data) => {
        console.log("[OAuth] Success")
        login(data.token, data.user)
        navigate("/dash")
      })
      .catch((err) => {
        console.error("[OAuth] Failed:", err.error)
        setError(err.error || "OAuth callback failed")
        setTimeout(() => navigate("/auth?error=oauth_failed"), 2000)
      })
  }, [searchParams, navigate, login])

  return (
    <div className="flex items-center justify-center min-h-screen bg-zinc-50 dark:bg-zinc-950">
      <div className="text-center">
        {error ? (
          <>
            <p className="text-red-600 dark:text-red-400 mb-2">{error}</p>
            <p className="text-sm text-zinc-600 dark:text-zinc-400">
              Redirecting to login...
            </p>
          </>
        ) : (
          <>
            <div className="animate-spin rounded-full h-8 w-8 border b-2 border-orange-500 border-t-transparent mx-auto mb-4"></div>
            <p className="text-zinc-600 dark:text-zinc-400">
              Completing sign in...
            </p>
          </>
        )}
      </div>
    </div>
  )
}
