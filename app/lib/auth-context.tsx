import React, { createContext, useContext, useEffect, useState } from "react"

export interface User {
  id: number
  email: string
  name: string
  is_admin: boolean
}

interface AuthContextType {
  user: User | null
  token: string | null
  login: (token: string, user: User) => void
  logout: () => void
}

const AuthContext = createContext<AuthContextType | undefined>(undefined)

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<User | null>(null)
  const [token, setToken] = useState<string | null>(null)

  useEffect(() => {
    const storedToken = localStorage.getItem("hf_token")
    const storedUser = localStorage.getItem("hf_user")

    if (storedToken && storedUser) {
      setToken(storedToken)
      setUser(JSON.parse(storedUser))
    }
  }, [])

  const login = (newToken: string, newUser: User) => {
    console.log("[Auth] Login:", newUser.email)
    setToken(newToken)
    setUser(newUser)
    localStorage.setItem("hf_token", newToken)
    localStorage.setItem("hf_user", JSON.stringify(newUser))
  }

  const logout = () => {
    console.log("[Auth] Logout")
    setToken(null)
    setUser(null)
    localStorage.removeItem("hf_token")
    localStorage.removeItem("hf_user")
  }

  return (
    <AuthContext.Provider value={{ user, token, login, logout }}>
      {children}
    </AuthContext.Provider>
  )
}

export function useAuth() {
  const context = useContext(AuthContext)
  if (!context) {
    throw new Error("useAuth must be used within AuthProvider")
  }
  return context
}
