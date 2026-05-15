import { Outlet, useNavigate, useLocation } from "react-router"
import { useEffect } from "react"
import { motion } from "framer-motion"
import { SidebarProvider, SidebarTrigger } from "~/components/ui/sidebar"
import { AppSidebar } from "~/components/app-sidebar"
import { DarkModeToggle } from "~/components/dark-mode-toggle"
import { useAuth } from "~/lib/auth-context"

export default function SidebarLayout() {
  const navigate = useNavigate()
  const location = useLocation()
  const { user, ready } = useAuth()

  useEffect(() => {
    if (ready && !user) {
      navigate("/auth")
    }
  }, [ready, user, navigate])

  if (!ready || !user) {
    return null
  }

  return (
    <SidebarProvider>
      <AppSidebar />
      <main className="flex min-h-screen flex-1 flex-col">
        <header className="flex h-12 items-center gap-3 border-b px-4">
          <SidebarTrigger />
          <DarkModeToggle />
        </header>
        <div className="border-b border-orange-200 bg-orange-50 px-6 py-3 text-sm text-orange-900 dark:border-orange-900/40 dark:bg-orange-950/30 dark:text-orange-200">
          Signed in with your Hack Club session.
        </div>
        <div className="flex-1 p-6">
          <motion.div
            key={location.pathname}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 0.2, ease: "easeOut" }}
          >
            <Outlet />
          </motion.div>
        </div>
      </main>
    </SidebarProvider>
  )
}
