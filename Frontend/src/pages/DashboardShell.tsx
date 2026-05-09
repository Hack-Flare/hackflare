import React, { useEffect, useMemo, useState } from "react";
import { Link, useLocation } from "react-router-dom";
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

type Session = {
  token: string;
  user: {
    id: string;
    email: string;
    created_at: string;
  };
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

function DashboardShell() {
  const location = useLocation();
  const [zones, setZones] = useState<Zone[]>([]);
  const [selectedZone, setSelectedZone] = useState<string>("");
  const [status, setStatus] = useState<string>("Ready");
  const [session, setSession] = useState<Session | null>(null);

  const [registerEmail, setRegisterEmail] = useState("");
  const [registerPassword, setRegisterPassword] = useState("");
  const [loginEmail, setLoginEmail] = useState("");
  const [loginPassword, setLoginPassword] = useState("");

  const [newZoneName, setNewZoneName] = useState("");
  const [recordName, setRecordName] = useState("@");
  const [recordType, setRecordType] = useState<RecordType>("A");
  const [recordValue, setRecordValue] = useState("");
  const [recordTtl, setRecordTtl] = useState(300);

  const currentView = useMemo(() => {
    if (location.pathname === "/dash") return "home";
    if (location.pathname.startsWith("/dash/domains")) return "domains";
    if (location.pathname.startsWith("/dash/settings")) return "settings";
    if (location.pathname.startsWith("/dash/analytics")) return "analytics";
    if (location.pathname.startsWith("/dash/notifications")) return "notifications";
    if (location.pathname.startsWith("/dash/help")) return "help";
    return "home";
  }, [location.pathname]);

  const selectedZoneData = zones.find(z => z.name === selectedZone) ?? zones[0];

  const refreshZones = async () => {
    const data = await apiRequest<Zone[]>("/api/v1/dns/zones");
    setZones(data);
    const firstZone = data.at(0);
    if (firstZone && !selectedZone) {
      setSelectedZone(firstZone.name);
    }
  };

  useEffect(() => {
    void refreshZones();
  }, []);

  const handleRegister = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    try {
      setStatus("Registering...");
      const result = await apiRequest<Session>("/api/v1/auth/register", {
        method: "POST",
        body: JSON.stringify({ email: registerEmail, password: registerPassword }),
      });
      setSession(result);
      setStatus("Registered");
    } catch (err) {
      setStatus(`Register failed: ${String((err as Error).message)}`);
    }
  };

  const handleLogin = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    try {
      setStatus("Logging in...");
      const result = await apiRequest<Session>("/api/v1/auth/login", {
        method: "POST",
        body: JSON.stringify({ email: loginEmail, password: loginPassword }),
      });
      setSession(result);
      setStatus("Logged in");
    } catch (err) {
      setStatus(`Login failed: ${String((err as Error).message)}`);
    }
  };

  const handleCreateZone = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    try {
      setStatus("Creating zone...");
      await apiRequest<Zone>("/api/v1/dns/zones", {
        method: "POST",
        body: JSON.stringify({ name: newZoneName }),
      });
      setNewZoneName("");
      await refreshZones();
      setStatus("Zone created");
    } catch (err) {
      setStatus(`Create zone failed: ${String((err as Error).message)}`);
    }
  };

  const handleCreateRecord = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!selectedZoneData) {
      setStatus("No selected zone");
      return;
    }
    try {
      setStatus("Adding record...");
      await apiRequest<Zone>(`/api/v1/dns/zones/${encodeURIComponent(selectedZoneData.name)}/records`, {
        method: "POST",
        body: JSON.stringify({
          name: recordName,
          record_type: recordType,
          value: recordValue,
          ttl: Number(recordTtl),
        }),
      });
      await refreshZones();
      setStatus("Record added");
    } catch (err) {
      setStatus(`Add record failed: ${String((err as Error).message)}`);
    }
  };

  const title = {
    home: "Dashboard",
    domains: "Domains",
    settings: "Settings",
    analytics: "Analytics",
    notifications: "Notifications",
    help: "Help",
  }[currentView];

  const subtitle = {
    home: "Welcome to HackFlare",
    domains: "Manage DNS zones and records",
    settings: "Configure account and platform defaults",
    analytics: "Monitor performance and traffic",
    notifications: "Control alerts and updates",
    help: "Find docs and support resources",
  }[currentView];

  return (
    <div className="min-h-screen grid grid-cols-[260px_1fr] bg-slate-950 text-slate-100">
      <aside className="border-r border-slate-800 bg-slate-900 px-2.5 py-3.5 flex flex-col gap-3">
        <div className="flex items-center gap-2.5 font-black text-base text-slate-100 px-2">
          <img src={logo} width={34} alt="HackFlare" />
          <span>HackFlare</span>
        </div>

        <nav className="grid gap-0.5">
          <Link className={`rounded-lg px-3 py-2 text-sm font-medium transition-colors ${currentView === "home" ? "bg-slate-800 text-slate-100 border-l-2 border-l-orange-500 pl-2.75" : "text-slate-400 hover:text-slate-100 hover:bg-slate-800"}`} to="/dash">Home</Link>
          <Link className={`rounded-lg px-3 py-2 text-sm font-medium transition-colors ${currentView === "domains" ? "bg-slate-800 text-slate-100 border-l-2 border-l-orange-500 pl-2.75" : "text-slate-400 hover:text-slate-100 hover:bg-slate-800"}`} to="/dash/domains">Domains</Link>
          <Link className={`rounded-lg px-3 py-2 text-sm font-medium transition-colors ${currentView === "settings" ? "bg-slate-800 text-slate-100 border-l-2 border-l-orange-500 pl-2.75" : "text-slate-400 hover:text-slate-100 hover:bg-slate-800"}`} to="/dash/settings">Settings</Link>
          <Link className={`rounded-lg px-3 py-2 text-sm font-medium transition-colors ${currentView === "analytics" ? "bg-slate-800 text-slate-100 border-l-2 border-l-orange-500 pl-2.75" : "text-slate-400 hover:text-slate-100 hover:bg-slate-800"}`} to="/dash/analytics">Analytics</Link>
          <Link className={`rounded-lg px-3 py-2 text-sm font-medium transition-colors ${currentView === "notifications" ? "bg-slate-800 text-slate-100 border-l-2 border-l-orange-500 pl-2.75" : "text-slate-400 hover:text-slate-100 hover:bg-slate-800"}`} to="/dash/notifications">Notifications</Link>
          <Link className={`rounded-lg px-3 py-2 text-sm font-medium transition-colors ${currentView === "help" ? "bg-slate-800 text-slate-100 border-l-2 border-l-orange-500 pl-2.75" : "text-slate-400 hover:text-slate-100 hover:bg-slate-800"}`} to="/dash/help">Help</Link>
          <Link to="/admin" className="rounded-lg px-3 py-2 text-sm font-medium text-slate-400 hover:text-slate-100 hover:bg-slate-800 transition-colors">Admin Panel</Link>
        </nav>
      </aside>

      <main className="flex flex-col">
        <header className="flex justify-between items-center border-b border-slate-800 px-7 py-3.5 bg-slate-950">
          <div>
            <h1 className="text-2xl font-bold text-slate-100">{title}</h1>
            <p className="text-sm text-slate-400 mt-1">{subtitle}</p>
          </div>
          <div className="inline-flex items-center gap-2 border border-slate-700 bg-slate-900 rounded-full px-3 py-1.5 text-sm font-medium text-slate-300"><span className="w-1.5 h-1.5 bg-green-500 rounded-full"></span>{status}</div>
        </header>

        <div className="flex-1 px-4.5 py-4.5">
          {currentView === "home" && (
            <section className="grid grid-cols-3 gap-4">
              <article className="border border-slate-700 rounded-2xl bg-slate-900 px-5.5 py-5.5 hover:border-slate-600"><h3 className="text-base font-semibold mb-2 text-slate-100">Quick Start</h3><p className="text-sm text-slate-400">Set up your first domain and records.</p></article>
              <article className="border border-slate-700 rounded-2xl bg-slate-900 px-5.5 py-5.5 hover:border-slate-600"><h3 className="text-base font-semibold mb-2 text-slate-100">Your Domains</h3><p className="text-sm text-slate-400">Manage all DNS zones in one place.</p></article>
              <article className="border border-slate-700 rounded-2xl bg-slate-900 px-5.5 py-5.5 hover:border-slate-600"><h3 className="text-base font-semibold mb-2 text-slate-100">Analytics</h3><p className="text-sm text-slate-400">Traffic and DNS health metrics soon.</p></article>
            </section>
          )}

          {currentView === "domains" && (
            <section className="grid grid-cols-[280px_1fr] gap-4">
              <aside className="border border-slate-700 rounded-2xl bg-slate-900 overflow-hidden h-fit">
                <div className="flex items-center justify-between border-b border-slate-700 px-4 py-3">
                  <h3 className="text-sm font-semibold text-slate-100">Domains</h3>
                  <span className="text-xs font-semibold px-2 py-1 rounded-full bg-slate-800 text-slate-300">{zones.length}</span>
                </div>
                {zones.length === 0 && <p className="text-slate-500 text-sm italic p-4 text-center">No domains yet</p>}
                {zones.map(zone => (
                  <button
                    key={zone.id}
                    onClick={() => setSelectedZone(zone.name)}
                    className={`w-full text-left px-4 py-3 border-b border-slate-700 font-medium text-sm transition-colors hover:bg-slate-800 ${zone.name === (selectedZoneData?.name ?? "") ? "bg-slate-800 text-slate-100 border-l-2 border-l-orange-500 pl-3.75" : "text-slate-400"}`}
                  >
                    <strong className="block text-slate-100">{zone.name}</strong>
                    <small className="text-slate-500 text-xs">{zone.records.length} records</small>
                  </button>
                ))}
              </aside>

              <div className="grid gap-4">
                <div className="border border-slate-700 rounded-2xl bg-slate-900 px-5.5 py-5.5">
                  <form onSubmit={handleCreateZone} className="grid grid-cols-[1fr_auto] gap-2.5 mb-6">
                    <Input value={newZoneName} onChange={e => setNewZoneName(e.target.value)} placeholder="example.com" required className="bg-slate-800 border-slate-700 text-slate-100" />
                    <button type="submit" className="px-4 py-2 rounded-lg bg-orange-500 text-white font-semibold text-sm hover:bg-orange-600 transition-colors">+ Add Domain</button>
                  </form>

                  <form onSubmit={handleCreateRecord} className="grid grid-cols-2 gap-2.5">
                    <h4 className="col-span-2 text-xs font-semibold uppercase text-slate-400 tracking-wide mb-1">Add Record</h4>
                    <Input value={recordName} onChange={e => setRecordName(e.target.value)} placeholder="name" required className="bg-slate-800 border-slate-700 text-slate-100" />
                    <select value={recordType} onChange={e => setRecordType(e.target.value as RecordType)} className="rounded-lg border border-slate-700 bg-slate-800 text-slate-100 px-3 py-2 text-sm">
                      <option>A</option>
                      <option>AAAA</option>
                      <option>CNAME</option>
                      <option>TXT</option>
                      <option>NS</option>
                      <option>PTR</option>
                      <option>MX</option>
                    </select>
                    <Input value={recordValue} onChange={e => setRecordValue(e.target.value)} placeholder="value" required className="col-span-2 bg-slate-800 border-slate-700 text-slate-100" />
                    <Input type="number" min={1} value={recordTtl} onChange={e => setRecordTtl(Number(e.target.value))} required className="col-span-2 bg-slate-800 border-slate-700 text-slate-100" />
                    <button type="submit" className="col-span-2 px-4 py-2 rounded-lg bg-orange-500 text-white font-semibold text-sm hover:bg-orange-600 transition-colors">Save Record</button>
                  </form>
                </div>

                <div className="border border-slate-700 rounded-2xl bg-slate-900 px-5.5 py-5.5 overflow-hidden">
                  <h3 className="text-base font-semibold mb-3 text-slate-100">{selectedZoneData?.name ?? "No zone selected"}</h3>
                  {selectedZoneData && selectedZoneData.records.length > 0 ? (
                    <table className="w-full text-sm">
                      <thead>
                        <tr className="border-b border-slate-700 text-xs font-semibold uppercase text-slate-400 tracking-wide">
                          <th className="text-left px-3 py-2">Type</th>
                          <th className="text-left px-3 py-2">Name</th>
                          <th className="text-left px-3 py-2">Content</th>
                          <th className="text-left px-3 py-2">TTL</th>
                        </tr>
                      </thead>
                      <tbody>
                        {selectedZoneData.records.map((record, i) => (
                          <tr key={`${record.name}-${record.record_type}-${i}`} className="border-b border-slate-700 hover:bg-slate-800 transition-colors text-slate-300">
                            <td className="px-3 py-2.5 font-semibold text-orange-400">{record.record_type}</td>
                            <td className="px-3 py-2.5">{record.name}</td>
                            <td className="px-3 py-2.5 font-mono text-xs">{record.value}</td>
                            <td className="px-3 py-2.5">{record.ttl}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  ) : (
                    <p className="text-slate-500 text-sm italic p-4 text-center">No records in this domain yet.</p>
                  )}
                </div>
              </div>
            </section>
          )}

          {currentView === "settings" && (
            <section className="grid gap-4">
              <article className="border border-slate-700 rounded-2xl bg-slate-900 px-5.5 py-5.5 col-span-2">
                <h3 className="text-base font-semibold mb-4 text-slate-100">Auth Session</h3>
                <div className="grid grid-cols-2 gap-6 mb-6">
                  <form onSubmit={handleRegister} className="grid gap-2.5">
                    <h4 className="text-xs font-semibold uppercase text-slate-400 tracking-wide">Register</h4>
                    <Input type="email" value={registerEmail} onChange={e => setRegisterEmail(e.target.value)} placeholder="email" required className="bg-slate-800 border-slate-700 text-slate-100" />
                    <Input type="password" value={registerPassword} onChange={e => setRegisterPassword(e.target.value)} placeholder="password" required className="bg-slate-800 border-slate-700 text-slate-100" />
                    <button type="submit" className="px-4 py-2 rounded-lg bg-orange-500 text-white font-semibold text-sm hover:bg-orange-600 transition-colors">Register</button>
                  </form>
                  <form onSubmit={handleLogin} className="grid gap-2.5">
                    <h4 className="text-xs font-semibold uppercase text-slate-400 tracking-wide">Login</h4>
                    <Input type="email" value={loginEmail} onChange={e => setLoginEmail(e.target.value)} placeholder="email" required className="bg-slate-800 border-slate-700 text-slate-100" />
                    <Input type="password" value={loginPassword} onChange={e => setLoginPassword(e.target.value)} placeholder="password" required className="bg-slate-800 border-slate-700 text-slate-100" />
                    <button type="submit" className="px-4 py-2 rounded-lg bg-orange-500 text-white font-semibold text-sm hover:bg-orange-600 transition-colors">Login</button>
                  </form>
                </div>
                <pre className="mt-4 max-h-60 overflow-auto rounded-lg border border-slate-700 bg-slate-950 text-slate-300 p-3.5 text-xs font-mono">{JSON.stringify(session, null, 2)}</pre>
              </article>
            </section>
          )}

          {currentView === "analytics" && (
            <section className="grid gap-4"><article className="border border-slate-700 rounded-2xl bg-slate-900 px-5.5 py-5.5 col-span-2"><h3 className="text-base font-semibold mb-3 text-slate-100">Analytics</h3><p className="text-sm text-slate-400">Analytics view ported. Data integration pending backend metrics endpoint.</p></article></section>
          )}

          {currentView === "notifications" && (
            <section className="grid gap-4"><article className="border border-slate-700 rounded-2xl bg-slate-900 px-5.5 py-5.5 col-span-2"><h3 className="text-base font-semibold mb-3 text-slate-100">Notifications</h3><p className="text-sm text-slate-400">Notification center ported. Alert APIs can be wired next.</p></article></section>
          )}

          {currentView === "help" && (
            <section className="grid gap-4"><article className="border border-slate-700 rounded-2xl bg-slate-900 px-5.5 py-5.5 col-span-2"><h3 className="text-base font-semibold mb-3 text-slate-100">Help</h3><p className="text-sm text-slate-400">Support view ported from Elixir frontend layout.</p></article></section>
          )}
        </div>
      </main>
    </div>
  );
}



export default DashboardShell;
