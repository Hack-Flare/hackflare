import { useState, useEffect } from "react"
import { AlertTriangle, Info, CheckCircle, Filter } from "lucide-react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"
import { Button } from "~/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "~/components/ui/dropdown-menu"
import { DataTable } from "./data-table"
import { columns, type LogEntry } from "./columns"
import { api } from "~/lib/api"

type Level = LogEntry["level"]

export default function Logs() {
  const [logs, setLogs] = useState<LogEntry[]>([])
  const [errorsToday, setErrorsToday] = useState(0)
  const [warningsToday, setWarningsToday] = useState(0)
  const [infoToday, setInfoToday] = useState(0)
  const [loading, setLoading] = useState(true)
  const [activeLevels, setActiveLevels] = useState<Set<Level>>(
    new Set(["info", "warning", "error"])
  )

  useEffect(() => {
    api.logs
      .queryLogs()
      .then((res) => {
        setLogs(res.logs)
        setErrorsToday(res.summary.errors_today)
        setWarningsToday(res.summary.warnings_today)
        setInfoToday(res.summary.info_today)
      })
      .catch(console.error)
      .finally(() => setLoading(false))
  }, [])

  const toggleLevel = (level: Level) => {
    setActiveLevels((prev) => {
      const next = new Set(prev)
      next.has(level) ? next.delete(level) : next.add(level)
      return next
    })
  }

  const filteredLogs = logs.filter((log) => activeLevels.has(log.level))

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Logs</h1>
          <p className="mt-2 text-zinc-600 dark:text-zinc-400">
            View recent events, errors + request history
          </p>
        </div>

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="orange" className="flex items-center gap-2">
              <Filter className="h-4 w-4" />
              Filter
              {activeLevels.size < 3 && (
                <span className="ml-1 rounded-full bg-white/20 px-1.5 text-xs">
                  {activeLevels.size}
                </span>
              )}
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-40">
            <DropdownMenuLabel>Log Level</DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuCheckboxItem
              checked={activeLevels.has("info")}
              onCheckedChange={() => toggleLevel("info")}
            >
              <span className="flex items-center gap-2">
                <Info className="h-3 w-3 text-green-500" />
                Info
              </span>
            </DropdownMenuCheckboxItem>
            <DropdownMenuCheckboxItem
              checked={activeLevels.has("warning")}
              onCheckedChange={() => toggleLevel("warning")}
            >
              <span className="flex items-center gap-2">
                <AlertTriangle className="h-3 w-3 text-yellow-500" />
                Warning
              </span>
            </DropdownMenuCheckboxItem>
            <DropdownMenuCheckboxItem
              checked={activeLevels.has("error")}
              onCheckedChange={() => toggleLevel("error")}
            >
              <span className="flex items-center gap-2">
                <AlertTriangle className="h-3 w-3 text-red-500" />
                Error
              </span>
            </DropdownMenuCheckboxItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <AlertTriangle className="h-4 w-4" />
              Errors Today
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">
              {loading ? "..." : errorsToday}
            </p>
            <p className="mt-1 text-xs text-red-600 dark:text-red-500">
              DNS errors
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <AlertTriangle className="h-4 w-4" />
              Warnings
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">
              {loading ? "..." : warningsToday}
            </p>
            <p className="mt-1 text-xs text-yellow-600 dark:text-yellow-500">
              NXDOMAIN responses
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <CheckCircle className="h-4 w-4" />
              Info Events
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">
              {loading ? "..." : infoToday}
            </p>
            <p className="mt-1 text-xs text-green-600 dark:text-green-500">
              Successful queries
            </p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Recent Events</CardTitle>
          <CardDescription>Logs + errors from past 24 hours</CardDescription>
        </CardHeader>
        <CardContent>
          {loading ? (
            <p className="text-sm text-zinc-500">Loading...</p>
          ) : (
            <DataTable columns={columns} data={filteredLogs} />
          )}
        </CardContent>
      </Card>
    </div>
  )
}
