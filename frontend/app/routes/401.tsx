import { useNavigate } from "react-router"
import { Button } from "~/components/ui/button"

export default function Unauthorized() {
  const navigate = useNavigate()

  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-background px-4">
      <div className="max-w-md text-center">
        <div className="mb-8">
          <div className="mb-4 text-8xl font-bold text-orange-600">401</div>
          <h1 className="mb-2 text-3xl font-bold tracking-tight">
            Not authenticated
          </h1>
          <p className="text-lg text-muted-foreground">
            You need to log in to access this page.
          </p>
        </div>

        <div className="flex justify-center gap-3">
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
