import React, { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { Input } from "@/components/ui/input";
import logo from "@/logo.svg";

type RecordType = "A" | "AAAA" | "CNAME" | "TXT" | "NS" | "PTR" | "MX";

type ZoneRecord = {
  name: string;
  record_type: RecordType;
  value: string;
  ttl: number;
};

type Zone = {
  id: number;
  name: string;
  records: ZoneRecord[];
};

async function apiRequest<T>(
  path: string,
  options: RequestInit = {},
  token?: string,
): Promise<T> {
  const headers = new Headers(options.headers);
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

function AdminPage() {
  const [zones, setZones] = useState<Zone[]>([]);
  const [query, setQuery] = useState("");

  useEffect(() => {
    void apiRequest<Zone[]>("/api/v1/dns/zones").then(setZones).catch(() => setZones([]));
  }, []);

  const filtered = zones.filter(z => z.name.toLowerCase().includes(query.toLowerCase()));

  return (
    <div className="min-h-screen grid grid-cols-[260px_1fr] bg-slate-950 text-slate-100">
      <aside className="border-r border-slate-800 bg-slate-900 px-2.5 py-3.5 flex flex-col gap-3">
        <div className="flex items-center gap-2.5 font-black text-base text-slate-100 px-2"><img src={logo} width={34} alt="HackFlare" /><span>HackFlare Admin</span></div>
        <nav className="grid gap-0.5">
          <Link to="/admin" className="rounded-lg px-3 py-2 text-sm font-medium bg-slate-800 text-slate-100 border-l-2 border-l-orange-500 pl-2.75 transition-colors">Overview</Link>
          <Link to="/dash/domains" className="rounded-lg px-3 py-2 text-sm font-medium text-slate-400 hover:text-slate-100 hover:bg-slate-800 transition-colors">Domain Search</Link>
          <Link to="/dash/settings" className="rounded-lg px-3 py-2 text-sm font-medium text-slate-400 hover:text-slate-100 hover:bg-slate-800 transition-colors">Runtime Settings</Link>
          <Link to="/dash" className="rounded-lg px-3 py-2 text-sm font-medium text-slate-400 hover:text-slate-100 hover:bg-slate-800 transition-colors">Back to Dashboard</Link>
        </nav>
      </aside>
      <main className="flex flex-col">
        <header className="border-b border-slate-800 px-7 py-3.5 bg-slate-950"><div><h1 className="text-2xl font-bold text-slate-100">Admin Panel</h1><p className="text-sm text-slate-400 mt-1">Manage platform settings and edit any domain.</p></div></header>
        <div className="flex-1 px-4.5 py-4.5">
          <section className="grid grid-cols-3 gap-4">
            <article className="border border-slate-700 rounded-2xl bg-slate-900 px-5.5 py-5.5"><h3 className="text-base font-semibold mb-4 text-slate-100">Total Domains</h3><p className="text-4xl font-black text-slate-100">{zones.length}</p></article>
            <article className="border border-slate-700 rounded-2xl bg-slate-900 px-5.5 py-5.5 col-span-2">
              <h3 className="text-base font-semibold mb-4 text-slate-100">Domain Search</h3>
              <Input value={query} onChange={e => setQuery(e.target.value)} placeholder="Search domains..." className="mb-4 bg-slate-800 border-slate-700 text-slate-100" />
              <div className="space-y-2">
                {filtered.map(zone => (
                  <Link key={zone.id} to="/dash/domains" className="flex justify-between items-center px-3 py-2 rounded-lg border border-slate-700 bg-slate-800 hover:bg-slate-700 transition-colors text-sm font-medium"><span>{zone.name}</span><small className="text-slate-500">{zone.records.length} records</small></Link>
                ))}
                {filtered.length === 0 && <p className="text-slate-500 text-sm italic text-center py-4">No domains found.</p>}
              </div>
            </article>
          </section>
        </div>
      </main>
    </div>
  );
}

export default AdminPage;
