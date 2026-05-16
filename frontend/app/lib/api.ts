const API_ORIGIN = import.meta.env.DEV
  ? ""
  : import.meta.env.VITE_API_URL || "http://localhost:8080"

export interface AuthenticatedUser {
  id: string
  first_name: string
  last_name: string
  email: string
  slack_id: string
  eligible: boolean
}

interface ApiError {
  error: string
  status: number
}

async function request<T = unknown>(
  endpoint: string,
  options: {
    method?: string
    body?: unknown
  } = {}
): Promise<T> {
  const url = `${API_ORIGIN}${endpoint}`
  const headers: Record<string, string> = {}

  if (options.body) {
    headers["Content-Type"] = "application/json"
  }

  console.log(`[API] ${options.method || "GET"} ${endpoint}`)

  const response = await fetch(url, {
    method: options.method || "GET",
    headers,
    body: options.body ? JSON.stringify(options.body) : undefined,
    credentials: "include",
  })

  const text = await response.text()
  let data: unknown = null

  if (text) {
    try {
      data = JSON.parse(text)
    } catch {
      data = text
    }
  }

  if (!response.ok) {
    const errorMessage =
      typeof data === "string"
        ? data
        : data && typeof data === "object" && "error" in data
          ? String((data as { error?: unknown }).error || "Unknown error")
          : response.statusText || "Unknown error"

    console.error(`[API] Error ${response.status}:`, errorMessage)
    throw {
      error: errorMessage,
      status: response.status,
    } as ApiError
  }

  console.log(`[API] ✓ ${response.status}`)
  return data as T
}

export const api = {
  auth: {
    loginUrl: (target: string) =>
      `${API_ORIGIN}/api/v1/auth/login?target=${encodeURIComponent(target)}`,

    me: () => request<AuthenticatedUser>("/api/v1/users/me"),

    logout: async () => {
      const response = await fetch(`${API_ORIGIN}/api/v1/auth/logout`, {
        method: "POST",
        credentials: "include",
      })

      if (!response.ok) {
        throw {
          error: "Logout failed",
          status: response.status,
        } as ApiError
      }
    },
  },

  dns: {
    listZones: () => request("/api/v1/dns/zones"),

    createZone: (name: string) =>
      request("/api/v1/dns/zones", {
        method: "POST",
        body: { name },
      }),

    verifyZone: (zoneName: string) =>
      request(`/api/v1/dns/zones/${encodeURIComponent(zoneName)}/verify`, {
        method: "POST",
      }),
  },
}
