import { Network } from "lucide-react"

export default function Tunnel() {
  return (
    <div className="flex flex-1 items-center justify-center">
      <div className="mx-auto max-w-md text-center">
        <div className="mx-auto mb-6 flex h-16 w-16 items-center justify-center rounded-2xl border border-zinc-200 bg-zinc-50 dark:border-zinc-800 dark:bg-zinc-900">
          <Network className="h-8 w-8 text-zinc-400" />
        </div>
        <h1 className="mb-2 text-2xl font-bold text-zinc-900 dark:text-white">
          Tunnel
        </h1>
        <p className="text-sm text-zinc-600 dark:text-zinc-400">
          Create secure tunnels to expose local services to the internet. This
          feature is coming soon.
        </p>
      </div>
    </div>
  )
}
