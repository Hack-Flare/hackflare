import React from "react";
import logo from "@/logo.svg";

type FooterProps = {
  variant?: "light" | "dark";
};

export default function Footer({ variant = "light" }: FooterProps) {
  const isDark = variant === "dark";
  const outer = isDark ? "bg-slate-900 border-t border-slate-800 text-slate-300 mt-12" : "bg-white border-t border-slate-200 mt-12";
  const heading = isDark ? "font-semibold text-slate-100 mb-4" : "font-semibold text-slate-900 mb-4";
  const text = isDark ? "text-sm text-slate-400" : "text-sm text-slate-600";
  const linkHover = isDark ? "hover:text-slate-100" : "hover:text-slate-900";

  return (
    <footer className={outer}>
      <div className="max-w-4xl mx-auto px-6 py-12">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mb-8">
          <div>
            <div className={`flex items-center gap-2 font-black text-base ${isDark ? "text-slate-100 mb-4" : "text-slate-900 mb-4"}`}>
              <img src={logo} width={28} alt="HackFlare" />
              <span>HackFlare</span>
            </div>
            <p className={text}>The DNS platform for builders — simple, fast, and friendly.</p>
          </div>

          <div>
            <h4 className={heading}>Resources</h4>
            <ul className="space-y-2.5">
              <li><a href="#" className={linkHover}>API Docs</a></li>
              <li><a href="#" className={linkHover}>Guides</a></li>
              <li><a href="#" className={linkHover}>Community</a></li>
              <li><a href="#" className={linkHover}>Support</a></li>
            </ul>
          </div>

          <div>
            <h4 className={heading}>Legal</h4>
            <ul className="space-y-2.5">
              <li><a href="#" className={linkHover}>Privacy</a></li>
              <li><a href="#" className={linkHover}>Terms</a></li>
              <li><a href="#" className={linkHover}>Security</a></li>
              <li><a href="#" className={linkHover}>License</a></li>
            </ul>
          </div>
        </div>

        <div className={`pt-6 flex items-center justify-between ${isDark ? "border-t border-slate-800" : "border-t border-slate-200"}`}>
          <div className={`flex items-center gap-2 font-black text-base ${isDark ? "text-slate-100" : "text-slate-900"}`}>
            <img src={logo} width={20} alt="HackFlare" />
            <span>HackFlare</span>
          </div>
          <p className={text}>© 2026 HackFlare. All rights reserved.</p>
          <div className="flex gap-4">
            <a href="https://github.com/Hack-Flare/hackflare" className={linkHover}>Github</a>
            <a href="#" className={linkHover}>@Nayte</a>
            <a href="#" className={linkHover}>@Vejas</a>
          </div>
        </div>
      </div>
    </footer>
  );
}
