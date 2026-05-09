import { Moon, Sun } from "lucide-react"
import { useDarkMode } from "./dark-mode-provider"

export function DarkModeToggle() {
  const { theme, toggleTheme } = useDarkMode()

  return (
    <button
      onClick={toggleTheme}
      className="p-2 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-lg transition-colors"
      aria-label="Toggle dark mode"
    >
      {theme === "light" ? (
        <Moon className="h-5 w-5 text-slate-600 dark:text-slate-300" />
      ) : (
        <Sun className="h-5 w-5 text-slate-300" />
      )}
    </button>
  )
}
