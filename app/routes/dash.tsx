import { SidebarProvider, SidebarTrigger } from "~/components/ui/sidebar"
import { AppSidebar } from "~/components/app-sidebar"
import { DarkModeToggle } from "~/components/dark-mode-toggle"

export default function Dashboard() {
  return (
    <SidebarProvider>
      <AppSidebar />
      <main className="flex-1 flex flex-col bg-zinc-50 dark:bg-zinc-950">
        <header className="sticky top-0 z-50 border-b border-zinc-200/60 dark:border-zinc-800/60 bg-white/72 dark:bg-zinc-900/72 backdrop-blur-md">
          <div className="flex items-center justify-between gap-4 px-6 h-17">
            <div className="flex items-center gap-4">
              <SidebarTrigger />
              <h1 className="text-xl font-bold dark:text-white">Dashboard</h1>
            </div>
            <DarkModeToggle />
          </div>
        </header>
        <div className="flex-1 p-6">
          <div className="mx-auto max-w-4xl">
            <h2 className="text-3xl font-bold mb-8 dark:text-white">Dashboard</h2>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              <div className="border border-zinc-200 dark:border-zinc-800 rounded-lg bg-white dark:bg-zinc-900 p-6">
                <h3 className="font-semibold mb-2 dark:text-white">Domains</h3>
                <p className="text-zinc-600 dark:text-zinc-400 text-sm">Manage your DNS domains.</p>
              </div>
              <div className="border border-zinc-200 dark:border-zinc-800 rounded-lg bg-white dark:bg-zinc-900 p-6">
                <h3 className="font-semibold mb-2 dark:text-white">DNS Records</h3>
                <p className="text-zinc-600 dark:text-zinc-400 text-sm">Configure DNS records.</p>
              </div>
              <div className="border border-zinc-200 dark:border-zinc-800 rounded-lg bg-white dark:bg-zinc-900 p-6">
                <h3 className="font-semibold mb-2 dark:text-white">Settings</h3>
                <p className="text-zinc-600 dark:text-zinc-400 text-sm">Account & API settings.</p>
              </div>
            </div>
          </div>
        </div>
      </main>
    </SidebarProvider>
  )
}
