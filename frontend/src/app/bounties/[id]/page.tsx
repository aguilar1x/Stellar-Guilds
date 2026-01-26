"use client";
import React, { useState, use } from "react";
import ReactMarkdown from "react-markdown";
import { MOCK_BOUNTIES } from "@/lib/mocks/bounties";
import { StatusBadge } from "@/features/bounties/components/BountyCard";
import { SubmissionForm } from "@/features/bounties/components/SubmissionForm";
import { toast, Toaster } from "sonner";
import {
  Clock,
  Wallet,
  ChevronLeft,
  ShieldCheck,
  ExternalLink,
  Github,
  Globe,
  Award,
  Users,
  BarChart3,
  Code2,
  CheckCircle2,
} from "lucide-react";
import Link from "next/link";

interface PageProps {
  params: Promise<{ id: string }>;
}

export default function BountyDetailPage({ params }: PageProps) {
  const resolvedParams = use(params);
  const id = resolvedParams.id;
  const bounty = MOCK_BOUNTIES.find((b) => b.id === id) || MOCK_BOUNTIES[0];

  const [viewState, setViewState] = useState<
    "idle" | "claimed" | "submitting" | "completed"
  >("idle");

  const handleClaim = () => {
    const promise = () => new Promise((resolve) => setTimeout(resolve, 1500));

    toast.promise(promise, {
      loading: "Initializing neural link to mission...",
      success: () => {
        setViewState("claimed");
        return "Mission Initialized. Good luck, contributor.";
      },
      error: "Connection failed.",
    });
  };

  const handleFinalSubmit = () => {
    toast.success("SUBMISSION RECEIVED", {
      description:
        "Your work has been encrypted and sent to the guild for review.",
      icon: <CheckCircle2 className="text-emerald-500" size={18} />,
    });
    setViewState("completed");
  };

  return (
    <div className="min-h-screen w-full bg-[#050505] text-white">
      <Toaster theme="dark" position="bottom-right" richColors />

      <div className="fixed top-0 right-0 w-[500px] h-[500px] bg-emerald-500/5 blur-[120px] rounded-full pointer-events-none" />

      <div className="max-w-7xl mx-auto p-6 lg:p-12 relative z-10">
        <Link
          href="/bounties"
          className="inline-flex items-center gap-2 text-slate-500 hover:text-emerald-500 mb-10 transition-all group"
        >
          <ChevronLeft
            size={18}
            className="group-hover:-translate-x-1 transition-transform"
          />
          <span className="text-[10px] font-black uppercase tracking-[0.3em]">
            Return to Terminal
          </span>
        </Link>

        <div className="grid grid-cols-1 lg:grid-cols-12 gap-12">
          <div className="lg:col-span-8 space-y-12">
            <section className="space-y-6">
              <div className="flex flex-wrap items-center gap-3">
                <StatusBadge
                  status={
                    viewState === "completed" ? "Under Review" : bounty.status
                  }
                />
                <div className="h-1 w-1 bg-slate-700 rounded-full" />
                <span className="text-[10px] font-mono text-emerald-500 uppercase tracking-widest bg-emerald-500/10 px-2 py-1 rounded">
                  {bounty.difficulty}
                </span>
              </div>

              <h1 className="text-5xl lg:text-6xl font-black tracking-tighter leading-none italic uppercase">
                {bounty.title}
              </h1>

              <div className="flex flex-wrap gap-2">
                {bounty.tags.map((tag) => (
                  <span
                    key={tag}
                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-white/5 border border-white/10 text-xs font-medium text-slate-300"
                  >
                    <Code2 size={12} className="text-emerald-500" />
                    {tag}
                  </span>
                ))}
              </div>
            </section>

            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <BentoStat
                icon={<Users size={16} />}
                label="Applicants"
                value={bounty.applicants.toString()}
              />
              <BentoStat
                icon={<BarChart3 size={16} />}
                label="Avg. Completion"
                value="4 Days"
              />
              <BentoStat
                icon={<Award size={16} />}
                label="Guild Rep"
                value="+150 XP"
              />
              <BentoStat
                icon={<ShieldCheck size={16} />}
                label="Security"
                value="Audited"
              />
            </div>

            <section className="relative">
              <div className="absolute -left-6 top-0 bottom-0 w-[1px] bg-gradient-to-b from-emerald-500/50 to-transparent hidden lg:block" />
              <h2 className="text-xs font-black uppercase tracking-[0.4em] text-slate-500 mb-8 flex items-center gap-3">
                Technical Requirements{" "}
                <div className="h-[1px] flex-grow bg-white/5" />
              </h2>
              <div className="prose prose-invert max-w-none prose-emerald">
                <ReactMarkdown>{bounty.description}</ReactMarkdown>
              </div>
            </section>
          </div>

          <div className="lg:col-span-4 space-y-6">
            <div className="bg-[#0A0A0A] border border-white/10 rounded-[32px] p-8   overflow-hidden">
              <div className="absolute -top-24 -right-24 w-48 h-48 bg-emerald-500/10 blur-[60px] rounded-full" />

              <div className="relative z-10">
                <p className="text-[10px] text-slate-500 uppercase font-black tracking-[0.3em] mb-4">
                  Contract Yield
                </p>
                <div className="flex items-baseline gap-3 mb-8">
                  <span className="text-6xl font-black italic tracking-tighter text-white">
                    {bounty.rewardAmount}
                  </span>
                  <span className="text-2xl font-bold text-emerald-500">
                    {bounty.tokenSymbol}
                  </span>
                </div>

                <div className="space-y-4 mb-10">
                  <SidebarInfo
                    icon={<Clock size={16} />}
                    text={`Deadline: ${bounty.deadline}`}
                  />
                  <SidebarInfo
                    icon={<Wallet size={16} />}
                    text="Funds: Locked in Escrow"
                  />
                  <SidebarInfo
                    icon={<Globe size={16} />}
                    text="Open to: Worldwide"
                  />
                </div>

                <div className="space-y-3">
                  {viewState === "idle" && (
                    <button
                      onClick={handleClaim}
                      className="w-full bg-white text-black py-5 rounded-2xl font-black uppercase tracking-widest hover:bg-emerald-500 transition-all active:scale-95 shadow-[0_0_30px_rgba(255,255,255,0.1)]"
                    >
                      Initialize Mission
                    </button>
                  )}

                  {viewState === "claimed" && (
                    <button
                      onClick={() => setViewState("submitting")}
                      className="w-full bg-emerald-500 text-black py-5 rounded-2xl font-black uppercase tracking-widest transition-all hover:bg-emerald-400"
                    >
                      Upload Submission
                    </button>
                  )}

                  {viewState === "submitting" && (
                    <SubmissionForm
                      onCancel={() => setViewState("claimed")}
                      onSubmit={handleFinalSubmit} 
                    />
                  )}

                  {viewState === "completed" && (
                    <div className="w-full bg-white/5 border border-white/10 text-emerald-500 py-5 rounded-2xl font-black uppercase tracking-widest text-center flex items-center justify-center gap-2">
                      <CheckCircle2 size={18} />
                      Submitted
                    </div>
                  )}
                </div>
              </div>
            </div>

            <div className="bg-white/5 border border-white/10 rounded-[32px] p-6 space-y-6">
              <div className="flex items-center gap-4">
                <img
                  src={bounty.guildLogo}
                  className="w-12 h-12 rounded-2xl bg-black p-1 border border-white/10"
                  alt=""
                />
                <div>
                  <h4 className="font-bold text-lg">{bounty.guildName}</h4>
                  <p className="text-[10px] text-slate-500 uppercase tracking-widest font-bold italic">
                    Official Guild Partner
                  </p>
                </div>
              </div>
              <p className="text-xs text-slate-400 leading-relaxed italic">
                We reward precision and speed. The terminal is your weapon.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

const BentoStat = ({
  icon,
  label,
  value,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
}) => (
  <div className="bg-white/5 border border-white/10 rounded-2xl p-4 hover:bg-white/[0.08] transition-colors">
    <div className="text-emerald-500 mb-2">{icon}</div>
    <p className="text-[9px] uppercase font-black text-slate-500 tracking-tighter mb-1">
      {label}
    </p>
    <p className="text-lg font-bold italic tracking-tight">{value}</p>
  </div>
);

const SidebarInfo = ({
  icon,
  text,
}: {
  icon: React.ReactNode;
  text: string;
}) => (
  <div className="flex items-center gap-3 text-xs font-medium text-slate-400">
    <div className="text-emerald-500">{icon}</div>
    {text}
  </div>
);

const SocialLink = ({ icon }: { icon: React.ReactNode }) => (
  <button className="w-8 h-8 rounded-lg bg-white/5 flex items-center justify-center text-slate-400 hover:text-white transition-all border border-white/10">
    {icon}
  </button>
);
