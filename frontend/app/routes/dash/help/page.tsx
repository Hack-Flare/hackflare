import { useState } from "react"
import { Button } from "~/components/ui/button"
import { Input } from "~/components/ui/input"
import { Label } from "~/components/ui/label"
import { SlackIcon } from "~/components/icons/slack"
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card"
import { MessageCircle, BookOpen } from "lucide-react"
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

const SLACK_WEBHOOK =
  "https://hooks.slack.com/services/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"

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
      const categoryLabel = form.category
        ? `*Category:* ${form.category}\n`
        : ""

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
      setTimeout(() => {
        setSuccess(false)
        setOpen(false)
      }, 1500)
    } catch (err) {
      setError(
        "Failed to send message. Please try again or reach out on Slack."
      )
    } finally {
      setSending(false)
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Help</h1>
          <p className="mt-2 text-zinc-600 dark:text-zinc-400">
            Find resources about HackFlare here.
          </p>
        </div>
        <Button variant="orange" onClick={() => setOpen(true)}>
          <MessageCircle className="h-4 w-4" />
          Contact the Team
        </Button>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <SlackIcon className="h-4 w-4" />
              Hack Club Slack
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-xl font-bold">
              Join{" "}
              <a
                href=""
                className="rounded-md bg-blue-500/20 px-1 text-blue-700 dark:text-blue-300"
              >
                #hackflare-public
              </a>{" "}
              on Slack for help from the community!
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <BookOpen className="h-4 w-4" />
              HackFlare Docs
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-xl font-bold">
              You can('t) find the docs{" "}
              <a href="/docs" className="text-orange-500 hover:underline">
                here
              </a>
            </p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-sm font-medium">
            <MessageCircle className="h-4 w-4" />
            Contact Form
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="pb-3 text-zinc-600 dark:text-zinc-400">
            If you have a problem, use the button below to send a message to the
            team!
          </p>
          <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
              <Button variant="orange">Contact the Team</Button>
            </DialogTrigger>
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
                      placeholder="Your name"
                      value={form.name}
                      onChange={(e) =>
                        setForm({ ...form, name: e.target.value })
                      }
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
                  <Select
                    value={form.category}
                    onValueChange={(v) => setForm({ ...form, category: v })}
                  >
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
                    onChange={(e) =>
                      setForm({ ...form, message: e.target.value })
                    }
                    className="w-full rounded-md border border-zinc-200 bg-white px-3 py-2 text-sm placeholder:text-zinc-400 focus:ring-2 focus:ring-orange-500/30 focus:outline-none dark:border-zinc-800 dark:bg-zinc-950 dark:text-white dark:placeholder:text-zinc-500"
                  />
                </div>
              </div>
              {error && (
                <p className="text-sm text-red-500 dark:text-red-400">
                  {error}
                </p>
              )}
              {success && (
                <p className="text-sm text-green-600 dark:text-green-400">
                  ✓ Message sent! We'll be in touch.
                </p>
              )}
              <DialogFooter>
                <Button variant="outline" onClick={() => setOpen(false)}>
                  Cancel
                </Button>
                <Button
                  className="bg-orange-500 text-white hover:bg-orange-600"
                  onClick={handleSubmit}
                  disabled={sending}
                >
                  {sending ? "Sending..." : "Send Message"}
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </CardContent>
      </Card>
    </div>
  )
}
