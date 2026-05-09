import * as React from "react"

type Theme = "light" | "dark"

interface DarkModeContextType {
  theme: Theme
  toggleTheme: () => void
}

const DarkModeContext = React.createContext<DarkModeContextType | undefined>(
  undefined
)

export function DarkModeProvider({ children }: { children: React.ReactNode }) {
  const [theme, setTheme] = React.useState<Theme>(() => {
    if (typeof window === "undefined") return "light"
    return (localStorage.getItem("theme") as Theme) || "light"
  })

  React.useEffect(() => {
    const root = document.documentElement
    if (theme === "dark") {
      root.classList.add("dark")
    } else {
      root.classList.remove("dark")
    }
    localStorage.setItem("theme", theme)
  }, [theme])

  const toggleTheme = () => {
    setTheme((prev) => (prev === "light" ? "dark" : "light"))
  }

  return (
    <DarkModeContext.Provider value={{ theme, toggleTheme }}>
      {children}
    </DarkModeContext.Provider>
  )
}

export function useDarkMode() {
  const ctx = React.useContext(DarkModeContext)
  if (!ctx) throw new Error("useDarkMode outside DarkModeProvider")
  return ctx
}
