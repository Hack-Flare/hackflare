import { useNavigate } from "react-router"
import { Button } from "~/components/ui/button"

export default function Unauthorized() {
  const navigate = useNavigate()

  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-background px-4">
      <div className="max-w-md text-center">
        <div className="mb-8">
          <div className="mb-4 text-8xl font-bold text-yellow-600">403</div>
          <h1 className="mb-2 text-3xl font-bold tracking-tight">
            Access denied
          </h1>
          <p className="text-lg text-muted-foreground">
            You don't have permission to access this resource.
          </p>
        </div>

        <div className="flex justify-center gap-3">
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
