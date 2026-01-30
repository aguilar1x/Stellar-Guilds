import React from "react";
import { motion } from "framer-motion";
import { LucideIcon } from "lucide-react";
import { Users, Target, Wallet, Milestone, Scale, Award } from "lucide-react";
import FeatureCard from "@/components/landing-page/ui/FeatureCard";

type AccentColor = "violet" | "cyan" | "amber" | "violet" | "rose" | "blue";

interface Feature {
  icon: LucideIcon;
  title: string;
  description: string;
  accentColor: AccentColor;
}

export default function FeaturesSection() {
  const features: Feature[] = [
    {
      icon: Users,
      title: "Guild Creation & Membership",
      description:
        "Create decentralized guilds with smart membership contracts. Define roles, voting power, and profit-sharing rules on-chain.",
      accentColor: "violet",
    },
    {
      icon: Target,
      title: "Bounty & Task Contracts",
      description:
        "Post bounties with escrowed funds. Contributors claim tasks, complete deliverables, and receive automatic payment upon approval.",
      accentColor: "cyan",
    },
    {
      icon: Wallet,
      title: "Payment Distribution",
      description:
        "Automated payment splitting based on contribution. Support for XLM, USDC, and custom Stellar tokens with instant settlement.",
      accentColor: "violet",
    },
    {
      icon: Milestone,
      title: "Milestone & Subscriptions",
      description:
        "Break large projects into milestones with staged funding release. Offer subscription-based guild memberships for recurring revenue.",
      accentColor: "amber",
    },
    {
      icon: Scale,
      title: "Dispute Resolution",
      description:
        "Community-driven arbitration system. Stake-weighted voting ensures fair outcomes with economic incentives for honest participation.",
      accentColor: "rose",
    },
    {
      icon: Award,
      title: "Reputation & Incentives",
      description:
        "Build portable reputation through completed bounties. Earn badges, unlock premium opportunities, and showcase your track record.",
      accentColor: "blue",
    },
  ];

  return (
    <section
      id="features"
      className="relative py-32 bg-slate-950 overflow-hidden"
    >
      {/* Background elements */}
      <div className="absolute inset-0">
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[800px] h-[400px] bg-violet-500/5 rounded-full blur-[120px]" />
      </div>

      <div className="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Section header */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6 }}
          viewport={{ once: true }}
          className="text-center mb-20"
        >
          <span className="inline-block px-4 py-2 rounded-full bg-violet-500/10 border border-violet-500/20 text-violet-400 text-sm font-medium mb-6">
            Core Features
          </span>
          <h2 className="text-3xl sm:text-4xl md:text-5xl font-bold text-white mb-6">
            Everything You Need to{" "}
            <span className="bg-gradient-to-r from-violet-400 to-violet-200 bg-clip-text text-transparent">
              Build & Earn
            </span>
          </h2>
          <p className="text-lg text-slate-400 max-w-2xl mx-auto">
            A complete suite of smart contract tools designed for developer
            communities, DAOs, and collaborative teams.
          </p>
        </motion.div>

        {/* Features grid */}
        <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-6">
          {features.map((feature, index) => (
            <FeatureCard key={feature.title} {...feature} delay={index * 0.1} />
          ))}
        </div>

        {/* Bottom decoration */}
        <motion.div
          initial={{ opacity: 0, scale: 0.8 }}
          whileInView={{ opacity: 1, scale: 1 }}
          transition={{ duration: 0.6 }}
          viewport={{ once: true }}
          className="mt-20 text-center"
        >
          <div className="inline-flex items-center gap-2 px-6 py-3 rounded-full bg-slate-800/50 border border-slate-700/50">
            <span className="text-slate-400">Powered by</span>
            <span className="font-semibold text-white">Stellar Soroban</span>
            <span className="w-2 h-2 rounded-full bg-violet-400 animate-pulse" />
          </div>
        </motion.div>
      </div>
    </section>
  );
}
