import { useEffect, useState } from "react"
import { useSearchParams, useNavigate } from "react-router"
import { useAuth } from "~/lib/auth-context"

export default function HackclubCallback() {
  const [searchParams] = useSearchParams()
  const navigate = useNavigate()
  const { refreshUser } = useAuth()
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const returnTo = searchParams.get("returnTo") || "/dash"

    console.info("[Auth] callback loaded", { returnTo })

    refreshUser()
      .then((user) => {
        if (user) {
          console.info("[Auth] session ready after callback", {
            userId: user.id,
            returnTo,
          })
          navigate(returnTo, { replace: true })
          return
        }

        console.error("[Auth] callback finished without session user", {
          returnTo,
        })
        setError("Sign-in did not complete")
        setTimeout(
          () => navigate("/login?error=auth_failed", { replace: true }),
          2000
        )
      })
      .catch((error) => {
        console.error("[Auth] callback session refresh failed", {
          returnTo,
          error,
        })
        setError("Unable to load your session")
        setTimeout(
          () => navigate("/login?error=session_failed", { replace: true }),
          2000
        )
      })
  }, [searchParams, navigate, refreshUser])

  return (
    <div className="flex min-h-screen items-center justify-center bg-zinc-50 dark:bg-zinc-950">
      <div className="text-center">
        {error ? (
          <>
            <p className="mb-2 text-red-600 dark:text-red-400">{error}</p>
            <p className="text-sm text-zinc-600 dark:text-zinc-400">
              Redirecting to login...
            </p>
          </>
        ) : (
          <>
            <div className="b-2 mx-auto mb-4 h-8 w-8 animate-spin rounded-full border border-orange-500 border-t-transparent"></div>
            <p className="text-zinc-600 dark:text-zinc-400">
              Completing sign in...
            </p>
          </>
        )}
      </div>
    </div>
  )
}
