const API_ORIGIN = import.meta.env.DEV
  ? ""
  : (import.meta.env.VITE_API_URL ?? "")

export interface AuthenticatedUser {
  id: string
  slack_id: string | null
  first_name: string
  last_name: string
  email: string
  eligible: boolean
  has_password: boolean
  is_admin: boolean
  created_at: string
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
  zone: string
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
  category: string
  env_value: string | null
  override_value: string | null
  effective_value: string
  default_value: string | null
  default_override: boolean
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

export interface Notification {
  id: string
  user_id: string
  title: string
  message: string
  type: string
  read: boolean
  link: string | null
  created_at: string
}

export interface UnreadCount {
  count: number
}

export interface TrafficSummary {
  total_requests: number
  avg_processing_ms: number
  success_rate: number
  error_rate: number
}

export interface TimeseriesPoint {
  date: string
  requests: number
  errors: number
  nxdomain: number
}

export interface ZoneTraffic {
  zone: string
  requests: number
  errors: number
  avg_ms: number
}

export interface TopQuery {
  query: string
  count: number
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

const ERROR_MESSAGES: Record<string, string> = {
  invalid_email_or_password: "Invalid email or password.",
  email_already_registered: "An account with this email already exists.",
  password_too_short: "Password must be at least 8 characters.",
  invalid_email: "Please enter a valid email address.",
  email_and_password_required: "Email and password are required.",
  invalid_or_expired_token: "This reset link is invalid or has expired.",
  email_required: "Please enter your email address.",
  token_required: "Reset token is required.",
  current_password_incorrect: "Your current password is incorrect.",
}

export function friendlyError(error: string): string {
  return ERROR_MESSAGES[error] || error
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

    register: (data: {
      email: string
      password: string
      first_name: string
      last_name: string
    }) =>
      request<void>("/api/v1/auth/register", {
        method: "POST",
        body: data,
      }),

    emailLogin: (data: { email: string; password: string }) =>
      request<void>("/api/v1/auth/login", {
        method: "POST",
        body: data,
      }),

    forgotPassword: (data: { email: string }) =>
      request<{ message: string }>("/api/v1/auth/forgot-password", {
        method: "POST",
        body: data,
      }),

    resetPassword: (data: { token: string; password: string }) =>
      request<{ message: string }>("/api/v1/auth/reset-password", {
        method: "POST",
        body: data,
      }),

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

    setPassword: (data: {
      current_password?: string
      new_password: string
    }) => request<{ message: string }>("/api/v1/settings/password", {
      method: "PUT",
      body: data,
    }),
  },

  admin: {
    listConfig: () => request<ConfigEntry[]>("/api/v1/admin/config"),
    applyConfig: () =>
      request<{ status: string }>("/api/v1/admin/config/apply", {
        method: "POST",
      }),

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
    trafficSummary: () => request<TrafficSummary>("/api/v1/admin/traffic/summary"),
    trafficTimeseries: (days = 30) => request<TimeseriesPoint[]>(`/api/v1/admin/traffic/timeseries?days=${days}`),
    trafficTopQueries: (limit = 10) => request<TopQuery[]>(`/api/v1/admin/traffic/top-queries?limit=${limit}`),
  },

  traffic: {
    summary: () => request<TrafficSummary>("/api/v1/traffic/summary"),
    timeseries: (days = 30) => request<TimeseriesPoint[]>(`/api/v1/traffic/timeseries?days=${days}`),
    byZone: () => request<ZoneTraffic[]>("/api/v1/traffic/by-zone"),
    topQueries: (limit = 5) => request<TopQuery[]>(`/api/v1/traffic/top-queries?limit=${limit}`),
  },

  notifications: {
    list: () => request<Notification[]>("/api/v1/notifications"),
    unreadCount: () => request<UnreadCount>("/api/v1/notifications/unread-count"),
    markRead: (id: string) =>
      request<void>(`/api/v1/notifications/${id}/read`, { method: "PUT" }),
    markAllRead: () =>
      request<void>("/api/v1/notifications/read-all", { method: "PUT" }),
  },

  logs: {
    queryLogs: () => request<QueryLogsResponse>("/api/v1/logs/query-logs"),
    queryLogsByZone: (zone: string) => request<QueryLogsResponse>(`/api/v1/logs/query-logs?zone=${encodeURIComponent(zone)}`),
  },

  slack: {
    contact: (text: string) =>
      request<void>("/api/v1/slack/contact", {
        method: "POST",
        body: { text },
      }),
  },
}
