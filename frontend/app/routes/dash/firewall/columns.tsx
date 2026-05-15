import { type ColumnDef } from "@tanstack/react-table"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "~/components/ui/dropdown-menu"
import { Button } from "~/components/ui/button"
import { MoreHorizontal, Pencil, Trash2 } from "lucide-react"

export type FirewallRule = {
  id: number
  type: "Blocklist" | "Allowlist" | "Rate Limit" | "Bot Challenge"
  pattern: string
  action: "Block" | "Allow" | "Limit" | "Challenge"
  enabled: boolean
  requests: number
}

const actionStyles: Record<FirewallRule["action"], string> = {
  Block:     "bg-red-100 text-red-700 border border-red-200 dark:bg-red-900/30 dark:text-red-400 dark:border-red-700",
  Allow:     "bg-green-100 text-green-700 border border-green-200 dark:bg-green-900/30 dark:text-green-400 dark:border-green-700",
  Limit:     "bg-yellow-100 text-yellow-700 border border-yellow-200 dark:bg-yellow-900/30 dark:text-yellow-400 dark:border-yellow-700",
  Challenge: "bg-blue-100 text-blue-700 border border-blue-200 dark:bg-blue-900/30 dark:text-blue-400 dark:border-blue-700",
}

export const columns: ColumnDef<FirewallRule>[] = [
  {
    accessorKey: "type",
    header: "Type",
    cell: ({ row }) => (
      <span className="font-semibold text-zinc-900 dark:text-white">{row.getValue("type")}</span>
    ),
  },
  {
    accessorKey: "pattern",
    header: "Pattern",
    cell: ({ row }) => (
      <span className="font-mono text-xs text-zinc-600 dark:text-zinc-400">{row.getValue("pattern")}</span>
    ),
  },
  {
    accessorKey: "action",
    header: "Action",
    cell: ({ row }) => {
      const action = row.getValue("action") as FirewallRule["action"]
      return (
        <span className={`px-2 py-1 rounded text-xs font-medium ${actionStyles[action]}`}>
          {action}
        </span>
      )
    },
  },
  {
    accessorKey: "enabled",
    header: "Status",
    cell: ({ row }) => {
      const enabled = row.getValue("enabled") as boolean
      return (
        <div className="flex items-center gap-2">
          <span className={`inline-flex h-2 w-2 rounded-full ${enabled ? "bg-green-500" : "bg-zinc-400 dark:bg-zinc-600"}`} />
          <span className="text-xs text-zinc-600 dark:text-zinc-400">{enabled ? "Active" : "Disabled"}</span>
        </div>
      )
    },
  },
  {
    accessorKey: "requests",
    header: "Requests",
    cell: ({ row }) => (
      <span className="text-zinc-600 dark:text-zinc-400">
        {(row.getValue("requests") as number).toLocaleString()}
      </span>
    ),
  },
  {
    id: "actions",
    cell: ({ row }) => {
      const rule = row.original
      return (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon">
              <MoreHorizontal className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onClick={() => console.log("edit", rule)}>
              <Pencil className="h-4 w-4 mr-2" />
              Edit
            </DropdownMenuItem>
            <DropdownMenuItem
              onClick={() => console.log("delete", rule)}
              className="text-red-500 focus:text-red-500"
            >
              <Trash2 className="h-4 w-4 mr-2" />
              Delete
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      )
    },
  },
]