import { type ColumnDef } from "@tanstack/react-table"
import { AlertTriangle, Info } from "lucide-react"

export type LogEntry = {
  id: number
  timestamp: string
  level: "info" | "warning" | "error"
  path: string
  status: number
  ms: number
}

const levelColor = (level: string) => {
  switch (level) {
    case "error":   return "bg-red-100 text-red-700 border border-red-200 dark:bg-red-900/30 dark:text-red-400 dark:border-red-700"
    case "warning": return "bg-yellow-100 text-yellow-700 border border-yellow-200 dark:bg-yellow-900/30 dark:text-yellow-400 dark:border-yellow-700"
    default:        return "bg-green-100 text-green-700 border border-green-200 dark:bg-green-900/30 dark:text-green-400 dark:border-green-700"
  }
}

const levelIcon = (level: string) => {
  switch (level) {
    case "error":   return <AlertTriangle className="h-3 w-3" />
    case "warning": return <AlertTriangle className="h-3 w-3" />
    default:        return <Info className="h-3 w-3" />
  }
}

const statusColor = (status: number) => {
  if (status < 300) return "bg-green-100 text-green-700 border border-green-200 dark:bg-green-900/30 dark:text-green-400 dark:border-green-700"
  if (status < 400) return "bg-blue-100 text-blue-700 border border-blue-200 dark:bg-blue-900/30 dark:text-blue-400 dark:border-blue-700"
  if (status < 500) return "bg-yellow-100 text-yellow-700 border border-yellow-200 dark:bg-yellow-900/30 dark:text-yellow-400 dark:border-yellow-700"
  return "bg-red-100 text-red-700 border border-red-200 dark:bg-red-900/30 dark:text-red-400 dark:border-red-700"
}

export const columns: ColumnDef<LogEntry>[] = [
  {
    accessorKey: "timestamp",
    header: "Time",
    cell: ({ row }) => (
      <span className="text-xs text-zinc-600 dark:text-zinc-400">{row.getValue("timestamp")}</span>
    ),
  },
  {
    accessorKey: "level",
    header: "Level",
    cell: ({ row }) => {
      const level = row.getValue("level") as string
      return (
        <span className={`flex w-fit items-center gap-1 rounded px-2 py-1 text-xs font-medium ${levelColor(level)}`}>
          {levelIcon(level)}
          {level}
        </span>
      )
    },
  },
  {
    accessorKey: "path",
    header: "Path",
    cell: ({ row }) => (
      <span className="font-mono text-xs text-zinc-600 dark:text-zinc-400">{row.getValue("path")}</span>
    ),
  },
  {
    accessorKey: "status",
    header: "Status",
    cell: ({ row }) => {
      const status = row.getValue("status") as number
      return (
        <div className="flex justify-center">
          <span className={`rounded px-2 py-1 text-xs font-medium ${statusColor(status)}`}>
            {status}
          </span>
        </div>
      )
    },
  },
  {
    accessorKey: "ms",
    header: "Duration",
    cell: ({ row }) => (
      <div className="text-right text-zinc-600 dark:text-zinc-400">
        {row.getValue("ms")}ms
      </div>
    ),
  },
]