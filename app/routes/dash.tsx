export default function Dashboard() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
        <p className="text-muted-foreground">Quick snapshot of domains, DNS, and account health.</p>
      </div>

      <div className="grid gap-4 md:grid-cols-3">
        <div className="rounded-lg border bg-card p-6">
          <p className="text-sm text-muted-foreground">Active domains</p>
          <p className="mt-2 text-3xl font-bold">12</p>
        </div>
        <div className="rounded-lg border bg-card p-6">
          <p className="text-sm text-muted-foreground">DNS changes today</p>
          <p className="mt-2 text-3xl font-bold">4</p>
        </div>
        <div className="rounded-lg border bg-card p-6">
          <p className="text-sm text-muted-foreground">Services healthy</p>
          <p className="mt-2 text-3xl font-bold">Yes</p>
        </div>
      </div>

      <div className="rounded-lg border bg-card p-6">
        <h2 className="text-lg font-semibold">Recent activity</h2>
        <ul className="mt-4 space-y-2 text-sm text-muted-foreground">
          <li>• domain added to project</li>
          <li>• DNS record updated</li>
          <li>• firewall rule created</li>
        </ul>
      </div>
    </div>
  )
}
