import { useNavigate } from "react-router"
import { useEffect, useState } from "react"
import { Button } from "~/components/ui/button"
import { ArrowLeft, Home } from "lucide-react"

export default function NotFound() {
  const navigate = useNavigate()
  const [isDark, setIsDark] = useState(false)

  useEffect(() => {
    const theme = localStorage.getItem("theme") || "dark"
    setIsDark(theme === "dark")
  }, [])

  return (
    <div
      className="flex min-h-screen w-full flex-col items-center justify-center bg-background px-4 text-foreground"
      style={
        isDark
          ? {
              backgroundColor: "rgb(18, 18, 18)",
              color: "rgb(250, 250, 250)",
            }
          : undefined
      }
    >
      <div className="max-w-md space-y-8 text-center">
        <div className="space-y-4">
          <div className="bg-linear-to-br from-red-500 to-red-600 bg-clip-text text-9xl font-bold text-transparent dark:from-red-400 dark:to-red-500">
            404
          </div>
          <h1 className="text-4xl font-bold tracking-tight">Page not found</h1>
          <p className="text-lg text-muted-foreground">
            Sorry, we couldn't find the page you're looking for.
          </p>
        </div>

        <div className="flex flex-col justify-center gap-4 sm:flex-row">
          <Button
            variant="default"
            onClick={() => navigate("/")}
            className="gap-2"
          >
            <Home className="h-4 w-4" />
            Go home
          </Button>
          <Button
            variant="outline"
            onClick={() => navigate(-1)}
            className="gap-2"
          >
            <ArrowLeft className="h-4 w-4" />
            Go back
          </Button>
        </div>
      </div>
    </div>
  )
}
