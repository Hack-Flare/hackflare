export type RecordType = "A" | "AAAA" | "CNAME" | "TXT" | "NS" | "PTR" | "MX";

export type ZoneRecord = {
  name: string;
  record_type: RecordType;
  value: string;
  ttl: number;
};

export type Zone = {
  id: number;
  name: string;
  records: ZoneRecord[];
};

export type Session = {
  token: string;
  user: {
    id: string;
    email: string;
    created_at: string;
  };
};

export async function apiRequest<T>(
  path: string,
  options: RequestInit = {},
  token?: string,
): Promise<T> {
  const headers = new Headers(options.headers as HeadersInit);
  if (!(options.body instanceof FormData)) {
    headers.set("content-type", "application/json");
  }
  if (token) {
    headers.set("authorization", `Bearer ${token}`);
  }

  const res = await fetch(path, {
    ...options,
    headers,
  });

  if (!res.ok) {
    try {
      const json = (await res.json()) as { error?: string };
      throw new Error(json.error ?? `${res.status} ${res.statusText}`);
    } catch {
      throw new Error(`${res.status} ${res.statusText}`);
    }
  }

  if (res.status === 204) {
    return undefined as T;
  }

  return (await res.json()) as T;
}

export default apiRequest;
