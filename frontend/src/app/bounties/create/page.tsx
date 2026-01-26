"use client";

import React, { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import ReactMarkdown from "react-markdown";
import {
  Edit3,
  Eye,
  Zap,
  Coins,
  Calendar,
  ShieldCheck,
  ChevronRight,
  Info,
  AlertCircle,
} from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";

const bountySchema = z.object({
  title: z
    .string()
    .min(10, "PROTOCOL ERROR: Title must be at least 10 characters")
    .max(100, "PROTOCOL ERROR: Title too long"),
  guild: z.string().min(1, "AUTH ERROR: Guild Authority must be selected"),
  reward: z
    .string()
    .refine(
      (val) => !isNaN(Number(val)) && Number(val) > 0,
      " ERROR: Payout must be a positive number",
    ),
  token: z.string().min(1, "ECON ERROR: Token type required"),
  deadline: z
    .string()
    .min(1, "ERROR: Deadline required")
    .refine(
      (val) => new Date(val) > new Date(),
      "CHRONO ERROR: Deadline must be in the future",
    ),
  description: z
    .string()
    .min(50, "INTEL ERROR: Intelligence report too brief (min 50 chars)"),
});

type BountyFormValues = z.infer<typeof bountySchema>;

export default function CreateBountyPage() {
  const [isPreview, setIsPreview] = useState(false);

  const {
    register,
    handleSubmit,
    watch,
    setValue,
    trigger,
    formState: { errors, isSubmitting, isValid },
  } = useForm<BountyFormValues>({
    resolver: zodResolver(bountySchema),
    mode: "onChange", 
    defaultValues: { token: "USDC", guild: "" },
  });

  const formData = watch();

  const onSubmit = async (data: BountyFormValues) => {
    console.log("Deploying Bounty:", data);
    await new Promise((resolve) => setTimeout(resolve, 2000));
    alert("SYSTEM: BOUNTY DEPLOYED TO MAINNET");
  };

  const handleGuildSelect = (guild: string) => {
    setValue("guild", guild, { shouldValidate: true });
  };

  return (
    <div className="min-h-screen w-full bg-[#050505] text-white p-6 lg:p-12">
      <div className="max-w-5xl mx-auto">
        <header className="flex flex-col md:flex-row md:items-center justify-between gap-6 mb-12">
          <div className="space-y-2">
            <div className="flex items-center gap-2 text-emerald-500 mb-1">
              <Zap size={14} className="fill-emerald-500" />
              <span className="text-[10px] font-black uppercase tracking-[0.4em]">
                Deployment Terminal
              </span>
            </div>
            <h1 className="text-5xl font-black italic tracking-tighter uppercase leading-none">
              Forge <span className="text-emerald-500">Mission.</span>
            </h1>
          </div>

          <div className="flex bg-white/5 p-1 rounded-xl border border-white/10 h-fit">
            <TabButton
              active={!isPreview}
              onClick={() => setIsPreview(false)}
              icon={<Edit3 size={14} />}
              label="Editor"
            />
            <TabButton
              active={isPreview}
              onClick={() => setIsPreview(true)}
              icon={<Eye size={14} />}
              label="Preview"
            />
          </div>
        </header>

        <form
          onSubmit={handleSubmit(onSubmit)}
          className="grid grid-cols-1 lg:grid-cols-12 gap-12"
        >
          <div className="lg:col-span-8">
            <AnimatePresence mode="wait">
              {!isPreview ? (
                <motion.div
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                  className="space-y-10"
                >
                  <section className="space-y-6">
                    <SectionHeader number="01" title="Mission Identity" />

                    <div className="space-y-4">
                      <div className="group">
                        <FieldLabel
                          label="Mission Title"
                          error={errors.title?.message}
                        />
                        <input
                          {...register("title")}
                          className={`w-full bg-white/5 border rounded-2xl py-4 px-6 outline-none transition-all text-lg font-bold italic ${errors.title ? "border-red-500/50 bg-red-500/5" : "border-white/10 focus:border-emerald-500/50"}`}
                          placeholder="Project Name..."
                        />
                      </div>

                      <div>
                        <FieldLabel
                          label="Guild Authority"
                          error={errors.guild?.message}
                        />
                        <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
                          {["PrivacyGuard", "Stellar", "NeonDAO"].map(
                            (guild) => (
                              <button
                                key={guild}
                                type="button"
                                onClick={() => handleGuildSelect(guild)}
                                className={`p-4 rounded-xl border text-[10px] font-black uppercase tracking-widest transition-all ${formData.guild === guild ? "bg-emerald-500 border-emerald-500 text-black shadow-[0_0_20px_rgba(16,185,129,0.2)]" : "bg-white/5 border-white/10 text-slate-500 hover:border-white/20"}`}
                              >
                                {guild}
                              </button>
                            ),
                          )}
                        </div>
                      </div>
                    </div>
                  </section>

                  <section className="space-y-6">
                    <SectionHeader number="02" title="Operational Intel" />
                    <div className="space-y-2">
                      <FieldLabel
                        label="Requirements (Markdown)"
                        error={errors.description?.message}
                      />
                      <textarea
                        {...register("description")}
                        rows={8}
                        className={`w-full bg-white/5 border rounded-2xl py-4 px-6 outline-none transition-all font-mono text-sm leading-relaxed ${errors.description ? "border-red-500/50 bg-red-500/5" : "border-white/10 focus:border-emerald-500/50"}`}
                        placeholder="Detail the parameters of the mission..."
                      />
                    </div>
                  </section>
                </motion.div>
              ) : (
                <motion.div
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  className="bg-white/[0.02] border border-white/5 rounded-[40px] p-10 min-h-[500px]"
                >
                  <h2 className="text-4xl font-black italic uppercase tracking-tighter mb-8">
                    {formData.title || "Untitled Mission"}
                  </h2>
                  <div className="prose prose-invert max-w-none prose-emerald">
                    <ReactMarkdown>
                      {formData.description || "_Intel pending..._"}
                    </ReactMarkdown>
                  </div>
                </motion.div>
              )}
            </AnimatePresence>
          </div>

          <div className="lg:col-span-4">
            <div className="bg-[#0A0A0A] border border-white/10 rounded-[32px] p-8 space-y-8 sticky top-12">
              <div className="space-y-6">
                <div>
                  <FieldLabel
                    label="Reward Structure"
                    error={errors.reward?.message}
                  />
                  <div className="flex gap-2">
                    <input
                      {...register("reward")}
                      placeholder="0.00"
                      className="flex-grow bg-white/5 border border-white/10 rounded-xl py-3 px-4 outline-none focus:border-emerald-500/50 font-bold"
                    />
                  </div>
                </div>

                <div>
                  <FieldLabel
                    label="Expiration"
                    error={errors.deadline?.message}
                  />
                  <input
                    type="date"
                    {...register("deadline")}
                    className="w-full bg-white/5 border border-white/10 rounded-xl py-3 px-4 outline-none focus:border-emerald-500/50 text-xs font-mono uppercase"
                  />
                </div>
              </div>

              <div className="pt-8 border-t border-white/5 space-y-4">
                <SummaryRow label="Security Audit" value="PASS" highlight />
                <SummaryRow label="Protocol Fee" value="2.5%" />

                <button
                  disabled={isSubmitting}
                  className={`w-full py-5 rounded-2xl font-black uppercase tracking-[0.2em] text-[10px] transition-all flex items-center justify-center gap-2 ${isValid ? "bg-emerald-500 text-black shadow-[0_0_30px_rgba(16,185,129,0.2)] hover:bg-emerald-400" : "bg-white/5 text-slate-600 cursor-not-allowed"}`}
                >
                  {isSubmitting ? (
                    "Syncing..."
                  ) : (
                    <>
                      Finalize & Deploy <ChevronRight size={14} />
                    </>
                  )}
                </button>
              </div>

              <div className="flex items-start gap-3 p-4 bg-white/5 rounded-2xl border border-white/5">
                <Info size={16} className="text-slate-500 shrink-0 mt-0.5" />
                <p className="text-[9px] text-slate-500 leading-relaxed uppercase tracking-tighter">
                  Deployment requires gas. Ensure your linked wallet contains
                  sufficient funds for escrow deposit.
                </p>
              </div>
            </div>
          </div>
        </form>
      </div>
    </div>
  );
}


const SectionHeader = ({
  number,
  title,
}: {
  number: string;
  title: string;
}) => (
  <h3 className="text-xs font-black uppercase tracking-[0.3em] text-slate-500 flex items-center gap-3">
    {number}. {title} <div className="h-[1px] flex-grow bg-white/5" />
  </h3>
);

const FieldLabel = ({ label, error }: { label: string; error?: string }) => (
  <div className="flex justify-between items-end mb-2">
    <label className="text-[10px] uppercase font-black text-slate-500 tracking-widest">
      {label}
    </label>
    <AnimatePresence>
      {error && (
        <motion.span
          initial={{ opacity: 0, x: 10 }}
          animate={{ opacity: 1, x: 0 }}
          className="text-[9px] text-red-500 font-bold uppercase italic flex items-center gap-1"
        >
          <AlertCircle size={10} /> {error}
        </motion.span>
      )}
    </AnimatePresence>
  </div>
);

const TabButton = ({ active, onClick, icon, label }: any) => (
  <button
    onClick={onClick}
    className={`flex items-center gap-2 px-6 py-2 rounded-lg text-[10px] font-black uppercase tracking-widest transition-all ${active ? "bg-white text-black shadow-lg" : "text-slate-500 hover:text-slate-300"}`}
  >
    {icon} {label}
  </button>
);

const SummaryRow = ({ label, value, highlight }: any) => (
  <div className="flex justify-between text-[10px] uppercase tracking-widest font-bold">
    <span className="text-slate-500">{label}</span>
    <span
      className={
        highlight ? "text-emerald-500 flex items-center gap-1" : "text-white"
      }
    >
      {value} {highlight && <ShieldCheck size={12} />}
    </span>
  </div>
);
