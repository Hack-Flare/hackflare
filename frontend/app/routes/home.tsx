import { Link } from "react-router"
import Footer from "~/components/footer"
import { DarkModeToggle } from "~/components/dark-mode-toggle"

export default function Home() {
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
              href="#features"
              className="hover:text-zinc-900 dark:hover:text-white"
            >
              Features
            </a>
            <a
              href="#how-it-works"
              className="hover:text-zinc-900 dark:hover:text-white"
            >
              How it works
            </a>
            <a
              href="/team"
              className="hover:text-zinc-900 dark:hover:text-white"
            >
              Team
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
          <h1 className="mb-0 max-w-205 bg-linear-to-b from-zinc-900 to-zinc-600 bg-clip-text text-4xl leading-tight font-bold text-transparent md:text-5xl lg:text-6xl dark:from-white dark:to-zinc-400">
            The DNS platform for builders.
          </h1>
          <p className="mt-5.5 max-w-160 text-lg leading-relaxed text-zinc-600 dark:text-zinc-400">
            HackFlare helps you point your domain, manage DNS records, and ship
            faster. Built by Hack Clubbers.
          </p>
          <div className="mt-8 flex flex-wrap gap-3">
            <Link
              to="/dash"
              className="hover:-tranzinc-y-0.5 inline-flex items-center justify-center rounded-[10px] bg-orange-500 px-[1.15rem] py-[0.7rem] text-sm font-semibold text-white transition-all hover:bg-orange-600"
            >
              Launch Dashboard
            </Link>
            <a
              href="/docs"
              className="inline-flex items-center justify-center rounded-[10px] border border-zinc-300 bg-white px-[1.15rem] py-[0.7rem] text-sm font-semibold text-zinc-900 transition-colors hover:bg-zinc-100 dark:border-zinc-700 dark:bg-zinc-800 dark:text-zinc-100 dark:hover:bg-zinc-700"
            >
              Read Docs
            </a>
          </div>
        </section>

        <section id="features" className="mx-auto max-w-6xl px-6 py-16">
          <h2 className="mb-8 text-2xl font-bold text-zinc-900 md:text-3xl lg:text-4xl dark:text-white">
            Everything you need to manage DNS.
          </h2>
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
            <article className="hover:-tranzinc-y-0.5 rounded-2xl border border-zinc-200 bg-white p-6 transition-all hover:border-zinc-300 dark:border-zinc-800 dark:bg-zinc-900 dark:hover:border-zinc-700">
              <h3 className="mb-2 font-semibold text-zinc-900 dark:text-white">
                Edge Performance
              </h3>
              <p className="text-sm leading-normal text-zinc-600 dark:text-zinc-400">
                Built to feel super fast and direct.
              </p>
            </article>
            <article className="hover:-tranzinc-y-0.5 rounded-2xl border border-zinc-200 bg-white p-6 transition-all hover:border-zinc-300 dark:border-zinc-800 dark:bg-zinc-900 dark:hover:border-zinc-700">
              <h3 className="mb-2 font-semibold text-zinc-900 dark:text-white">
                Good Security
              </h3>
              <p className="text-sm leading-normal text-zinc-600 dark:text-zinc-400">
                Great defaults, clear states and better UX.
              </p>
            </article>
            <article className="hover:-tranzinc-y-0.5 rounded-2xl border border-zinc-200 bg-white p-6 transition-all hover:border-zinc-300 dark:border-zinc-800 dark:bg-zinc-900 dark:hover:border-zinc-700">
              <h3 className="mb-2 font-semibold text-zinc-900 dark:text-white">
                Automation Friendly
              </h3>
              <p className="text-sm leading-normal text-zinc-600 dark:text-zinc-400">
                REST and gRPC APIs for full automation.
              </p>
            </article>
            <article className="hover:-tranzinc-y-0.5 rounded-2xl border border-zinc-200 bg-white p-6 transition-all hover:border-zinc-300 dark:border-zinc-800 dark:bg-zinc-900 dark:hover:border-zinc-700">
              <h3 className="mb-2 font-semibold text-zinc-900 dark:text-white">
                Made for Hack Club
              </h3>
              <p className="text-sm leading-normal text-zinc-600 dark:text-zinc-400">
                Built by the Hack Club community.
              </p>
            </article>
            <article className="hover:-tranzinc-y-0.5 rounded-2xl border border-zinc-200 bg-white p-6 transition-all hover:border-zinc-300 dark:border-zinc-800 dark:bg-zinc-900 dark:hover:border-zinc-700">
              <h3 className="mb-2 font-semibold text-zinc-900 dark:text-white">
                Docs That Help
              </h3>
              <p className="text-sm leading-normal text-zinc-600 dark:text-zinc-400">
                Useful docs, guides, and AI integration.
              </p>
            </article>
            <article className="hover:-tranzinc-y-0.5 rounded-2xl border border-zinc-200 bg-white p-6 transition-all hover:border-zinc-300 dark:border-zinc-800 dark:bg-zinc-900 dark:hover:border-zinc-700">
              <h3 className="mb-2 font-semibold text-zinc-900 dark:text-white">
                Global Reach
              </h3>
              <p className="text-sm leading-normal text-zinc-600 dark:text-zinc-400">
                A solid network that scales with projects.
              </p>
            </article>
          </div>
        </section>

        <section id="how-it-works" className="mx-auto max-w-6xl px-6 py-16">
          <h2 className="mb-8 text-2xl font-bold text-zinc-900 md:text-3xl lg:text-4xl dark:text-white">
            Get started in minutes.
          </h2>
          <ol className="space-y-2.5">
            <li className="relative rounded-xl border border-zinc-200 bg-white px-4 py-2.5 pl-14 text-zinc-900 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100">
              <span className="absolute top-1/2 left-4 flex h-6 w-6 -translate-y-1/2 items-center justify-center rounded-full bg-orange-100 text-xs font-bold text-orange-700 dark:bg-orange-950 dark:text-orange-400">
                1
              </span>
              <strong>Sign in.</strong> Open the dashboard and authenticate.
            </li>
            <li className="relative rounded-xl border border-zinc-200 bg-white px-4 py-2.5 pl-14 text-zinc-900 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100">
              <span className="absolute top-1/2 left-4 flex h-6 w-6 -translate-y-1/2 items-center justify-center rounded-full bg-orange-100 text-xs font-bold text-orange-700 dark:bg-orange-950 dark:text-orange-400">
                2
              </span>
              <strong>Add domains.</strong> Point nameservers to HackFlare.
            </li>
            <li className="relative rounded-xl border border-zinc-200 bg-white px-4 py-2.5 pl-14 text-zinc-900 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100">
              <span className="absolute top-1/2 left-4 flex h-6 w-6 -translate-y-1/2 items-center justify-center rounded-full bg-orange-100 text-xs font-bold text-orange-700 dark:bg-orange-950 dark:text-orange-400">
                3
              </span>
              <strong>Manage records.</strong> Add and edit DNS entries quickly.
            </li>
          </ol>
        </section>

        <section className="mt-8 bg-linear-to-r from-red-600 via-orange-600 to-orange-500 py-16">
          <div className="mx-auto max-w-6xl px-6">
            <h2 className="mb-3 max-w-2xl text-3xl font-bold text-white md:text-4xl lg:text-5xl">
              Ready to take control of your DNS?
            </h2>
            <p className="mb-8 max-w-xl text-lg text-white/92">
              Join Hack Club members simplifying DNS infrastructure with
              HackFlare.
            </p>
            <div className="flex flex-wrap gap-3">
              <Link
                to="/dash"
                className="hover:-tranzinc-y-0.5 inline-flex items-center justify-center rounded-[10px] bg-white px-[1.15rem] py-[0.7rem] text-sm font-semibold text-orange-600 transition-all"
              >
                Get Started Free
              </Link>
              <a
                href="/docs"
                className="inline-flex items-center justify-center rounded-[10px] border-2 border-white/85 px-[1.15rem] py-[0.7rem] text-sm font-semibold text-white transition-colors hover:bg-white/10"
              >
                Read Documentation
              </a>
            </div>
          </div>
        </section>
      </main>
      <Footer />
    </div>
  )
}
