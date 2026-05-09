export default function Footer() {
  return (
    <footer className="border-t border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900">
      <div className="mx-auto max-w-4xl px-6 py-16">
        <div className="grid grid-cols-4 gap-8 mb-12">
          <div>
            <h4 className="font-semibold text-slate-900 dark:text-white mb-4">Product</h4>
            <ul className="space-y-2.5 text-sm text-slate-600 dark:text-slate-400">
              <li><a href="#features" className="hover:text-slate-900 dark:hover:text-white">Features</a></li>
              <li><a href="/docs" className="hover:text-slate-900 dark:hover:text-white">Documentation</a></li>
              <li><a href="#" className="hover:text-slate-900 dark:hover:text-white">Pricing</a></li>
              <li><a href="#" className="hover:text-slate-900 dark:hover:text-white">Status</a></li>
            </ul>
          </div>
          <div>
            <h4 className="font-semibold text-slate-900 dark:text-white mb-4">Company</h4>
            <ul className="space-y-2.5 text-sm text-slate-600 dark:text-slate-400">
              <li><a href="#" className="hover:text-slate-900 dark:hover:text-white">About</a></li>
              <li><a href="#" className="hover:text-slate-900 dark:hover:text-white">Our Team</a></li>
              <li><a href="#" className="hover:text-slate-900 dark:hover:text-white">Careers</a></li>
              <li><a href="#" className="hover:text-slate-900 dark:hover:text-white">Contact</a></li>
            </ul>
          </div>
          <div>
            <h4 className="font-semibold text-slate-900 dark:text-white mb-4">Resources</h4>
            <ul className="space-y-2.5 text-sm text-slate-600 dark:text-slate-400">
              <li><a href="#" className="hover:text-slate-900 dark:hover:text-white">API Docs</a></li>
              <li><a href="#" className="hover:text-slate-900 dark:hover:text-white">Guides</a></li>
              <li><a href="#" className="hover:text-slate-900 dark:hover:text-white">Community</a></li>
              <li><a href="#" className="hover:text-slate-900 dark:hover:text-white">Support</a></li>
              </ul>
            </div>
          <div>
            <h4 className="font-semibold text-slate-900 dark:text-white mb-4">Legal</h4>
            <ul className="space-y-2.5 text-sm text-slate-600 dark:text-slate-400">
              <li><a href="#" className="hover:text-slate-900 dark:hover:text-white">Privacy</a></li>
              <li><a href="#" className="hover:text-slate-900 dark:hover:text-white">Terms</a></li>
              <li><a href="#" className="hover:text-slate-900 dark:hover:text-white">Security</a></li>
              <li><a href="#" className="hover:text-slate-900 dark:hover:text-white">License</a></li>
            </ul>
          </div>
        </div>
        <div className="border-t border-slate-200 dark:border-slate-800 pt-8 flex items-center justify-between">
          <div className="flex items-center gap-2 font-black text-base text-slate-900 dark:text-white">
            <img src="/logo.svg" width={24} alt="HackFlare" />
            <span>HackFlare</span>
          </div>
          <p className="text-sm text-slate-600 dark:text-slate-400">© 2026 HackFlare.  All rights reserved.</p>
          <a href="https://github.com/Hack-Flare/hackflare" className="text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white">GitHub</a>
          <div className="flex gap-4">
            <p className="text-slate-600 dark:text-slate-400">Our team:</p>
            <a href="https://vejas.zip" className="text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white">@Vejas</a>
            <a href="https://kirze.de" className="text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white">@Nayte</a>
          </div>
        </div>
      </div>
    </footer>
    )
  }
