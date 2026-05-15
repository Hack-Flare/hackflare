import {
  Links,
  Meta,
  Outlet,
  Scripts,
  ScrollRestoration,
  isRouteErrorResponse,
} from "react-router"

import type { Route } from "./+types/root"
import { DarkModeProvider } from "./components/dark-mode-provider"
import { AuthProvider } from "./lib/auth-context"
import "./app.css"

export function Layout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <Meta />
        <Links />
        <script
          dangerouslySetInnerHTML={{
            __html: `
              try {
                const theme = localStorage.getItem("theme") || "dark"
                if (theme === "dark") {
                  document.documentElement.classList.add("dark")
                }
              } catch (e) {}
            `,
          }}
        />
      </head>
      <body>
        {children}
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  )
}

export default function App() {
  return (
    <AuthProvider>
      <DarkModeProvider>
        <Outlet />
      </DarkModeProvider>
    </AuthProvider>
  )
}

export function ErrorBoundary({ error }: Route.ErrorBoundaryProps) {
  let status = 500
  let message = "Something went wrong"
  let details = "An unexpected error occurred."
  let stack: string | undefined

  if (isRouteErrorResponse(error)) {
    status = error.status
    message = getErrorMessage(error.status)
    details = error.statusText || getErrorDetails(error.status)
  } else if (import.meta.env.DEV && error && error instanceof Error) {
    details = error.message
    stack = error.stack
  }

  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-background px-4">
      <div className="max-w-lg text-center">
        <div className="mb-8">
          <div className="mb-4 text-8xl font-bold text-muted-foreground">
            {status}
          </div>
          <h1 className="mb-2 text-3xl font-bold tracking-tight">{message}</h1>
          <p className="text-lg text-muted-foreground">{details}</p>
        </div>

        {stack && import.meta.env.DEV && (
          <div className="mt-8 rounded-lg bg-muted p-4 text-left">
            <p className="mb-2 text-sm font-semibold text-muted-foreground">
              Stack Trace:
            </p>
            <pre className="max-h-40 overflow-x-auto text-xs text-muted-foreground">
              <code>{stack}</code>
            </pre>
          </div>
        )}

        <div className="mt-8 flex justify-center gap-3">
          <a
            href="/"
            className="inline-flex h-10 items-center justify-center rounded-md bg-primary px-8 text-sm font-medium text-primary-foreground hover:bg-primary/90"
          >
            Go home
          </a>
        </div>
      </div>
    </div>
  )
}

function getErrorMessage(status: number): string {
  switch (status) {
    case 404:
      return "Page not found"
    case 500:
      return "Server error"
    case 503:
      return "Service unavailable"
    case 401:
      return "Unauthorized"
    case 403:
      return "Forbidden"
    default:
      return "Error"
  }
}

function getErrorDetails(status: number): string {
  switch (status) {
    case 404:
      return "The page you're looking for doesn't exist."
    case 500:
      return "Something went wrong on our end. We're working to fix it."
    case 503:
      return "The service is temporarily unavailable. Please try again later."
    case 401:
      return "You need to be logged in to access this page."
    case 403:
      return "You don't have permission to access this resource."
    default:
      return "An unexpected error occurred."
  }
}
