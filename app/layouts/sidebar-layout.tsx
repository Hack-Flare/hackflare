import { Outlet } from "react-router"
import { SidebarProvider, SidebarTrigger } from "~/components/ui/sidebar"
import { AppSidebar } from "~/components/app-sidebar"
import { DarkModeToggle } from "~/components/dark-mode-toggle"

export default function SidebarLayout() {
  return (
    <SidebarProvider>
      <AppSidebar />
      <main className="flex flex-1 flex-col min-h-screen">
        <header className="flex h-12 items-center gap-3 border-b px-4">
          <SidebarTrigger />
          <DarkModeToggle />
        </header>
        <div className="flex-1 p-6">
          <Outlet />  {/* ← your page renders here */}
          
        </div>
        <p className="pl-4 pb-4 text-gray-400">Data is fake.</p>
      </main>
    </SidebarProvider>
  )
}