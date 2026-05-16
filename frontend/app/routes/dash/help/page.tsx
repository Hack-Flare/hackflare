import { useState } from "react"
import { Button } from "~/components/ui/button"
import { Input } from "~/components/ui/input"
import { Label } from "~/components/ui/label"
import { SlackIcon } from "~/components/icons/slack"
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card"
import { MessageCircle, BookOpen, ArrowRight, ExternalLink } from "lucide-react"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "~/components/ui/dialog"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "~/components/ui/select"
import { useAuth } from "~/lib/auth-context"

const defaultForm = {
  name: "",
  email: "",
  category: "",
  message: "",
}

export default function Help() {
  const { user } = useAuth()
  const slackId = user?.slack_id
  const id = user?.id

  const [open, setOpen] = useState(false)
  const [form, setForm] = useState(defaultForm)
  const [sending, setSending] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState(false)

  const handleSubmit = async () => {
    if (!form.name || !form.message) return
    setSending(true)
    setError(null)
    setSuccess(false)
    try {
      const userMention = slackId ? `<@${slackId}>` : form.name
      const categoryLabel = form.category ? `*Category:* ${form.category}\n` : ""
      const res = await fetch("/api/v1/slack/contact", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          text: `📬 *New HackFlare Support Request*\n*From:* ${userMention} id: \`${id}\`\n${categoryLabel}*Message:*\n${form.message}`,
        }),
      })
      if (!res.ok) throw new Error("Failed to send")
      setSuccess(true)
      setForm(defaultForm)
      setTimeout(() => { setSuccess(false); setOpen(false) }, 1500)
    } catch {
      setError("Failed to send message. Please try again or reach out on Slack.")
    } finally {
      setSending(false)
    }
  }

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Help & Support</h1>
          <p className="mt-1 text-zinc-500 dark:text-zinc-400">
            Get help from the HackFlare team or community
          </p>
        </div>
        <Dialog open={open} onOpenChange={setOpen}>
          <DialogContent className="sm:max-w-md">
            <DialogHeader>
              <DialogTitle>Contact the HackFlare Team</DialogTitle>
              <DialogDescription>
                We'll get back to you as soon as possible.
              </DialogDescription>
            </DialogHeader>
            <div className="space-y-4 py-2">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="name">Name</Label>
                  <Input
                    id="name"
                    placeholder={user?.first_name ?? "Your name"}
                    value={form.name}
                    onChange={(e) => setForm({ ...form, name: e.target.value })}
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="slackid">Slack ID</Label>
                  <Input
                    id="slackid"
                    value={slackId ?? "Not found"}
                    readOnly
                    className="cursor-not-allowed opacity-60"
                  />
                </div>
              </div>
              <div className="space-y-2">
                <Label>Category</Label>
                <Select value={form.category} onValueChange={(v) => setForm({ ...form, category: v })}>
                  <SelectTrigger className="w-full">
                    <SelectValue placeholder="Select a category" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="bug">Bug Report</SelectItem>
                    <SelectItem value="feature">Feature Request</SelectItem>
                    <SelectItem value="billing">Billing</SelectItem>
                    <SelectItem value="other">Other</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label htmlFor="message">Message</Label>
                <textarea
                  id="message"
                  rows={4}
                  placeholder="Describe your issue or question..."
                  value={form.message}
                  onChange={(e) => setForm({ ...form, message: e.target.value })}
                  className="w-full rounded-md border border-zinc-200 bg-white px-3 py-2 text-sm placeholder:text-zinc-400 focus:outline-none focus:ring-2 focus:ring-orange-500/30 dark:border-zinc-800 dark:bg-zinc-950 dark:text-white dark:placeholder:text-zinc-500"
                />
              </div>
            </div>
            {error && <p className="text-sm text-red-500 dark:text-red-400">{error}</p>}
            {success && <p className="text-sm text-green-600 dark:text-green-400">✓ Message sent! We'll be in touch.</p>}
            <DialogFooter>
              <Button variant="outline" onClick={() => setOpen(false)}>Cancel</Button>
              <Button className="bg-orange-500 text-white hover:bg-orange-600" onClick={handleSubmit} disabled={sending}>
                {sending ? "Sending..." : "Send Message"}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      {/* Quick help cards */}
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <a
          href="https://hackclub.slack.com/channels/hackflare-public"
          target="_blank"
          rel="noreferrer"
          className="group block"
        >
          <Card className="h-full transition-all duration-200 hover:border-orange-500/50 hover:shadow-md">
            <CardHeader className="pb-3">
              <CardTitle className="flex items-center gap-2 text-sm font-medium text-zinc-500 dark:text-zinc-400">
                <SlackIcon className="h-4 w-4" />
                Community Support
              </CardTitle>
            </CardHeader>
            <CardContent>
              <p className="font-semibold text-zinc-900 dark:text-white">
                Join{" "}
                <span className="rounded bg-[#4A154B]/10 px-1.5 py-0.5 font-mono text-sm text-[#4A154B] dark:bg-purple-900/30 dark:text-purple-300">
                  #hackflare-public
                </span>{" "}
                on Slack
              </p>
              <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">
                Get help from the community and the team
              </p>
              <div className="mt-3 flex items-center gap-1 text-xs font-medium text-orange-500 opacity-0 transition-opacity group-hover:opacity-100">
                Open Slack <ExternalLink className="h-3 w-3" />
              </div>
            </CardContent>
          </Card>
        </a>

        <a href="/docs" className="group block">
          <Card className="h-full transition-all duration-200 hover:border-orange-500/50 hover:shadow-md">
            <CardHeader className="pb-3">
              <CardTitle className="flex items-center gap-2 text-sm font-medium text-zinc-500 dark:text-zinc-400">
                <BookOpen className="h-4 w-4" />
                Documentation
              </CardTitle>
            </CardHeader>
            <CardContent>
              <p className="font-semibold text-zinc-900 dark:text-white">
                HackFlare Docs
              </p>
              <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">
                Guides, API references, and tutorials
              </p>
              <div className="mt-3 flex items-center gap-1 text-xs font-medium text-orange-500 opacity-0 transition-opacity group-hover:opacity-100">
                Read the docs <ArrowRight className="h-3 w-3" />
              </div>
            </CardContent>
          </Card>
        </a>
      </div>

      {/* Contact form card */}
      <Card>
        <CardHeader className="border-b border-zinc-100 dark:border-zinc-800">
          <CardTitle className="flex items-center gap-2 text-base">
            <MessageCircle className="h-4 w-4 text-orange-500" />
            Contact the Team
          </CardTitle>
        </CardHeader>
        <CardContent className="pt-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="font-medium text-zinc-900 dark:text-white">
                Have a specific issue or question?
              </p>
              <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">
                Send us a message and we'll get back to you on Slack.
              </p>
            </div>
            <Button variant="orange" onClick={() => setOpen(true)} className="shrink-0">
              <MessageCircle className="h-4 w-4" />
              Open Form
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}