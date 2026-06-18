import { useSearchParams } from "react-router"
import { useMemo } from "react"
import Markdown from "react-markdown"
import remarkGfm from "remark-gfm"
import Footer from "~/components/footer"
import { DarkModeToggle } from "~/components/dark-mode-toggle"

const DOCS_ORDER = [
  "getting-started",
  "managing-domains",
  "dns-records",
  "ssl-tls",
  "api-reference",
  "faq",
] as const

type DocSlug = (typeof DOCS_ORDER)[number]

const docsModules = import.meta.glob("/docs/*.md", {
  query: "?raw",
  eager: true,
  import: "default",
}) as Record<string, string>

export default function Docs() {
  const [searchParams, setSearchParams] = useSearchParams()
  const currentDoc = (searchParams.get("doc") as DocSlug | null) ?? "getting-started"

  const docs = useMemo(() => {
    return Object.entries(docsModules)
      .map(([path, content]) => {
        const slug = path.replace("/docs/", "").replace(".md", "") as DocSlug
        const title = content.match(/^#\s+(.+)/m)?.[1] ?? slug
        return { slug, title, content }
      })
      .sort((a, b) => DOCS_ORDER.indexOf(a.slug) - DOCS_ORDER.indexOf(b.slug))
  }, [])

  const activeDoc = docs.find((d) => d.slug === currentDoc) ?? docs[0]

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
            <a
              href="/dash"
              className="inline-flex items-center justify-center rounded-[10px] bg-orange-500 px-[1.15rem] py-[0.7rem] text-sm font-semibold text-white transition-colors hover:bg-orange-600"
            >
              Get Started
            </a>
          </div>
        </div>
      </header>

      <div className="mx-auto flex w-full max-w-6xl flex-1 px-6">
        <aside className="hidden w-56 shrink-0 border-r border-zinc-200 py-8 pr-6 dark:border-zinc-800 md:block">
          <nav className="space-y-1">
            {docs.map((doc) => (
              <button
                key={doc.slug}
                onClick={() => setSearchParams({ doc: doc.slug })}
                className={`block w-full rounded-lg px-3 py-2 text-left text-sm font-medium transition-colors ${
                  doc.slug === currentDoc
                    ? "bg-orange-50 text-orange-700 dark:bg-orange-950 dark:text-orange-400"
                    : "text-zinc-600 hover:bg-zinc-100 hover:text-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-800 dark:hover:text-white"
                }`}
              >
                {doc.title}
              </button>
            ))}
          </nav>
        </aside>

        <main className="min-w-0 flex-1 py-8 pl-0 md:pl-8">
          <article className="prose prose-zinc max-w-none dark:prose-invert prose-headings:font-bold prose-h1:text-3xl prose-h2:text-2xl prose-h3:text-xl prose-a:text-orange-600 prose-a:no-underline hover:prose-a:underline prose-code:rounded prose-code:bg-zinc-100 prose-code:px-1.5 prose-code:py-0.5 prose-code:text-sm prose-pre:rounded-xl prose-pre:border prose-pre:border-zinc-200 dark:prose-code:bg-zinc-800 dark:prose-pre:border-zinc-700">
            <Markdown remarkPlugins={[remarkGfm]}>{activeDoc.content}</Markdown>
          </article>
        </main>
      </div>

      <Footer />
    </div>
  )
}
