import { useNavigate } from "react-router"
import { Button } from "~/components/ui/button"

export default function NotFound() {
  const navigate = useNavigate()

  return (
    <div className="min-h-screen flex flex-col items-center justify-center bg-background px-4">
      <div className="text-center max-w-md">
        <div className="mb-8">
          <div className="text-8xl font-bold text-red-500 mb-4">404</div>
          <h1 className="text-3xl font-bold tracking-tight mb-2">Page not found</h1>
          <p className="text-muted-foreground text-lg">
            Sorry, we couldn't find the page you're looking for.
          </p>
        </div>

        <div className="flex gap-3 justify-center">
          <Button variant="default" onClick={() => navigate("/")}>
            Go home
          </Button>
          <Button variant="outline" onClick={() => navigate(-1)}>
            Go back
          </Button>
        </div>
      </div>
    </div>
  )
}
