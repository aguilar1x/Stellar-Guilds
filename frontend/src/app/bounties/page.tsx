"use client";

import React, { useState, useMemo } from "react";
import { MOCK_BOUNTIES } from "@/lib/mocks/bounties";
import { BountyCard } from "@/features/bounties/components/BountyCard";
import {
  Search,
  SlidersHorizontal,
  Zap,
  XCircle,
  LayoutDashboard,
  Target,
  Activity,
  Globe,
  ShieldCheck,
  Code2,
  Palette,
  Megaphone,
} from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import Link from "next/link";

type SortOption = "Newest" | "Highest Reward" | "Expiring Soon";

export default function MarketplacePage() {
  const [search, setSearch] = useState("");
  const [filterStatus, setFilterStatus] = useState("All");
  const [sortBy, setSortBy] = useState<SortOption>("Newest");
  const [activeCategory, setActiveCategory] = useState("All");

  const filteredAndSortedBounties = useMemo(() => {
    let result = MOCK_BOUNTIES.filter((bounty) => {
      const matchesSearch = bounty.title
        .toLowerCase()
        .includes(search.toLowerCase());
      const matchesStatus =
        filterStatus === "All" ||
        bounty.status.toLowerCase() === filterStatus.toLowerCase();
      return matchesSearch && matchesStatus;
    });

    return result.sort((a, b) => {
      if (sortBy === "Highest Reward") return b.rewardAmount - a.rewardAmount;
      if (sortBy === "Expiring Soon")
        return new Date(a.deadline).getTime() - new Date(b.deadline).getTime();
      if (sortBy === "Newest") return Number(b.id) - Number(a.id);
      return 0;
    });
  }, [search, filterStatus, sortBy]);

  const clearFilters = () => {
    setSearch("");
    setFilterStatus("All");
    setSortBy("Newest");
    setActiveCategory("All");
  };

  return (
    <div className="min-h-screen w-full bg-[#050505] text-white selection:bg-emerald-500/30">
      <div className="w-full bg-emerald-500/5 border-b border-emerald-500/10 py-2 overflow-hidden whitespace-nowrap">
        <motion.div
          animate={{ x: [0, -1000] }}
          transition={{ duration: 30, repeat: Infinity, ease: "linear" }}
          className="flex gap-12 items-center"
        >
          {[1, 2, 3, 4, 5].map((i) => (
            <div
              key={i}
              className="flex gap-8 items-center text-[9px] font-mono text-emerald-500/60 uppercase tracking-[0.2em]"
            >
              <span className="flex items-center gap-2">
                <Activity size={10} /> Network Load: Stable
              </span>
              <span className="flex items-center gap-2">
                <Globe size={10} /> Global Payouts: 4.2M USDC
              </span>
              <span className="flex items-center gap-2">
                <ShieldCheck size={10} /> Security Level: High
              </span>
            </div>
          ))}
        </motion.div>
      </div>

      <div className="relative border-b border-white/5 bg-[#080808] overflow-hidden">
        <div className="absolute top-0 right-0 w-[600px] h-[600px] bg-emerald-500/10 blur-[140px] rounded-full -translate-y-1/2 translate-x-1/4" />
        <div className="max-w-7xl mx-auto px-6 py-16 relative z-10">
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="flex items-center gap-2 mb-6 text-emerald-500"
          >
            <Zap className="fill-emerald-500" size={14} />
            <span className="text-[10px] font-black uppercase tracking-[0.4em]">
              Protocol Active
            </span>
          </motion.div>
          <h1 className="text-6xl md:text-8xl font-black italic tracking-tighter mb-4 uppercase leading-none">
            Forge <span className="text-emerald-500">Value.</span>
          </h1>
          <p className="text-slate-500 text-lg max-w-xl font-light">
            Deploy your skills to the world most aggressive guilds.
          </p>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-6 py-12 grid grid-cols-1 lg:grid-cols-12 gap-16">
        <aside className="lg:col-span-3 space-y-10">
          <div className="space-y-4">
            <h4 className="text-[10px] font-black uppercase tracking-[0.3em] pb-2 text-slate-500">
              Contributor Hub
            </h4>
            <Link href="/bounties/my-bounties">
              <div className="group relative bg-emerald-500/10 border border-emerald-500/20 rounded-2xl p-4 hover:border-emerald-500/50 transition-all cursor-pointer overflow-hidden">
                <div className="absolute -right-4 -bottom-4 text-emerald-500/10 group-hover:scale-110 transition-transform">
                  <LayoutDashboard size={80} />
                </div>
                <div className="flex items-center gap-3 mb-1">
                  <LayoutDashboard size={16} className="text-emerald-500" />
                  <span className="text-xs font-black uppercase">
                    My Operations
                  </span>
                </div>
                <p className="text-[10px] text-emerald-100/60 font-light">
                  Track earnings & active claims
                </p>
              </div>
            </Link>
          </div>

          <div className="space-y-4">
            <h4 className="text-[10px] font-black uppercase tracking-[0.3em] text-slate-500">
              Command Search
            </h4>
            <div className="relative group">
              <Search
                className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-600"
                size={16}
              />
              <input
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder="Find missions..."
                className="w-full bg-white/5 border border-white/10 rounded-2xl py-4 pl-12 pr-4 text-sm focus:border-emerald-500/50 transition-all outline-none"
              />
            </div>
          </div>

          <div className="space-y-4">
            <h4 className="text-[10px] font-black uppercase tracking-[0.3em] text-slate-500">
              Sectors
            </h4>
            <div className="grid grid-cols-2 gap-2">
              <CategoryBtn
                icon={<Code2 size={12} />}
                label="Dev"
                active={activeCategory === "Dev"}
                onClick={() => setActiveCategory("Dev")}
              />
              <CategoryBtn
                icon={<Palette size={12} />}
                label="Design"
                active={activeCategory === "Design"}
                onClick={() => setActiveCategory("Design")}
              />
              <CategoryBtn
                icon={<ShieldCheck size={12} />}
                label="Security"
                active={activeCategory === "Security"}
                onClick={() => setActiveCategory("Security")}
              />
              <CategoryBtn
                icon={<Megaphone size={12} />}
                label="Growth"
                active={activeCategory === "Growth"}
                onClick={() => setActiveCategory("Growth")}
              />
            </div>
          </div>

          <div className="space-y-4">
            <h4 className="text-[10px] font-black uppercase tracking-[0.3em] text-slate-500">
              Status
            </h4>
            <div className="flex flex-col gap-2">
              {["All", "Open", "Claimed", "Completed"].map((status) => (
                <button
                  key={status}
                  onClick={() => setFilterStatus(status)}
                  className={`text-left px-4 py-3 rounded-xl text-[10px] font-black uppercase tracking-widest transition-all border ${
                    filterStatus === status
                      ? "bg-white text-black border-white"
                      : "text-slate-500 border-white/5 hover:bg-white/5"
                  }`}
                >
                  {status}
                </button>
              ))}
            </div>
          </div>
        </aside>

        <main className="lg:col-span-9">
          <div className="flex flex-col md:flex-row justify-between items-center mb-10 pb-6 border-b border-white/5 gap-4">
            <div className="flex items-center gap-6">
              <div className="flex items-center gap-2">
                <Target size={14} className="text-emerald-500" />
                <span className="text-[10px] font-mono font-bold">
                  {filteredAndSortedBounties.length} RECOGNIZED MISSIONS
                </span>
              </div>
            </div>

            <div className="flex items-center gap-4">
              <select
                value={sortBy}
                onChange={(e) => setSortBy(e.target.value as SortOption)}
                className="bg-transparent text-[10px] font-black text-slate-400 uppercase tracking-widest outline-none cursor-pointer focus:text-white"
              >
                <option value="Newest" className="bg-[#0A0A0A]">
                  Sort // Newest
                </option>

                <option value="Highest Reward" className="bg-[#0A0A0A]">
                  Sort // Max Yield
                </option>

                <option value="Expiring Soon" className="bg-[#0A0A0A]">
                  Sort // Urgent
                </option>
              </select>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 min-h-[500px]">
            <AnimatePresence mode="popLayout">
              {filteredAndSortedBounties.length > 0 ? (
                filteredAndSortedBounties.map((bounty) => (
                  <motion.div
                    key={bounty.id}
                    layout
                    initial={{ opacity: 0, scale: 0.95 }}
                    animate={{ opacity: 1, scale: 1 }}
                    exit={{ opacity: 0, scale: 0.95 }}
                    transition={{ duration: 0.2 }}
                  >
                    <BountyCard bounty={bounty} />
                  </motion.div>
                ))
              ) : (
                <EmptyState onReset={clearFilters} />
              )}
            </AnimatePresence>
          </div>
        </main>
      </div>
    </div>
  );
}

const CategoryBtn = ({ icon, label, active, onClick }: any) => (
  <button
    onClick={onClick}
    className={`flex items-center justify-center gap-2 py-3 rounded-xl border text-[10px] font-black uppercase tracking-widest transition-all ${
      active
        ? "bg-emerald-500 border-emerald-500 text-black"
        : "bg-white/5 border-white/10 text-slate-500 hover:text-white"
    }`}
  >
    {icon} {label}
  </button>
);

const EmptyState = ({ onReset }: any) => (
  <div className="col-span-full flex flex-col items-center justify-center py-32 border-2 border-dashed border-white/5 rounded-[40px]">
    <XCircle size={40} className="text-slate-800 mb-4" />
    <h3 className="text-xl font-black italic tracking-tighter text-slate-500 mb-6 uppercase">
      Sector Dark
    </h3>
    <button
      onClick={onReset}
      className="px-8 py-3 bg-emerald-500 text-black text-[10px] font-black uppercase tracking-widest rounded-full hover:shadow-[0_0_20px_rgba(16,185,129,0.4)] transition-all"
    >
      Refresh Terminal
    </button>
  </div>
);
