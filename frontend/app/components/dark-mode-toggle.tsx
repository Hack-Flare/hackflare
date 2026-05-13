import { Moon, Sun } from "lucide-react"
import { useDarkMode } from "./dark-mode-provider"
import { Button } from "./ui/button"

export function DarkModeToggle() {
  const { theme, toggleTheme } = useDarkMode()

  return (
    <Button
      onClick={toggleTheme}
      aria-label="Toggle dark mode"
      variant={"ghost"}
      size={"icon-sm"}
    >
      {theme === "light" ? (
        <Moon />
      ) : (
        <Sun />
      )}
    </Button>
  )
}
