import { useNavigate } from "react-router"
import { Button } from "~/components/ui/button"

export default function Unauthorized() {
  const navigate = useNavigate()

  return (
    <div className="min-h-screen flex flex-col items-center justify-center bg-background px-4">
      <div className="text-center max-w-md">
        <div className="mb-8">
          <div className="text-8xl font-bold text-orange-600 mb-4">401</div>
          <h1 className="text-3xl font-bold tracking-tight mb-2">Not authenticated</h1>
          <p className="text-muted-foreground text-lg">
            You need to log in to access this page.
          </p>
        </div>

        <div className="flex gap-3 justify-center">
          <Button variant="default" onClick={() => navigate("/login")}>
            Sign in
          </Button>
          <Button variant="outline" onClick={() => navigate("/")}>
            Go home
          </Button>
        </div>
      </div>
    </div>
  )
}
