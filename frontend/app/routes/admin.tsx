import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card"

export default function Admin() {
	return (
		<div className="space-y-6 p-6">
			<div>
				<h1 className="text-3xl font-bold tracking-tight">Admin</h1>
				<p className="text-muted-foreground">Temporary admin space for future controls.</p>
			</div>

			<Card>
				<CardHeader>
					<CardTitle>Placeholder access</CardTitle>
					<CardDescription>This page exists so admin flow has a real route.</CardDescription>
				</CardHeader>
				<CardContent className="space-y-2 text-sm text-muted-foreground">
					<p>• User management</p>
					<p>• Feature flags</p>
					<p>• Audit logs</p>
				</CardContent>
			</Card>
		</div>
	)
}
