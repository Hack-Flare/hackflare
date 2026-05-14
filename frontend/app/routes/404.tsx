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
      className="min-h-screen w-full flex flex-col items-center justify-center bg-background text-foreground px-4"
      style={isDark ? {
        backgroundColor: "rgb(18, 18, 18)",
        color: "rgb(250, 250, 250)"
      } : undefined}
    >
      <div className="text-center max-w-md space-y-8">
        <div className="space-y-4">
          <div className="text-9xl font-bold bg-linear-to-br from-red-500 to-red-600 dark:from-red-400 dark:to-red-500 bg-clip-text text-transparent">
            404
          </div>
          <h1 className="text-4xl font-bold tracking-tight">Page not found</h1>
          <p className="text-muted-foreground text-lg">
            Sorry, we couldn't find the page you're looking for.
          </p>
        </div>

        <div className="flex gap-4 justify-center flex-col sm:flex-row">
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
