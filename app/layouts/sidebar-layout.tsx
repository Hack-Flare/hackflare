import { Outlet, useNavigate } from "react-router"
import { useEffect } from "react"
import { SidebarProvider, SidebarTrigger } from "~/components/ui/sidebar"
import { AppSidebar } from "~/components/app-sidebar"
import { DarkModeToggle } from "~/components/dark-mode-toggle"
import { useAuth } from "~/lib/auth-context"

export default function SidebarLayout() {
  const navigate = useNavigate()
  const { token, ready } = useAuth()

  useEffect(() => {
    if (ready && !token) {
      navigate("/auth")
    }
  }, [ready, token, navigate])

  if (!ready || !token) {
    return null
  }

  return (
    <SidebarProvider>
      <AppSidebar />
      <main className="flex flex-1 flex-col min-h-screen">
        <header className="flex h-12 items-center gap-3 border-b px-4">
          <SidebarTrigger />
          <DarkModeToggle />
        </header>
        <div className="border-b border-orange-200 bg-orange-50 px-6 py-3 text-sm text-orange-900 dark:border-orange-900/40 dark:bg-orange-950/30 dark:text-orange-200">
          Demo mode. Pages use placeholder data and layout only. Hook real API later.
        </div>
        <div className="flex-1 p-6">
          <Outlet />
        </div>
      </main>
    </SidebarProvider>
  )
}