import React from 'react';
import { motion } from 'framer-motion';
import { X, Check, AlertTriangle, Sparkles } from 'lucide-react';

export default function ProblemSolutionSection() {
  const problems = [
    'High platform fees eating into earnings',
    'Lack of transparent payment processes',
    'No community ownership or governance',
    'Reputation locked to single platforms',
    'Centralized dispute resolution bias',
  ];

  const solutions = [
    'Near-zero transaction fees on Stellar',
    'On-chain transparent payment contracts',
    'DAO-style guild governance',
    'Portable cross-platform reputation',
    'Community-driven dispute resolution',
  ];

  return (
    <section className="relative py-32 bg-slate-950 overflow-hidden">
      {/* Background decorations */}
      <div className="absolute inset-0">
        <div className="absolute top-1/2 left-0 w-96 h-96 bg-red-500/5 rounded-full blur-[100px]" />
        <div className="absolute top-1/2 right-0 w-96 h-96 bg-violet-500/5 rounded-full blur-[100px]" />
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
            Why Stellar Guilds?
          </span>
          <h2 className="text-3xl sm:text-4xl md:text-5xl font-bold text-white mb-6">
            A Better Way to{' '}
            <span className="bg-gradient-to-r from-violet-400 to-violet-200 bg-clip-text text-transparent">
              Build Together
            </span>
          </h2>
        </motion.div>

        <div className="grid md:grid-cols-2 gap-8 lg:gap-12">
          {/* Problem Card */}
          <motion.div
            initial={{ opacity: 0, x: -30 }}
            whileInView={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.6 }}
            viewport={{ once: true }}
            className="relative group"
          >
            <div className="absolute inset-0 bg-gradient-to-br from-red-500/10 to-transparent rounded-3xl blur-xl opacity-0 group-hover:opacity-100 transition-opacity duration-500" />
            <div className="relative p-8 lg:p-10 rounded-3xl bg-slate-900/80 border border-slate-800 hover:border-red-500/30 transition-colors duration-300">
              {/* Header */}
              <div className="flex items-center gap-4 mb-8">
                <div className="w-14 h-14 rounded-2xl bg-red-500/10 flex items-center justify-center">
                  <AlertTriangle className="w-7 h-7 text-red-400" />
                </div>
                <div>
                  <h3 className="text-2xl font-bold text-white">The Problem</h3>
                  <p className="text-slate-500 text-sm">Traditional platforms fall short</p>
                </div>
              </div>

              {/* Problems list */}
              <div className="space-y-4">
                {problems.map((problem, index) => (
                  <motion.div
                    key={index}
                    initial={{ opacity: 0, x: -20 }}
                    whileInView={{ opacity: 1, x: 0 }}
                    transition={{ duration: 0.4, delay: index * 0.1 }}
                    viewport={{ once: true }}
                    className="flex items-start gap-4 p-4 rounded-xl bg-slate-800/30 border border-slate-700/30"
                  >
                    <div className="flex-shrink-0 w-6 h-6 rounded-full bg-red-500/20 flex items-center justify-center mt-0.5">
                      <X className="w-3.5 h-3.5 text-red-400" />
                    </div>
                    <p className="text-slate-300">{problem}</p>
                  </motion.div>
                ))}
              </div>
            </div>
          </motion.div>

          {/* Solution Card */}
          <motion.div
            initial={{ opacity: 0, x: 30 }}
            whileInView={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.6 }}
            viewport={{ once: true }}
            className="relative group"
          >
            <div className="absolute inset-0 bg-gradient-to-br from-violet-500/10 to-transparent rounded-3xl blur-xl opacity-0 group-hover:opacity-100 transition-opacity duration-500" />
            <div className="relative p-8 lg:p-10 rounded-3xl bg-slate-900/80 border border-slate-800 hover:border-violet-500/30 transition-colors duration-300">
              {/* Header */}
              <div className="flex items-center gap-4 mb-8">
                <div className="w-14 h-14 rounded-2xl bg-violet-500/10 flex items-center justify-center">
                  <Sparkles className="w-7 h-7 text-violet-400" />
                </div>
                <div>
                  <h3 className="text-2xl font-bold text-white">Our Solution</h3>
                  <p className="text-slate-500 text-sm">Stellar Guilds delivers more</p>
                </div>
              </div>

              {/* Solutions list */}
              <div className="space-y-4">
                {solutions.map((solution, index) => (
                  <motion.div
                    key={index}
                    initial={{ opacity: 0, x: 20 }}
                    whileInView={{ opacity: 1, x: 0 }}
                    transition={{ duration: 0.4, delay: index * 0.1 }}
                    viewport={{ once: true }}
                    className="flex items-start gap-4 p-4 rounded-xl bg-violet-500/5 border border-violet-500/20"
                  >
                    <div className="flex-shrink-0 w-6 h-6 rounded-full bg-violet-500/20 flex items-center justify-center mt-0.5">
                      <Check className="w-3.5 h-3.5 text-violet-400" />
                    </div>
                    <p className="text-slate-300">{solution}</p>
                  </motion.div>
                ))}
              </div>
            </div>
          </motion.div>
        </div>
      </div>
    </section>
  );
}