import { FormEvent, useEffect, useMemo, useState } from "react";
import { Link, Navigate, Route, Routes, useLocation, BrowserRouter } from "react-router-dom";
import "./index.css";
import logo from "./logo.svg";

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
    <div className="home-page">
      <header className="topbar">
        <div className="wrap topbar-inner">
          <a href="/" className="brand">
            <img src={logo} width={36} alt="HackFlare" />
            <span>HackFlare</span>
          </a>

          <nav className="home-nav">
            <a href="#features">Features</a>
            <a href="#how-it-works">How it works</a>
            <a href="/docs">Docs</a>
          </nav>

          <div className="home-actions">
            <a href="https://github.com/Hack-Flare/hackflare">GitHub</a>
            <a href="/dash">Sign in</a>
            <Link to="/dash" className="btn-primary">Get Started</Link>
          </div>
        </div>
      </header>

      <main>
        <section className="hero wrap">
          <h1>The DNS platform for builders.</h1>
          <p>
            HackFlare helps you point your domain, manage DNS records, and ship faster.
            Built by Hack Clubbers.
          </p>
          <div className="hero-cta">
            <Link to="/dash" className="btn-primary">Launch Dashboard</Link>
            <a href="/docs" className="btn-ghost">Read Docs</a>
          </div>
        </section>

        <section id="features" className="features wrap">
          <h2>Everything you need to manage DNS.</h2>
          <div className="feature-grid">
            <article><h3>Edge Performance</h3><p>Built to feel super fast and direct.</p></article>
            <article><h3>Good Security</h3><p>Great defaults, clear states and better UX.</p></article>
            <article><h3>Automation Friendly</h3><p>REST and gRPC APIs for full automation.</p></article>
            <article><h3>Made for Hack Club</h3><p>Built by the Hack Club community.</p></article>
            <article><h3>Docs That Help</h3><p>Useful docs, guides, and AI integration.</p></article>
            <article><h3>Global Reach</h3><p>A solid network that scales with projects.</p></article>
          </div>
        </section>

        <section id="how-it-works" className="steps wrap">
          <h2>Get started in minutes.</h2>
          <ol>
            <li><strong>Sign in.</strong> Open the dashboard and authenticate.</li>
            <li><strong>Add domains.</strong> Point nameservers to HackFlare.</li>
            <li><strong>Manage records.</strong> Add and edit DNS entries quickly.</li>
          </ol>
        </section>

        <section className="cta-banner">
          <div className="wrap">
            <h2>Ready to take control of your DNS?</h2>
            <p>Join Hack Club members simplifying DNS infrastructure with HackFlare.</p>
            <div className="hero-cta">
              <Link to="/dash" className="btn-white">Get Started Free</Link>
              <a href="/docs" className="btn-outline-white">Read Documentation</a>
            </div>
          </div>
        </section>
      </main>
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
    if (data.length > 0 && !selectedZone) {
      setSelectedZone(data[0].name);
    }
  };

  useEffect(() => {
    void refreshZones();
  }, []);

  const handleRegister = async (e: FormEvent) => {
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

  const handleLogin = async (e: FormEvent) => {
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

  const handleCreateZone = async (e: FormEvent) => {
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

  const handleCreateRecord = async (e: FormEvent) => {
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
    <div className="dash-shell">
      <aside className="dash-sidebar">
        <div className="dash-brand">
          <img src={logo} width={34} alt="HackFlare" />
          <span>HackFlare</span>
        </div>

        <nav>
          <Link className={currentView === "home" ? "active" : ""} to="/dash">Home</Link>
          <Link className={currentView === "domains" ? "active" : ""} to="/dash/domains">Domains</Link>
          <Link className={currentView === "settings" ? "active" : ""} to="/dash/settings">Settings</Link>
          <Link className={currentView === "analytics" ? "active" : ""} to="/dash/analytics">Analytics</Link>
          <Link className={currentView === "notifications" ? "active" : ""} to="/dash/notifications">Notifications</Link>
          <Link className={currentView === "help" ? "active" : ""} to="/dash/help">Help</Link>
          <Link to="/admin">Admin Panel</Link>
        </nav>
      </aside>

      <main className="dash-main">
        <header className="dash-header">
          <div>
            <h1>{title}</h1>
            <p>{subtitle}</p>
          </div>
          <div className="status-pill">{status}</div>
        </header>

        <div className="dash-content">
          {currentView === "home" && (
            <section className="dash-grid">
              <article className="dash-card">
                <h3>Quick Start</h3>
                <p>Set up your first domain and records.</p>
              </article>
              <article className="dash-card">
                <h3>Your Domains</h3>
                <p>Manage all DNS zones in one place.</p>
              </article>
              <article className="dash-card">
                <h3>Analytics</h3>
                <p>Traffic and DNS health metrics soon.</p>
              </article>
            </section>
          )}

          {currentView === "domains" && (
            <section className="domains-layout">
              <aside className="zone-list">
                <div className="zone-list-head">
                  <h3>Domains</h3>
                  <span>{zones.length}</span>
                </div>
                {zones.length === 0 && <p className="empty">No domains yet</p>}
                {zones.map(zone => (
                  <button
                    key={zone.id}
                    className={zone.name === (selectedZoneData?.name ?? "") ? "selected" : ""}
                    onClick={() => setSelectedZone(zone.name)}
                  >
                    <strong>{zone.name}</strong>
                    <small>{zone.records.length} records</small>
                  </button>
                ))}
              </aside>

              <div className="zone-main">
                <div className="zone-actions card-dark">
                  <form onSubmit={handleCreateZone} className="inline-form">
                    <input
                      value={newZoneName}
                      onChange={e => setNewZoneName(e.target.value)}
                      placeholder="example.com"
                      required
                    />
                    <button type="submit">+ Add Domain</button>
                  </form>

                  <form onSubmit={handleCreateRecord} className="grid-form">
                    <h4>Add Record</h4>
                    <input value={recordName} onChange={e => setRecordName(e.target.value)} placeholder="name" required />
                    <select value={recordType} onChange={e => setRecordType(e.target.value as RecordType)}>
                      <option>A</option>
                      <option>AAAA</option>
                      <option>CNAME</option>
                      <option>TXT</option>
                      <option>NS</option>
                      <option>PTR</option>
                      <option>MX</option>
                    </select>
                    <input value={recordValue} onChange={e => setRecordValue(e.target.value)} placeholder="value" required />
                    <input type="number" min={1} value={recordTtl} onChange={e => setRecordTtl(Number(e.target.value))} required />
                    <button type="submit">Save Record</button>
                  </form>
                </div>

                <div className="records card-dark">
                  <h3>{selectedZoneData?.name ?? "No zone selected"}</h3>
                  {selectedZoneData && selectedZoneData.records.length > 0 ? (
                    <table>
                      <thead>
                        <tr>
                          <th>Type</th>
                          <th>Name</th>
                          <th>Content</th>
                          <th>TTL</th>
                        </tr>
                      </thead>
                      <tbody>
                        {selectedZoneData.records.map((record, i) => (
                          <tr key={`${record.name}-${record.record_type}-${i}`}>
                            <td>{record.record_type}</td>
                            <td>{record.name}</td>
                            <td>{record.value}</td>
                            <td>{record.ttl}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  ) : (
                    <p className="empty">No records in this domain yet.</p>
                  )}
                </div>
              </div>
            </section>
          )}

          {currentView === "settings" && (
            <section className="dash-grid">
              <article className="dash-card span-2">
                <h3>Auth Session</h3>
                <div className="two-col">
                  <form onSubmit={handleRegister} className="stack-form">
                    <h4>Register</h4>
                    <input type="email" value={registerEmail} onChange={e => setRegisterEmail(e.target.value)} placeholder="email" required />
                    <input type="password" value={registerPassword} onChange={e => setRegisterPassword(e.target.value)} placeholder="password" required />
                    <button type="submit">Register</button>
                  </form>
                  <form onSubmit={handleLogin} className="stack-form">
                    <h4>Login</h4>
                    <input type="email" value={loginEmail} onChange={e => setLoginEmail(e.target.value)} placeholder="email" required />
                    <input type="password" value={loginPassword} onChange={e => setLoginPassword(e.target.value)} placeholder="password" required />
                    <button type="submit">Login</button>
                  </form>
                </div>
                <pre>{JSON.stringify(session, null, 2)}</pre>
              </article>
            </section>
          )}

          {currentView === "analytics" && (
            <section className="dash-grid"><article className="dash-card span-2"><h3>Analytics</h3><p>Analytics view ported. Data integration pending backend metrics endpoint.</p></article></section>
          )}

          {currentView === "notifications" && (
            <section className="dash-grid"><article className="dash-card span-2"><h3>Notifications</h3><p>Notification center ported. Alert APIs can be wired next.</p></article></section>
          )}

          {currentView === "help" && (
            <section className="dash-grid"><article className="dash-card span-2"><h3>Help</h3><p>Support view ported from Elixir frontend layout.</p></article></section>
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
    <div className="dash-shell">
      <aside className="dash-sidebar">
        <div className="dash-brand"><img src={logo} width={34} alt="HackFlare" /><span>HackFlare Admin</span></div>
        <nav>
          <Link className="active" to="/admin">Overview</Link>
          <Link to="/dash/domains">Domain Search</Link>
          <Link to="/dash/settings">Runtime Settings</Link>
          <Link to="/dash">Back to Dashboard</Link>
        </nav>
      </aside>
      <main className="dash-main">
        <header className="dash-header"><div><h1>Admin Panel</h1><p>Manage platform settings and edit any domain.</p></div></header>
        <div className="dash-content">
          <section className="dash-grid">
            <article className="dash-card"><h3>Total Domains</h3><p className="big">{zones.length}</p></article>
            <article className="dash-card span-2">
              <h3>Domain Search</h3>
              <input value={query} onChange={e => setQuery(e.target.value)} placeholder="Search domains..." />
              <div className="admin-list">
                {filtered.map(zone => (
                  <Link key={zone.id} to="/dash/domains">{zone.name} <small>{zone.records.length} records</small></Link>
                ))}
                {filtered.length === 0 && <p className="empty">No domains found.</p>}
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
