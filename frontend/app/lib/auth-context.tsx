import React, { createContext, useContext, useEffect, useState } from "react"
import { api, type AuthenticatedUser } from "./api"

export interface User extends AuthenticatedUser {}

export function getUserDisplayName(user: User | null): string {
  if (!user) return "Signed in"
  const fullName = `${user.first_name} ${user.last_name}`.trim()
  if (fullName) return fullName
  if (user.email?.trim()) return user.email.trim()
  return `User ${user.id.slice(0, 8)}`
}

interface AuthContextType {
  user: User | null
  ready: boolean
  refreshUser: () => Promise<User | null>
  logout: () => Promise<void>
}

const AuthContext = createContext<AuthContextType | undefined>(undefined)

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<User | null>(null)
  const [ready, setReady] = useState(false)

  const refreshUser = async (): Promise<User | null> => {
    try {
      const currentUser = await api.auth.me()
      console.info("[Auth] session user loaded", {
        userId: currentUser.id,
        email: currentUser.email,
      })
      setUser(currentUser)
      return currentUser
    } catch (error) {
      console.error("[Auth] failed to load session user", error)
      setUser(null)
      return null
    } finally {
      setReady(true)
    }
  }

  useEffect(() => {
    void refreshUser()
  }, [])

  const logout = async () => {
    try {
      await api.auth.logout()
    } finally {
      setUser(null)
      setReady(true)
    }
  }

  return (
    <AuthContext.Provider value={{ user, ready, refreshUser, logout }}>
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
