"use client";
import React, { useState, useMemo } from "react";
import { MOCK_BOUNTIES } from "@/lib/mocks/bounties";
import { BountyCard } from "@/features/bounties/components/BountyCard";
import {
  LayoutDashboard,
  CheckCircle2,
  Hammer,
  PlusCircle,
  TrendingUp,
  Zap,
  History,
  Target,
  Trophy,
} from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import Link from "next/link";

type TabType = "Active" | "Completed" | "Created";

export default function MyBountiesDashboard() {
  const [activeTab, setActiveTab] = useState<TabType>("Active");

  const displayData = useMemo(() => {
    switch (activeTab) {
      case "Active":
        return MOCK_BOUNTIES.filter((b) => b.status === "Claimed" || b.status === "Under Review");
      case "Completed":
        return MOCK_BOUNTIES.filter((b) => b.status === "Completed");
      case "Created":
        return MOCK_BOUNTIES.slice(0, 1);
      default:
        return [];
    }
  }, [activeTab]);

  return (
    <div className="min-h-screen bg-[#050505] text-white p-4 md:p-8 lg:p-12 selection:bg-emerald-500/30">
      <div className="max-w-7xl mx-auto space-y-12">
        
        <header className="flex flex-col md:flex-row md:items-end justify-between gap-8">
          <div className="space-y-4">
            <div className="flex items-center gap-3 text-emerald-500">
              <div className="p-2 bg-emerald-500/10 rounded-lg border border-emerald-500/20">
                <LayoutDashboard size={20} />
              </div>
              <span className="text-[10px] font-black uppercase tracking-[0.5em]">
                System Status: Operational
              </span>
            </div>
            <h1 className="text-5xl md:text-7xl font-black italic tracking-tighter uppercase leading-none">
              Command <span className="text-emerald-500">Center</span>
            </h1>
          </div>

          <div className="flex flex-wrap gap-4">
            <Link href="/bounties/create">
              <button className="flex items-center gap-3 bg-white text-black px-8 py-4 rounded-2xl font-black uppercase text-[10px] tracking-widest hover:bg-emerald-500 transition-all active:scale-95 shadow-[0_0_20px_rgba(255,255,255,0.1)]">
                <PlusCircle size={14} />
                Deploy New Mission
              </button>
            </Link>
          </div>
        </header>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <StatCard
            label="Total Revenue"
            value="$12,840"
            trend="+12% this month"
            icon={<Zap className="text-emerald-500" size={16} />}
          />
          <StatCard
            label="Success Rate"
            value="98.2%"
            trend="Elite Tier"
            icon={<Target className="text-emerald-500" size={16} />}
          />
          <StatCard
            label="Reputation"
            value="Lv. 24"
            trend="Rank #142 Global"
            icon={<Trophy className="text-emerald-500" size={16} />}
          />
          <StatCard
            label="Avg. Payout"
            value="1,400 USDC"
            trend="Stable"
            icon={<TrendingUp className="text-emerald-500" size={16} />}
          />
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-12 gap-12">
          
          <div className="lg:col-span-8 space-y-8">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 border-b border-white/5 pb-6">
              <div className="flex gap-2 bg-white/5 p-1 rounded-2xl border border-white/5 w-fit">
                {(["Active", "Completed", "Created"] as TabType[]).map((tab) => (
                  <button
                    key={tab}
                    onClick={() => setActiveTab(tab)}
                    className={`px-6 py-2.5 rounded-xl text-[10px] font-black uppercase tracking-widest transition-all ${
                      activeTab === tab
                        ? "bg-emerald-500 text-black shadow-[0_0_20px_rgba(16,185,129,0.2)]"
                        : "text-slate-500 hover:text-slate-300"
                    }`}
                  >
                    {tab}
                  </button>
                ))}
              </div>
              <span className="text-[10px] font-mono text-slate-500 uppercase tracking-widest">
                Viewing {displayData.length} records
              </span>
            </div>

            <AnimatePresence mode="wait">
              <motion.div
                key={activeTab}
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: 20 }}
                className="grid grid-cols-1 md:grid-cols-2 gap-6"
              >
                {displayData.length > 0 ? (
                  displayData.map((bounty) => (
                    <BountyCard key={bounty.id} bounty={bounty} />
                  ))
                ) : (
                  <EmptyState tab={activeTab} />
                )}
              </motion.div>
            </AnimatePresence>
          </div>

          <div className="lg:col-span-4 space-y-8">
            
            <div className="bg-white/[0.02] border border-white/5 rounded-[32px] p-8 space-y-8">
              <div className="flex items-center gap-3 border-b border-white/5 pb-6">
                <History className="text-slate-500" size={18} />
                <h3 className="text-[10px] font-black uppercase tracking-[0.3em] text-slate-500">
                  Recent Activity
                </h3>
              </div>

              <div className="space-y-8">
                <ActivityItem 
                  title="Submission Under Review" 
                  desc="ZKP Voting Module // PrivacyGuard" 
                  time="2h ago" 
                  status="review"
                />
                <ActivityItem 
                  title="Bounty Claimed" 
                  desc="Three.js Hero Section // Stellar" 
                  time="5h ago" 
                  status="claim"
                />
                <ActivityItem 
                  title="Payment Received" 
                  desc="+500 USDC // Web3 Interface" 
                  time="1d ago" 
                  status="success"
                />
              </div>

              <button className="w-full py-4 bg-white/5 border border-white/10 rounded-2xl text-[10px] font-black uppercase tracking-widest text-slate-400 hover:text-white transition-all">
                View Full Audit Log
              </button>
            </div>

            {/* Guild Connect / Ads */}
            <div className="relative group overflow-hidden bg-gradient-to-br from-emerald-600 to-emerald-900 rounded-[32px] p-8">
                <div className="absolute -right-8 -bottom-8 opacity-20 group-hover:scale-110 transition-transform">
                    <Zap size={180} />
                </div>
                <div className="relative z-10 space-y-4">
                    <h3 className="text-2xl font-black italic tracking-tighter uppercase leading-tight">
                        Elevate your <br/> Standing
                    </h3>
                    <p className="text-emerald-100/70 text-xs font-light leading-relaxed">
                        Top 5% contributors get access to private high-yield pools. Keep shipping.
                    </p>
                </div>
            </div>

          </div>
        </div>
      </div>
    </div>
  );
}


const StatCard = ({ label, value, trend, icon }: any) => (
  <div className="bg-[#0A0A0A] border border-white/5 p-8 rounded-[32px] hover:border-emerald-500/30 transition-all group">
    <div className="flex justify-between items-start mb-6">
      <div className="p-3 bg-white/5 rounded-2xl border border-white/5 group-hover:bg-emerald-500/10 transition-colors">
        {icon}
      </div>
      <span className="text-[9px] font-black text-emerald-500 uppercase tracking-widest px-2 py-1 bg-emerald-500/10 rounded-lg">
        {trend}
      </span>
    </div>
    <p className="text-[10px] font-black uppercase text-slate-500 tracking-[0.2em] mb-2">{label}</p>
    <p className="text-4xl font-black italic tracking-tighter text-white">{value}</p>
  </div>
);

const ActivityItem = ({ title, desc, time, status }: any) => {
  const dots: any = {
    review: "bg-amber-500 shadow-[0_0_10px_rgba(245,158,11,0.5)]",
    claim: "bg-blue-500 shadow-[0_0_10px_rgba(59,130,246,0.5)]",
    success: "bg-emerald-500 shadow-[0_0_10px_rgba(16,185,129,0.5)]"
  };

  return (
    <div className="flex gap-4 relative">
      <div className={`mt-1.5 w-2 h-2 rounded-full shrink-0 ${dots[status]}`} />
      <div className="space-y-1">
        <p className="text-xs font-bold text-slate-200">{title}</p>
        <p className="text-[10px] text-slate-500 leading-relaxed">{desc}</p>
        <p className="text-[9px] font-mono text-slate-600 uppercase pt-1">{time}</p>
      </div>
    </div>
  );
};

const EmptyState = ({ tab }: { tab: string }) => (
  <div className="col-span-full border-2 border-dashed border-white/5 rounded-[40px] py-32 flex flex-col items-center justify-center text-center bg-white/[0.01]">
    <div className="w-20 h-20 bg-white/5 rounded-full flex items-center justify-center mb-6 text-slate-700">
      {tab === "Completed" ? <CheckCircle2 size={32} /> : <Hammer size={32} />}
    </div>
    <h3 className="text-2xl font-black italic tracking-tighter text-slate-400 mb-2 uppercase">Sector Empty</h3>
    <p className="text-slate-600 text-sm max-w-xs font-light">
      {tab === "Active"
        ? "No active missions detected. Scour the marketplace for new opportunities."
        : "Historical records for this sector are currently empty."}
    </p>
  </div>
);