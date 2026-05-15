import { Button } from "~/components/ui/button"
import { SlackIcon } from "~/components/icons/slack"
import {
  Card,
  CardAction,
  CardDescription,
  CardFooter,
  CardContent,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"
import { MessageCircle, BookOpen, Book } from "lucide-react"

export default function Help() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold dark:text-white">Help</h1>
          <p className="mt-2 text-zinc-600 dark:text-zinc-400">
            Find resources about HackFlare here.
          </p>
        </div>
        <Button variant="orange">
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
            <p className="text-xl font-bold">Join <a href="" className="bg-blue-500/50 text-blue-900 px-1 rounded-md dark:text-blue-100">#hackflare-public</a> on Slack for help from the community!</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-sm font-medium">
              <BookOpen className="h-4 w-4" />
              HackFlare docs
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-xl font-bold">You can find the docs <a href="/docs">here</a></p> 
          </CardContent>
        </Card>

      </div>
    </div>
  )
}
