"use client";
import React from "react";
import Link from "next/link";
import { motion } from "framer-motion";
import { Bounty, BountyStatus } from "../types";
import { Calendar, Users, ArrowUpRight, ShieldCheck } from "lucide-react";

interface StatusBadgeProps {
  status: BountyStatus;
}

export const BountyCard = ({ bounty }: { bounty: Bounty }) => {
  return (
    <motion.div
      whileHover={{ y: -4 }}
      className="group relative bg-[#0D0D0D] border border-white/5 rounded-2xl  p-6 transition-all hover:bg-[#121212] hover:border-emerald-500/30"
    >
      <div className="flex justify-between items-start mb-5">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-full bg-emerald-500/10 flex items-center justify-center border border-emerald-500/20">
            <img
              src={bounty.guildLogo}
              alt=""
              className="w-5 h-5 rounded-full"
            />
          </div>
          <span className="text-[10px] uppercase tracking-[0.2em] text-slate-500 font-bold">
            {bounty.guildName}
          </span>
        </div>
        <StatusBadge status={bounty.status} />
      </div>

      <Link href={`/bounties/${bounty.id}`}>
        <h3 className="text-xl font-bold max-w-[400px] text-white truncate mb-2 group-hover:text-emerald-400 transition-colors">
          {bounty.title}
        </h3>
      </Link>

      <p className="text-sm text-slate-400 line-clamp-2 mb-6 font-light leading-relaxed">
        {bounty.description}
      </p>

      <div className="flex flex-wrap gap-2 mb-6">
        {bounty.tags.map((tag) => (
          <span
            key={tag}
            className="px-2 py-0.5 rounded-md bg-white/5 text-[10px] text-slate-300 border border-white/5"
          >
            {tag}
          </span>
        ))}
      </div>

      <div className="flex items-center justify-between pt-5 border-t border-white/5">
        <div className="flex flex-col">
          <span className="text-[10px] uppercase text-slate-500 font-bold tracking-tighter">
            Reward
          </span>
          <div className="flex items-center gap-2">
            <span className="text-2xl font-black text-white italic tracking-tighter">
              {bounty.rewardAmount}
            </span>
            <span className="text-sm font-medium text-emerald-500 tracking-widest">
              {bounty.tokenSymbol}
            </span>
          </div>
        </div>
        <Link
          href={`/bounties/${bounty.id}`}
          className="w-10 h-10 rounded-full bg-white/5 flex items-center justify-center text-white group-hover:bg-emerald-500 group-hover:text-black transition-all"
        >
          <ArrowUpRight size={20} />
        </Link>
      </div>
    </motion.div>
  );
};

export const StatusBadge = ({ status }: StatusBadgeProps) => {
  const styles: Record<BountyStatus, string> = {
    Open: "bg-emerald-500/10 text-emerald-500 border-emerald-500/20",
    Claimed: "bg-blue-500/10 text-blue-400 border-blue-500/20",
    "Under Review": "bg-amber-500/10 text-amber-500 border-amber-500/20",
    Completed: "bg-slate-800 text-slate-400 border-white/5",
  };

  return (
    <span
      className={`px-2 py-1 rounded-md text-[9px] font-black uppercase tracking-widest border ${styles[status]}`}
    >
      {status}
    </span>
  );
};
