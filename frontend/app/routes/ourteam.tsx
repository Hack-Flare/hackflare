import { Link } from "react-router"
import Footer from "~/components/footer"
import { DarkModeToggle } from "~/components/dark-mode-toggle"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/avatar"
import { ExternalLink } from "lucide-react"

export default function Team() {
  return (
    <div className="flex min-h-screen flex-col bg-zinc-50 dark:bg-zinc-950">
      <header className="sticky top-0 z-50 border-b border-zinc-200/60 bg-white/72 backdrop-blur-md dark:border-zinc-800/60 dark:bg-zinc-900/72">
        <div className="mx-auto flex h-17 max-w-6xl items-center justify-between gap-4 px-6">
          <a
            href="/"
            className="flex items-center gap-2.5 text-base font-black dark:text-white"
          >
            <img src="/logo.svg" width={36} alt="HackFlare" />
            <span>HackFlare</span>
          </a>

          <nav className="flex items-center gap-5.5 text-sm font-medium text-zinc-700 dark:text-zinc-300">
            <a
              href="/#features"
              className="hover:text-zinc-900 dark:hover:text-white"
            >
              Features
            </a>
            <a
              href="/#how-it-works"
              className="hover:text-zinc-900 dark:hover:text-white"
            >
              How it works
            </a>
            <a
              href="/docs"
              className="hover:text-zinc-900 dark:hover:text-white"
            >
              Docs
            </a>
          </nav>

          <div className="flex items-center gap-5.5">
            <DarkModeToggle />
            <a
              href="https://github.com/Hack-Flare/hackflare"
              className="text-sm font-medium text-zinc-700 hover:text-zinc-900 dark:text-zinc-300 dark:hover:text-white"
            >
              GitHub
            </a>
            <a
              href="/dash"
              className="text-sm font-medium text-zinc-700 hover:text-zinc-900 dark:text-zinc-300 dark:hover:text-white"
            >
              Sign in
            </a>
            <Link
              to="/dash"
              className="inline-flex items-center justify-center rounded-[10px] bg-orange-500 px-[1.15rem] py-[0.7rem] text-sm font-semibold text-white transition-colors hover:bg-orange-600"
            >
              Get Started
            </Link>
          </div>
        </div>
      </header>

      <main className="flex-1">
        <section className="mx-auto max-w-6xl px-6 py-24">
          <h1 className="mb-0 bg-linear-to-b from-zinc-900 to-zinc-600 bg-clip-text text-4xl leading-tight font-bold text-transparent md:text-5xl lg:text-6xl dark:from-white dark:to-zinc-400">
            Meet the team.
          </h1>
          <p className="mt-5.5 max-w-160 text-lg leading-relaxed text-zinc-600 dark:text-zinc-400">
            Built by passionate Hack Club members dedicated to making DNS
            management simple and fast.
          </p>
        </section>

        {/* Core Team */}
        <section className="mx-auto max-w-6xl px-6 py-16">
          <h2 className="mb-12 text-3xl font-bold text-zinc-900 dark:text-white">
            Core Team
          </h2>
          <div className="grid grid-cols-1 gap-6 md:grid-cols-3">
            {/* Vejas */}
            <Card>
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <CardTitle>Vejas</CardTitle>
                    <CardDescription>Co-Founder & Frontend dev</CardDescription>
                  </div>
                  <Avatar className="h-12 w-12">
                    <AvatarFallback className="bg-orange-500 font-semibold text-white">
                      V
                    </AvatarFallback>
                  </Avatar>
                </div>
              </CardHeader>
              <CardContent className="space-y-4">
                <p className="text-sm text-zinc-600 dark:text-zinc-400">
                  Full-stack engineer passionate about developer tools and
                  infrastructure. Leads HackFlare vision and architecture.
                </p>
                <div className="flex gap-3">
                  <a
                    href="https://vejas.zip"
                    target="_blank"
                    rel="noreferrer"
                    className="flex items-center gap-1 text-xs font-medium text-zinc-600 hover:text-orange-500 dark:text-zinc-400 dark:hover:text-orange-400"
                  >
                    Website <ExternalLink className="h-4 w-4" />
                  </a>
                  <a
                    href="https://github.com/las-vejas"
                    target="_blank"
                    rel="noreferrer"
                    className="flex items-center gap-1 text-xs font-medium text-zinc-600 hover:text-orange-500 dark:text-zinc-400 dark:hover:text-orange-400"
                  >
                    Github <ExternalLink className="h-4 w-4" />
                  </a>
                </div>
              </CardContent>
            </Card>

            {/* Nayte */}
            <Card>
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <CardTitle>
                      Nayte{" "}
                      <span className="text-[10px] text-gray-300">
                        (SeradedStripes)
                      </span>
                    </CardTitle>
                    <CardDescription>Co-founder & Backend dev</CardDescription>
                  </div>
                  <Avatar className="h-12 w-12">
                    <AvatarFallback className="bg-purple-500 font-semibold text-white">
                      N
                    </AvatarFallback>
                  </Avatar>
                </div>
              </CardHeader>
              <CardContent className="space-y-4">
                <p className="text-sm text-zinc-600 dark:text-zinc-400">
                  Backend specialist focused on API design and system
                  reliability. Ensures HackFlare runs smoothly at scale.
                </p>
                <div className="flex gap-3">
                  <a
                    href="https://kirze.de"
                    target="_blank"
                    rel="noreferrer"
                    className="flex items-center gap-1 text-xs font-medium text-zinc-600 hover:text-orange-500 dark:text-zinc-400 dark:hover:text-orange-400"
                  >
                    Website <ExternalLink className="h-4 w-4" />
                  </a>
                  <a
                    href="https://github.com/seradedstripes"
                    target="_blank"
                    rel="noreferrer"
                    className="flex items-center gap-1 text-xs font-medium text-zinc-600 hover:text-orange-500 dark:text-zinc-400 dark:hover:text-orange-400"
                  >
                    Github <ExternalLink className="h-4 w-4" />
                  </a>
                </div>
              </CardContent>
            </Card>

            {/* Johann */}
            <Card>
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <CardTitle>
                      Johann{" "}
                      <span className="text-[10px] text-gray-300">
                        (Vimthusiast)
                      </span>
                    </CardTitle>
                    <CardDescription>Backend Developer</CardDescription>
                  </div>
                  <Avatar className="h-12 w-12">
                    <AvatarFallback className="bg-blue-500 font-semibold text-white">
                      J
                    </AvatarFallback>
                  </Avatar>
                </div>
              </CardHeader>
              <CardContent className="space-y-4">
                <p className="text-sm text-zinc-600 dark:text-zinc-400">
                  Frontend engineer obsessed with UX and performance. Crafts
                  HackFlare's beautiful, responsive interface.
                </p>
                <div className="flex gap-3">
                  {/*<a href="https://" target="_blank" rel="noreferrer" className="text-zinc-600 hover:text-orange-500 dark:text-zinc-400 dark:hover:text-orange-400 text-xs font-medium flex items-center gap-1">
                        Website <ExternalLink className="h-4 w-4" />
                    </a>*/}
                  <a
                    href="https://github.com/Vimthusiast"
                    target="_blank"
                    rel="noreferrer"
                    className="flex items-center gap-1 text-xs font-medium text-zinc-600 hover:text-orange-500 dark:text-zinc-400 dark:hover:text-orange-400"
                  >
                    GitHub <ExternalLink className="h-4 w-4" />
                  </a>
                </div>
              </CardContent>
            </Card>
          </div>
        </section>

        {/* Contributors */}
        {/* <section className="mx-auto max-w-6xl px-6 py-16 border-t border-zinc-200 dark:border-zinc-800">
          <h2 className="text-3xl font-bold mb-12 text-zinc-900 dark:text-white">Contributors & Helpers</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            {[
              { name: "Design Team", role: "UI/UX & Branding", icon: "🎨" },
              { name: "Testing Squad", role: "QA & Bug Hunting", icon: "🐛" },
              { name: "Community", role: "Feedback & Ideas", icon: "👥" },
              { name: "Hack Club", role: "Mentorship & Support", icon: "🎓" },
            ].map((contributor) => (
              <Card key={contributor.name} className="hover:border-zinc-300 dark:hover:border-zinc-700 transition-colors">
                <CardContent className="pt-6">
                  <div className="text-3xl mb-2">{contributor.icon}</div>
                  <p className="font-semibold text-zinc-900 dark:text-white">{contributor.name}</p>
                  <p className="text-sm text-zinc-600 dark:text-zinc-400 mt-1">{contributor.role}</p>
                </CardContent>
              </Card>
            ))}
          </div>
        </section> */}

        {/* Join Section */}
        <section className="mt-8 bg-linear-to-r from-red-600 via-orange-600 to-orange-500 py-16">
          <div className="mx-auto max-w-6xl px-6">
            <h2 className="mb-3 max-w-2xl text-3xl font-bold text-white md:text-4xl lg:text-5xl">
              Join us in building the future of DNS.
            </h2>
            <p className="mb-8 max-w-xl text-lg text-white/92">
              Interested in contributing or partnering with HackFlare? Reach out
              to the team on GitHub or join the Hack Club community.
            </p>
            <div className="flex flex-wrap gap-3">
              <a
                href="https://github.com/Hack-Flare/hackflare"
                target="_blank"
                rel="noreferrer"
                className="inline-flex items-center justify-center rounded-[10px] bg-white px-[1.15rem] py-[0.7rem] text-sm font-semibold text-orange-600 transition-all hover:-translate-y-0.5"
              >
                Contribute on GitHub
              </a>
              <a
                href="https://hackclub.slack.com"
                target="_blank"
                rel="noreferrer"
                className="inline-flex items-center justify-center rounded-[10px] border-2 border-white/85 px-[1.15rem] py-[0.7rem] text-sm font-semibold text-white transition-colors hover:bg-white/10"
              >
                Join Slack Community
              </a>
            </div>
          </div>
        </section>
      </main>
      <Footer />
    </div>
  )
}
