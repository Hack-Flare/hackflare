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

export const columns: ColumnDef<FirewallRule>[] = [
  {
    accessorKey: "type",
    header: "Type",
    cell: ({ row }) => (
      <span className="font-semibold text-white">{row.getValue("type")}</span>
    ),
  },
  {
    accessorKey: "pattern",
    header: "Pattern",
    cell: ({ row }) => (
      <span className="font-mono text-xs text-zinc-400">{row.getValue("pattern")}</span>
    ),
  },
  {
    accessorKey: "action",
    header: "Action",
    cell: ({ row }) => (
      <span className="px-2 py-1 rounded text-xs font-medium bg-zinc-800 text-zinc-300">
        {row.getValue("action")}
      </span>
    ),
  },
  {
    accessorKey: "enabled",
    header: "Status",
    cell: ({ row }) => {
      const enabled = row.getValue("enabled") as boolean
      return (
        <span className={`inline-flex h-2 w-2 rounded-full ${enabled ? "bg-green-500" : "bg-zinc-600"}`} />
      )
    },
  },
  {
    accessorKey: "requests",
    header: "Requests",
    cell: ({ row }) => (
      <span className="text-zinc-400">
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