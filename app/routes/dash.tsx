import { SidebarProvider, SidebarTrigger } from "~/components/ui/sidebar"
import { AppSidebar } from "~/components/app-sidebar"


export default function Dashboard() {
  return (
        <div className="flex-1 p-6">
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
            <div className="flex-col items-center gap-4">
              <h2 className="mt-8 text-xl font-bold">Everything you see on these pages is a temporary placeholder data used by the developer (<span className="text-blue-400 decoration-dashed"><a href="https://vejas.zip">vejas.zip</a></span>) to develop the dashboard UI for HackFlare.</h2>
              <h3 className="pt-4 text-gray-400 text-lg">No data is real. And the site is not yet functioning.</h3>
            </div>
          </div>

  )
}
