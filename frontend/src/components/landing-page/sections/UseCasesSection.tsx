import React from "react";
import { motion } from "framer-motion";
import {
  Code2,
  Palette,
  FlaskConical,
  Briefcase,
  Gamepad2,
} from "lucide-react";
import UseCaseCard from "@/components/landing-page/ui/UseCaseCard";
import StatCounter from "@/components/landing-page/ui/StatCounter";

export default function UseCasesSection() {
  const useCases = [
    {
      icon: Code2,
      title: "Open Source Communities",
      description:
        "Fund development, reward contributors, and build transparent governance for your project.",
      gradient: "from-violet-600 to-indigo-700",
    },
    {
      icon: Palette,
      title: "Creative Guilds",
      description:
        "Designers, artists, and content creators forming collectives with fair revenue sharing.",
      gradient: "from-pink-600 to-rose-700",
    },
    {
      icon: FlaskConical,
      title: "Research DAOs",
      description:
        "Decentralized science initiatives funding research and distributing findings openly.",
      gradient: "from-cyan-600 to-blue-700",
    },
    {
      icon: Briefcase,
      title: "Freelance Collectives",
      description:
        "Independent professionals pooling resources and sharing opportunities fairly.",
      gradient: "from-amber-600 to-orange-700",
    },
    {
      icon: Gamepad2,
      title: "Gaming Guilds",
      description:
        "Play-to-earn teams managing assets, distributing earnings, and coordinating play.",
      gradient: "from-violet-600 to-teal-700",
    },
  ];

  const stats = [
    { value: "250", suffix: "+", label: "Active Guilds" },
    { value: "12500", suffix: "+", label: "Bounties Completed" },
    { value: "850000", prefix: "$", label: "Paid to Contributors" },
    { value: "4800", suffix: "+", label: "Active Members" },
  ];

  return (
    <section
      id="use-cases"
      className="relative py-32 bg-slate-900 overflow-hidden"
    >
      {/* Background */}
      <div className="absolute inset-0">
        <div className="absolute inset-0 bg-gradient-to-b from-slate-950 via-slate-900 to-slate-950" />
      </div>

      <div className="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Stats section */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6 }}
          viewport={{ once: true }}
          className="mb-24"
        >
          <div className="relative p-8 md:p-12 rounded-3xl bg-gradient-to-br from-violet-900/30 to-slate-900/50 border border-violet-500/20 overflow-hidden">
            {/* Animated gradient background */}
            <div className="absolute inset-0 bg-gradient-to-r from-violet-500/10 via-cyan-500/10 to-violet-500/10 animate-pulse" />

            <div className="relative z-10 grid grid-cols-2 md:grid-cols-4 gap-8">
              {stats.map((stat, index) => (
                <StatCounter key={index} {...stat} />
              ))}
            </div>
          </div>
        </motion.div>

        {/* Header */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6 }}
          viewport={{ once: true }}
          className="text-center mb-16"
        >
          <span className="inline-block px-4 py-2 rounded-full bg-violet-500/10 border border-violet-500/20 text-violet-400 text-sm font-medium mb-6">
            Use Cases
          </span>
          <h2 className="text-3xl sm:text-4xl md:text-5xl font-bold text-white mb-6">
            Built for Every{" "}
            <span className="bg-gradient-to-r from-violet-400 to-violet-200 bg-clip-text text-transparent">
              Community
            </span>
          </h2>
          <p className="text-lg text-slate-400 max-w-2xl mx-auto">
            From indie game studios to research institutions, Stellar Guilds
            empowers diverse communities to collaborate effectively.
          </p>
        </motion.div>

        {/* Use cases grid */}
        <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-6">
          {useCases.map((useCase, index) => (
            <UseCaseCard key={useCase.title} {...useCase} delay={index * 0.1} />
          ))}
        </div>

        {/* Testimonial */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6 }}
          viewport={{ once: true }}
          className="mt-20"
        >
          <div className="relative p-8 md:p-10 rounded-3xl bg-slate-800/30 border border-slate-700/50">
            <div className="absolute top-6 left-8 text-6xl text-violet-500/20 font-serif">
              "
            </div>

            <div className="relative z-10 max-w-3xl mx-auto text-center">
              <p className="text-xl md:text-2xl text-slate-300 leading-relaxed mb-6 italic">
                Stellar Guilds transformed how our open source community manages
                contributions. Transparent bounties and automatic payments have
                increased contributor engagement by 300%.
              </p>

              <div className="flex items-center justify-center gap-4">
                <div className="w-12 h-12 rounded-full bg-gradient-to-br from-violet-500 to-cyan-500 flex items-center justify-center text-white font-bold text-lg">
                  AK
                </div>
                <div className="text-left">
                  <p className="font-semibold text-white">Alex Kim</p>
                  <p className="text-sm text-slate-500">
                    Lead Maintainer, NovaDev
                  </p>
                </div>
              </div>
            </div>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
