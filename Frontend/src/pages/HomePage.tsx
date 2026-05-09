import React from "react";
import { Link } from "react-router-dom";
import logo from "@/logo.svg";

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

export default HomePage;

