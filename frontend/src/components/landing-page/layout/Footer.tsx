import React from 'react';
import { Sparkles } from 'lucide-react';

export default function Footer() {
  const footerLinks = {
    Product: ['Features', 'How It Works', 'Pricing', 'Roadmap'],
    Developers: ['Documentation', 'API Reference', 'SDK', 'GitHub'],
    Community: ['Discord', 'Twitter', 'Blog', 'Forum'],
    Legal: ['Privacy Policy', 'Terms of Service', 'Cookie Policy'],
  };

  return (
    <footer className="relative bg-slate-950 border-t border-slate-800/50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16">
        <div className="grid grid-cols-2 md:grid-cols-5 gap-8 mb-12">
          {/* Brand */}
          <div className="col-span-2 md:col-span-1">
            <div className="flex items-center gap-3 mb-4">
              <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-violet-600 to-violet-800 flex items-center justify-center border border-violet-400/30">
                <Sparkles className="w-5 h-5 text-white" />
              </div>
              <span className="text-lg font-bold text-white">Stellar Guilds</span>
            </div>
            <p className="text-sm text-slate-500 leading-relaxed">
              Decentralized guilds, bounties, and reputation for community-driven projects.
            </p>
          </div>

          {/* Links */}
          {Object.entries(footerLinks).map(([category, links]) => (
            <div key={category}>
              <h4 className="font-semibold text-white mb-4">{category}</h4>
              <ul className="space-y-2">
                {links.map((link) => (
                  <li key={link}>
                    <a 
                      href="#" 
                      className="text-sm text-slate-500 hover:text-white transition-colors"
                    >
                      {link}
                    </a>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        {/* Bottom */}
        <div className="pt-8 border-t border-slate-800/50 flex flex-col sm:flex-row items-center justify-between gap-4">
          <p className="text-sm text-slate-500">
            © 2024 Stellar Guilds. All rights reserved.
          </p>
          <div className="flex items-center gap-2 text-sm text-slate-500">
            <span>Built on</span>
            <span className="font-semibold text-slate-400">Stellar</span>
            <span className="w-2 h-2 rounded-full bg-violet-400 animate-pulse" />
          </div>
        </div>
      </div>
    </footer>
  );
}