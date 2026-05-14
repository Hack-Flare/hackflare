import { type ColumnDef } from "@tanstack/react-table"
import { Button } from "~/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "~/components/ui/dropdown-menu"
import { MoreHorizontal, Pencil, Trash2 } from "lucide-react"

export type DnsRecord = {
  id: number
  name: string
  type: "A" | "CNAME" | "MX" | "AAAA" | "TXT" | "NS"
  value: string
  ttl: number
  status: "active" | "pending" | "inactive"
}

export const columns: ColumnDef<DnsRecord>[] = [
  {
    accessorKey: "name",
    header: "Name",
    cell: ({ row }) => (
      <span className="font-semibold text-white">{row.getValue("name")}</span>
    ),
  },
  {
    accessorKey: "type",
    header: "Type",
    cell: ({ row }) => (
      <span className="rounded bg-zinc-700 px-2 py-1 font-mono text-xs text-zinc-300">
        {row.getValue("type")}
      </span>
    ),
  },
  {
    accessorKey: "value",
    header: "Value",
    cell: ({ row }) => (
      <span className="font-mono text-xs text-zinc-400">
        {row.getValue("value")}
      </span>
    ),
  },
  {
    accessorKey: "ttl",
    header: "TTL",
    cell: ({ row }) => (
      <span className="text-zinc-400">{row.getValue("ttl")}s</span>
    ),
  },
  {
    accessorKey: "status",
    header: "Status",
    cell: ({ row }) => {
      const status = row.getValue("status") as DnsRecord["status"]
      return (
        <span
          className={`rounded px-2 py-1 text-xs font-medium ${
            status === "active"
              ? "border border-green-700 bg-green-900/40 text-green-400"
              : "border border-yellow-700 bg-yellow-900/40 text-yellow-400"
          }`}
        >
          {status}
        </span>
      )
    },
  },
  {
    id: "actions",
    cell: ({ row }) => {
      const record = row.original

      return (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon">
              <MoreHorizontal className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onClick={() => console.log("edit", record)}>
              <Pencil className="h-4 w-4 mr-2" />
              Edit
            </DropdownMenuItem>
            <DropdownMenuItem
              onClick={() => console.log("delete", record)}
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
