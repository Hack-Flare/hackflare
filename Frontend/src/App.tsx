import React, { useEffect, useMemo, useState } from "react";
import { Link, Navigate, Route, Routes, useLocation, BrowserRouter } from "react-router-dom";
// Ignore missing type declarations for side-effect CSS import
// @ts-ignore
import "./global.css";
import logo from "./logo.svg";
import { Input } from "./components/ui/input";
import { Card, CardHeader, CardTitle, CardContent, CardFooter } from "./components/ui/card";

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

function HomePage() {
  return (
    <div className="flex flex-col min-h-screen bg-slate-50">
      <header className="sticky top-0 z-50 border-b border-slate-200/60 bg-white/72 backdrop-blur-md">
        <div className="max-w-4xl mx-auto px-6 h-17 flex items-center justify-between gap-4">
          <a href="/" className="flex items-center gap-2.5 font-black text-base">
            <img src={logo} width={36} alt="HackFlare" />
            <span>HackFlare</span>
          </a>

          <nav className="flex items-center gap-5.5 text-sm font-medium text-slate-700">
            <a href="#features" className="hover:text-slate-900">Features</a>
            <a href="#how-it-works" className="hover:text-slate-900">How it works</a>
            <a href="/docs" className="hover:text-slate-900">Docs</a>
          </nav>

          <div className="flex items-center gap-5.5">
            <a href="https://github.com/Hack-Flare/hackflare" className="text-sm font-medium text-slate-700 hover:text-slate-900">GitHub</a>
            <a href="/dash" className="text-sm font-medium text-slate-700 hover:text-slate-900">Sign in</a>
            <Link to="/dash" className="inline-flex items-center justify-center rounded-[10px] px-[1.15rem] py-[0.7rem] bg-orange-500 text-white font-semibold text-sm hover:bg-orange-600 transition-colors">Get Started</Link>
          </div>
        </div>
      </header>

      <main className="flex-1">
        <section className="max-w-4xl mx-auto px-6 py-24">
          <h1 className="max-w-205 text-4xl md:text-5xl lg:text-6xl font-bold leading-tight mb-0 bg-linear-to-b from-slate-900 to-slate-600 bg-clip-text text-transparent">The DNS platform for builders.</h1>
          <p className="max-w-160 mt-5.5 text-lg leading-relaxed text-slate-600">
            HackFlare helps you point your domain, manage DNS records, and ship faster.
            Built by Hack Clubbers.
          </p>
          <div className="mt-8 flex gap-3 flex-wrap">
            <Link to="/dash" className="inline-flex items-center justify-center rounded-[10px] px-[1.15rem] py-[0.7rem] bg-orange-500 text-white font-semibold text-sm hover:bg-orange-600 transition-all hover:-translate-y-0.5">Launch Dashboard</Link>
            <a href="/docs" className="inline-flex items-center justify-center rounded-[10px] px-[1.15rem] py-[0.7rem] border border-slate-300 bg-white text-slate-900 font-semibold text-sm hover:bg-slate-100 transition-colors">Read Docs</a>
          </div>
        </section>

        <section id="features" className="max-w-4xl mx-auto px-6 py-16">
          <h2 className="text-2xl md:text-3xl lg:text-4xl font-bold mb-8">Everything you need to manage DNS.</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <article className="border border-slate-200 rounded-2xl bg-white p-6 hover:border-slate-300 hover:-translate-y-0.5 transition-all"><h3 className="font-semibold mb-2">Edge Performance</h3><p className="text-slate-600 text-sm leading-normal">Built to feel super fast and direct.</p></article>
            <article className="border border-slate-200 rounded-2xl bg-white p-6 hover:border-slate-300 hover:-translate-y-0.5 transition-all"><h3 className="font-semibold mb-2">Good Security</h3><p className="text-slate-600 text-sm leading-normal">Great defaults, clear states and better UX.</p></article>
            <article className="border border-slate-200 rounded-2xl bg-white p-6 hover:border-slate-300 hover:-translate-y-0.5 transition-all"><h3 className="font-semibold mb-2">Automation Friendly</h3><p className="text-slate-600 text-sm leading-normal">REST and gRPC APIs for full automation.</p></article>
            <article className="border border-slate-200 rounded-2xl bg-white p-6 hover:border-slate-300 hover:-translate-y-0.5 transition-all"><h3 className="font-semibold mb-2">Made for Hack Club</h3><p className="text-slate-600 text-sm leading-normal">Built by the Hack Club community.</p></article>
            <article className="border border-slate-200 rounded-2xl bg-white p-6 hover:border-slate-300 hover:-translate-y-0.5 transition-all"><h3 className="font-semibold mb-2">Docs That Help</h3><p className="text-slate-600 text-sm leading-normal">Useful docs, guides, and AI integration.</p></article>
            <article className="border border-slate-200 rounded-2xl bg-white p-6 hover:border-slate-300 hover:-translate-y-0.5 transition-all"><h3 className="font-semibold mb-2">Global Reach</h3><p className="text-slate-600 text-sm leading-normal">A solid network that scales with projects.</p></article>
          </div>
        </section>

        <section id="how-it-works" className="max-w-4xl mx-auto px-6 py-16">
          <h2 className="text-2xl md:text-3xl lg:text-4xl font-bold mb-8">Get started in minutes.</h2>
          <ol className="space-y-2.5">
            <li className="relative pl-14 py-2.5 px-4 border border-slate-200 rounded-xl bg-white"><span className="absolute left-4 top-1/2 -translate-y-1/2 w-6 h-6 rounded-full bg-orange-100 text-orange-700 font-bold text-xs flex items-center justify-center">1</span><strong>Sign in.</strong> Open the dashboard and authenticate.</li>
            <li className="relative pl-14 py-2.5 px-4 border border-slate-200 rounded-xl bg-white"><span className="absolute left-4 top-1/2 -translate-y-1/2 w-6 h-6 rounded-full bg-orange-100 text-orange-700 font-bold text-xs flex items-center justify-center">2</span><strong>Add domains.</strong> Point nameservers to HackFlare.</li>
            <li className="relative pl-14 py-2.5 px-4 border border-slate-200 rounded-xl bg-white"><span className="absolute left-4 top-1/2 -translate-y-1/2 w-6 h-6 rounded-full bg-orange-100 text-orange-700 font-bold text-xs flex items-center justify-center">3</span><strong>Manage records.</strong> Add and edit DNS entries quickly.</li>
          </ol>
        </section>

        <section className="bg-linear-to-r from-red-600 via-orange-600 to-orange-500 py-16 mt-8">
          <div className="max-w-4xl mx-auto px-6">
            <h2 className="text-3xl md:text-4xl lg:text-5xl font-bold text-white max-w-2xl mb-3">Ready to take control of your DNS?</h2>
            <p className="text-lg text-white/92 max-w-xl mb-8">Join Hack Club members simplifying DNS infrastructure with HackFlare.</p>
            <div className="flex gap-3 flex-wrap">
              <Link to="/dash" className="inline-flex items-center justify-center rounded-[10px] px-[1.15rem] py-[0.7rem] bg-white text-orange-600 font-semibold text-sm hover:-translate-y-0.5 transition-all">Get Started Free</Link>
              <a href="/docs" className="inline-flex items-center justify-center rounded-[10px] px-[1.15rem] py-[0.7rem] border-2 border-white/85 text-white font-semibold text-sm hover:bg-white/10 transition-colors">Read Documentation</a>
            </div>
          </div>
        </section>
      </main>

      <footer className="border-t border-slate-200 bg-white">
        <div className="max-w-4xl mx-auto px-6 py-16">
          <div className="grid grid-cols-4 gap-8 mb-12">
            <div>
              <h4 className="font-semibold text-slate-900 mb-4">Product</h4>
              <ul className="space-y-2.5 text-sm text-slate-600">
                <li><a href="#features" className="hover:text-slate-900">Features</a></li>
                <li><a href="/docs" className="hover:text-slate-900">Documentation</a></li>
                <li><a href="#" className="hover:text-slate-900">Pricing</a></li>
                <li><a href="#" className="hover:text-slate-900">Status</a></li>
              </ul>
            </div>
            <div>
              <h4 className="font-semibold text-slate-900 mb-4">Company</h4>
              <ul className="space-y-2.5 text-sm text-slate-600">
                <li><a href="#" className="hover:text-slate-900">About</a></li>
                <li><a href="#" className="hover:text-slate-900">Blog</a></li>
                <li><a href="#" className="hover:text-slate-900">Careers</a></li>
                <li><a href="#" className="hover:text-slate-900">Contact</a></li>
              </ul>
            </div>
            <div>
              <h4 className="font-semibold text-slate-900 mb-4">Resources</h4>
              <ul className="space-y-2.5 text-sm text-slate-600">
                <li><a href="#" className="hover:text-slate-900">API Docs</a></li>
                <li><a href="#" className="hover:text-slate-900">Guides</a></li>
                <li><a href="#" className="hover:text-slate-900">Community</a></li>
                <li><a href="#" className="hover:text-slate-900">Support</a></li>
              </ul>
            </div>
            <div>
              <h4 className="font-semibold text-slate-900 mb-4">Legal</h4>
              <ul className="space-y-2.5 text-sm text-slate-600">
                <li><a href="#" className="hover:text-slate-900">Privacy</a></li>
                <li><a href="#" className="hover:text-slate-900">Terms</a></li>
                <li><a href="#" className="hover:text-slate-900">Security</a></li>
                <li><a href="#" className="hover:text-slate-900">License</a></li>
              </ul>
            </div>
          </div>
          <div className="border-t border-slate-200 pt-8 flex items-center justify-between">
            <div className="flex items-center gap-2 font-black text-base text-slate-900">
              <img src={logo} width={24} alt="HackFlare" />
              <span>HackFlare</span>
            </div>
            <p className="text-sm text-slate-600">© 2026 HackFlare. All rights reserved.</p>
            <div className="flex gap-4">
              <a href="https://github.com/Hack-Flare/hackflare" className="text-slate-600 hover:text-slate-900">Github</a>
              <a href="https://kirze.de" className="text-slate-600 hover:text-slate-900">@Nayte</a>
              <a href="https://vejas.zip" className="text-slate-600 hover:text-slate-900">@Vejas</a>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
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

function AppRoutes() {
  return (
    <Routes>
      <Route path="/" element={<HomePage />} />
      <Route path="/dash" element={<DashboardShell />} />
      <Route path="/dash/domains" element={<DashboardShell />} />
      <Route path="/dash/settings" element={<DashboardShell />} />
      <Route path="/dash/analytics" element={<DashboardShell />} />
      <Route path="/dash/notifications" element={<DashboardShell />} />
      <Route path="/dash/help" element={<DashboardShell />} />
      <Route path="/admin" element={<AdminPage />} />
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}

export function App() {
  return (
    <BrowserRouter>
      <AppRoutes />
    </BrowserRouter>
  );
}

export default App;
