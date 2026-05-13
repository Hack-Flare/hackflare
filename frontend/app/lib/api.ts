const BASE_URL = import.meta.env.VITE_API_URL || "http://localhost:8080"
const INTERNAL_TOKEN = import.meta.env.VITE_INTERNAL_TOKEN || ""

interface ApiError {
  error: string
  status: number
}

async function request(
  endpoint: string,
  options: {
    method?: string
    body?: unknown
    token?: string
  } = {}
) {
  const url = `${BASE_URL}${endpoint}`
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    "x-internal-token": INTERNAL_TOKEN,
  }

  if (options.token) {
    headers["Authorization"] = `Bearer ${options.token}`
  }

  console.log(`[API] ${options.method || "GET"} ${endpoint}`)

  const response = await fetch(url, {
    method: options.method || "GET",
    headers,
    body: options.body ? JSON.stringify(options.body) : undefined,
  })

  const data = await response.json()

  if (!response.ok) {
    console.error(`[API] Error ${response.status}:`, data.error)
    throw {
      error: data.error || "Unknown error",
      status: response.status,
    } as ApiError
  }

  console.log(`[API] ✓ ${response.status}`)
  return data
}

export const api = {
  auth: {
    login: (input: { email: string; password: string }) =>
      request("/api/v1/auth/login", { method: "POST", body: input }),

    hackclubUrl: () =>
      request("/api/v1/auth/hackclub/url"),

    hackclubCallback: (code: string) =>
      request(`/api/v1/auth/hackclub/callback?code=${encodeURIComponent(code)}`),

    me: (token: string) =>
      request("/api/v1/auth/me", { token }),
  },

  dns: {
    listZones: (token: string) =>
      request("/api/v1/dns/zones", { token }),

    createZone: (token: string, name: string) =>
      request("/api/v1/dns/zones", {
        method: "POST",
        token,
        body: { name },
      }),

    verifyZone: (token: string, zoneName: string) =>
      request(`/api/v1/dns/zones/${encodeURIComponent(zoneName)}/verify`, {
        method: "POST",
        token,
      }),
  },
}
