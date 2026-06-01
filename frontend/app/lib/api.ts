const API_ORIGIN = import.meta.env.DEV
  ? ""
  : (import.meta.env.VITE_API_URL ?? "")

export interface AuthenticatedUser {
  id: string
  first_name: string
  last_name: string
  email: string
  slack_id: string
  eligible: boolean
}

export interface DnsZone {
  name: string
  ns_verified: boolean
}

export interface DnsRecord {
  id: string
  name: string
  type: string
  value: string
  ttl: number
  status: string
}

export interface UserSession {
  id: string
  user_id: string
  ip_address: string
  expires_at: string
  created_at: string
  revoked_at: string | null
}

export interface HealthResponse {
  status: string
  database: string
  dns_zones: number
}

export interface QueryLogEntry {
  id: number
  timestamp: string
  level: "info" | "warning" | "error"
  path: string
  status: number
  ms: number
}

export interface QueryLogsResponse {
  logs: QueryLogEntry[]
  summary: {
    errors_today: number
    warnings_today: number
    info_today: number
  }
}

export interface ConfigEntry {
  key: string
  label: string
  description: string
  env_value: string | null
  override_value: string | null
  effective_value: string
  editable: boolean
  requires_restart: boolean
  updated_at: string | null
  updated_by: string | null
}

export interface AdminUser {
  id: string
  email: string
  first_name: string
  last_name: string
  status: string
  created_at: string
}

export interface AdminStats {
  total_users: number
  total_zones: number
  total_sessions: number
}

export interface ApiKey {
  id: string
  name: string
  prefix: string
  created_at: string
  last_used_at: string | null
  revoked: boolean
}

export interface CreatedApiKey {
  key: ApiKey
  raw_key: string
}

interface ApiError {
  error: string
  status: number
}

let refreshing: Promise<void> | null = null

async function refreshTokens(): Promise<void> {
  const response = await fetch(`${API_ORIGIN}/api/v1/auth/refresh`, {
    method: "POST",
    credentials: "include",
  })
  if (!response.ok) {
    throw { error: "Refresh failed", status: response.status } as ApiError
  }
}

async function request<T = unknown>(
  endpoint: string,
  options: {
    method?: string
    body?: unknown
  } = {},
  _retried?: boolean,
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

  if (response.status === 401 && !endpoint.includes("/auth/refresh") && !_retried) {
    refreshing = refreshing ?? refreshTokens()
    try {
      await refreshing
      refreshing = null
      return await request<T>(endpoint, options, true)
    } catch {
      refreshing = null
      throw {
        error: "Session expired",
        status: 401,
      } as ApiError
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
    listZones: () => request<DnsZone[]>("/api/v1/dns/zones"),

    createZone: (name: string) =>
      request<DnsZone>("/api/v1/dns/zones", {
        method: "POST",
        body: { name },
      }),

    deleteZone: (zoneName: string) =>
      request<void>(`/api/v1/dns/zones/${encodeURIComponent(zoneName)}`, {
        method: "DELETE",
      }),

    verifyZone: (zoneName: string) =>
      request<{ verified: boolean; message?: string }>(
        `/api/v1/dns/zones/${encodeURIComponent(zoneName)}/verify`,
        { method: "POST" }
      ),

    listRecords: (zoneName: string) =>
      request<DnsRecord[]>(
        `/api/v1/dns/zones/${encodeURIComponent(zoneName)}/records`
      ),

    createRecord: (
      zoneName: string,
      data: { name: string; type: string; value: string; ttl: number }
    ) =>
      request<DnsRecord>(
        `/api/v1/dns/zones/${encodeURIComponent(zoneName)}/records`,
        { method: "POST", body: data }
      ),

    updateRecord: (
      zoneName: string,
      recordName: string,
      recordType: string,
      data: { value: string; ttl: number }
    ) =>
      request<DnsRecord>(
        `/api/v1/dns/zones/${encodeURIComponent(zoneName)}/records/${encodeURIComponent(recordName)}/${encodeURIComponent(recordType)}`,
        { method: "PUT", body: data }
      ),

    deleteRecord: (
      zoneName: string,
      recordName: string,
      recordType: string
    ) =>
      request<void>(
        `/api/v1/dns/zones/${encodeURIComponent(zoneName)}/records/${encodeURIComponent(recordName)}/${encodeURIComponent(recordType)}`,
        { method: "DELETE" }
      ),
  },

  health: {
    check: () => request<HealthResponse>("/api/v1/health"),
  },

  sessions: {
    list: () => request<UserSession[]>("/api/v1/sessions"),
  },

  settings: {
    listApiKeys: () => request<ApiKey[]>("/api/v1/settings/api-keys"),

    createApiKey: (name: string) =>
      request<CreatedApiKey>("/api/v1/settings/api-keys", {
        method: "POST",
        body: { name },
      }),

    revokeApiKey: (id: string) =>
      request<void>(`/api/v1/settings/api-keys/${id}`, {
        method: "DELETE",
      }),
  },

  admin: {
    listConfig: () => request<ConfigEntry[]>("/api/v1/admin/config"),

    upsertConfig: (key: string, value: string) =>
      request<void>(`/api/v1/admin/config/${encodeURIComponent(key)}`, {
        method: "PUT",
        body: { value },
      }),

    deleteConfig: (key: string) =>
      request<void>(`/api/v1/admin/config/${encodeURIComponent(key)}`, {
        method: "DELETE",
      }),

    listUsers: () => request<AdminUser[]>("/api/v1/admin/users"),

    getStats: () => request<AdminStats>("/api/v1/admin/stats"),
  },

  logs: {
    queryLogs: () => request<QueryLogsResponse>("/api/v1/logs/query-logs"),
  },

  slack: {
    contact: (text: string) =>
      request<void>("/api/v1/slack/contact", {
        method: "POST",
        body: { text },
      }),
  },
}
